use api_example::{router, state::AppState};
use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
    response::Response,
};
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;

fn app(pool: PgPool) -> (Router, PgPool) {
    let state = AppState { pool: pool.clone() };
    (router(state), pool)
}

async fn request(
    app: &Router,
    method: &str,
    uri: &str,
    cookie: Option<&str>,
    body: &str,
) -> Response {
    let mut request = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json");
    if let Some(cookie) = cookie {
        request = request.header("cookie", cookie);
    }
    app.clone()
        .oneshot(
            request
                .body(Body::from(body.to_owned()))
                .expect("build request"),
        )
        .await
        .expect("serve request")
}

async fn json_body(response: Response) -> Value {
    let bytes = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .expect("read response");
    serde_json::from_slice(&bytes).expect("parse response JSON")
}

fn cookie(response: &Response) -> String {
    response.headers()["set-cookie"]
        .to_str()
        .expect("cookie text")
        .split(';')
        .next()
        .expect("cookie pair")
        .to_owned()
}

const LOGIN: &str = r#"{"username":"admin","password":"admin"}"#;

#[tokio::test]
async fn health_works_without_a_database_and_old_routes_are_not_registered() {
    let pool = PgPool::connect_lazy("postgres://example:example@127.0.0.1:1/example")
        .expect("configure lazy pool");
    let (app, _) = app(pool);
    let response = request(&app, "GET", "/health", None, "").await;
    assert_eq!(response.status(), StatusCode::OK);
    assert!(!response.headers().contains_key("set-cookie"));
    assert_eq!(json_body(response).await, json!({"status": "ok"}));

    for (method, uri) in [
        ("GET", "/hello"),
        ("GET", "/api/health"),
        ("POST", "/auth/login"),
        ("POST", "/auth/logout"),
        ("GET", "/users/me"),
        ("GET", "/users/search?q=ali"),
        ("GET", "/openapi.json"),
    ] {
        assert_eq!(
            request(&app, method, uri, None, "").await.status(),
            StatusCode::NOT_FOUND,
            "{method} {uri}"
        );
    }
}

#[sqlx::test]
async fn health_is_public_and_search_requires_login(pool: PgPool) {
    let (app, _) = app(pool);
    let response = request(&app, "GET", "/health", None, "").await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(json_body(response).await, json!({"status": "ok"}));
    let response = request(&app, "POST", "/api/auth/login", None, LOGIN).await;
    let session_cookie = cookie(&response);
    let response = request(
        &app,
        "GET",
        "/api/users/search?q=ALI",
        Some(&session_cookie),
        "",
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        json_body(response).await,
        json!([
            {"id": 2, "username": "alice", "display_name": "Alice Chen"}
        ])
    );
    for query in ["Nobody", "%25", "%27%20OR%201%3D1--"] {
        let response = request(
            &app,
            "GET",
            &format!("/api/users/search?q={query}"),
            Some(&session_cookie),
            "",
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(json_body(response).await, json!([]));
    }
}

#[sqlx::test]
async fn rejects_missing_empty_and_long_queries(pool: PgPool) {
    let (app, _) = app(pool);
    let response = request(&app, "POST", "/api/auth/login", None, LOGIN).await;
    let session_cookie = cookie(&response);
    for uri in [
        "/api/users/search".to_owned(),
        "/api/users/search?q=".to_owned(),
        "/api/users/search?q=%20".to_owned(),
        format!("/api/users/search?q={}", "a".repeat(101)),
    ] {
        let response = request(&app, "GET", &uri, Some(&session_cookie), "").await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{uri}");
        assert!(json_body(response).await["error"].is_string());
    }
}

#[sqlx::test]
async fn login_reads_current_database_user_and_logout_revokes_cookie(pool: PgPool) {
    let (app, pool) = app(pool);
    assert_eq!(
        request(&app, "GET", "/api/users/me", None, "")
            .await
            .status(),
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        request(
            &app,
            "GET",
            "/api/users/me",
            Some("api_example_session=invalid"),
            ""
        )
        .await
        .status(),
        StatusCode::UNAUTHORIZED
    );
    let response = request(&app, "POST", "/api/auth/login", None, LOGIN).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["cache-control"], "no-store");
    let set_cookie = response.headers()["set-cookie"].to_str().expect("cookie");
    assert!(set_cookie.contains("HttpOnly"));
    assert!(set_cookie.contains("Secure"));
    assert!(set_cookie.contains("SameSite=Strict"));
    let session_cookie = cookie(&response);
    assert_eq!(json_body(response).await["username"], "admin");
    sqlx::query("UPDATE users SET display_name = 'Updated Admin' WHERE username = 'admin'")
        .execute(&pool)
        .await
        .expect("update user");
    let response = request(&app, "GET", "/api/users/me", Some(&session_cookie), "").await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["cache-control"], "no-store");
    assert!(
        response.headers()["set-cookie"]
            .to_str()
            .expect("renewed session cookie")
            .contains("Max-Age=3600")
    );
    assert_eq!(json_body(response).await["display_name"], "Updated Admin");
    assert_eq!(
        request(&app, "POST", "/api/auth/logout", Some(&session_cookie), "")
            .await
            .status(),
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        request(&app, "GET", "/api/users/me", Some(&session_cookie), "")
            .await
            .status(),
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        request(&app, "POST", "/api/auth/logout", None, "")
            .await
            .status(),
        StatusCode::UNAUTHORIZED
    );
}

#[sqlx::test]
async fn rejects_invalid_credentials_and_json(pool: PgPool) {
    let (app, _) = app(pool);
    for body in [
        r#"{"username":"admin","password":"wrong"}"#,
        r#"{"username":"alice","password":"admin"}"#,
    ] {
        let response = request(&app, "POST", "/api/auth/login", None, body).await;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert!(!response.headers().contains_key("set-cookie"));
    }
    for body in ["{", r#"{"username":"admin"}"#] {
        let response = request(&app, "POST", "/api/auth/login", None, body).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert!(json_body(response).await["error"].is_string());
    }
}

#[sqlx::test]
async fn login_rotates_session_and_deleted_users_lose_access(pool: PgPool) {
    let (app, pool) = app(pool);
    let first = request(&app, "POST", "/api/auth/login", None, LOGIN).await;
    let old_cookie = cookie(&first);
    let second = request(&app, "POST", "/api/auth/login", Some(&old_cookie), LOGIN).await;
    assert_eq!(second.status(), StatusCode::OK);
    let new_cookie = cookie(&second);
    assert_ne!(new_cookie, old_cookie);
    assert_eq!(
        request(&app, "GET", "/api/users/me", Some(&old_cookie), "")
            .await
            .status(),
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        request(&app, "GET", "/api/users/me", Some(&new_cookie), "")
            .await
            .status(),
        StatusCode::OK
    );
    sqlx::query("DELETE FROM users WHERE username = 'admin'")
        .execute(&pool)
        .await
        .expect("delete user");
    assert_eq!(
        request(&app, "GET", "/api/users/me", Some(&new_cookie), "")
            .await
            .status(),
        StatusCode::UNAUTHORIZED
    );
    for (method, uri) in [
        ("GET", "/api/users/search?q=ali"),
        ("POST", "/api/auth/logout"),
    ] {
        assert_eq!(
            request(&app, method, uri, Some(&new_cookie), "")
                .await
                .status(),
            StatusCode::UNAUTHORIZED
        );
    }
}

#[sqlx::test]
async fn spec_describes_routes_query_and_cookie_auth(pool: PgPool) {
    let (app, _) = app(pool);
    let response = request(&app, "GET", "/api/openapi.json", None, "").await;
    assert_eq!(response.status(), StatusCode::OK);
    let spec = json_body(response).await;
    assert_eq!(spec["paths"].as_object().expect("API paths").len(), 5);
    for (path, method) in [
        ("/health", "get"),
        ("/api/auth/login", "post"),
        ("/api/auth/logout", "post"),
        ("/api/users/me", "get"),
        ("/api/users/search", "get"),
    ] {
        assert!(spec["paths"][path][method].is_object(), "{method} {path}");
    }
    for (path, method) in [
        ("/api/users/me", "get"),
        ("/api/users/search", "get"),
        ("/api/auth/logout", "post"),
    ] {
        assert_eq!(
            spec["paths"][path][method]["security"],
            json!([{"session": []}])
        );
    }
    assert!(!spec.to_string().contains("private_notes"));
    let properties = spec["components"]["schemas"]["UserResponse"]["properties"]
        .as_object()
        .expect("public user properties");
    assert_eq!(properties.len(), 3);
    for field in ["id", "username", "display_name"] {
        assert!(properties.contains_key(field));
    }
    assert_eq!(
        spec["components"]["securitySchemes"]["session"]["in"],
        "cookie"
    );
    assert_eq!(
        spec["components"]["securitySchemes"]["session"]["name"],
        "api_example_session"
    );
    let query = &spec["paths"]["/api/users/search"]["get"]["parameters"][0];
    assert_eq!(query["name"], "q");
    assert_eq!(query["in"], "query");
    assert_eq!(query["required"], true);
}

#[sqlx::test]
async fn protected_routes_reject_missing_and_invalid_sessions(pool: PgPool) {
    let (app, _) = app(pool);
    for (method, uri) in [
        ("GET", "/api/users/me"),
        ("GET", "/api/users/search?q=ali"),
        ("GET", "/api/users/search"),
        ("POST", "/api/auth/logout"),
    ] {
        for cookie in [None, Some("api_example_session=invalid")] {
            let response = request(&app, method, uri, cookie, "").await;
            assert_eq!(
                response.status(),
                StatusCode::UNAUTHORIZED,
                "{method} {uri}"
            );
            assert_eq!(
                json_body(response).await,
                json!({"error": "Invalid credentials or session"})
            );
        }
    }
}

#[sqlx::test]
async fn api_user_responses_exclude_private_database_fields(pool: PgPool) {
    let notes: Vec<String> = sqlx::query_scalar("SELECT private_notes FROM users")
        .fetch_all(&pool)
        .await
        .expect("read private notes");
    assert_eq!(notes.len(), 3);
    assert!(notes.iter().all(|note| !note.is_empty()));
    let (app, _) = app(pool);
    let response = request(&app, "POST", "/api/auth/login", None, LOGIN).await;
    assert_eq!(response.status(), StatusCode::OK);
    let session_cookie = cookie(&response);
    let login_user = json_body(response).await;
    let me = request(&app, "GET", "/api/users/me", Some(&session_cookie), "").await;
    assert_eq!(me.status(), StatusCode::OK);
    let me_user = json_body(me).await;
    let search = request(
        &app,
        "GET",
        "/api/users/search?q=a",
        Some(&session_cookie),
        "",
    )
    .await;
    assert_eq!(search.status(), StatusCode::OK);
    let search_users = json_body(search).await;
    let mut users = vec![login_user, me_user];
    users.extend(
        search_users
            .as_array()
            .expect("search results")
            .iter()
            .cloned(),
    );
    assert_eq!(users.len(), 5);
    for user in users {
        let object = user.as_object().expect("user object");
        assert_eq!(object.len(), 3);
        for field in ["id", "username", "display_name"] {
            assert!(object.contains_key(field));
        }
        let serialized = user.to_string();
        assert!(!serialized.contains("private_notes"));
        for note in &notes {
            assert!(!serialized.contains(note));
        }
    }
}
