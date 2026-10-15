# API example

> [!NOTE]
> Install [mise](https://mise.jdx.dev/installing-mise.html):
>
> ```sh
> curl -fsSL https://mise.run | sh
> ```

A small Rust API template using Axum, SQLx, Serde, tracing, tower-sessions, and Utoipa.

## Docker development

```sh
docker compose -f compose.yaml -f compose.dev.yaml up --build
```

Starts PostgreSQL and the watch server behind Traefik at `https://localhost`. Source changes rebuild with watchexec.

## Local Rust development

Runs rust natively on local, supply your own PostgreSQL endpoint or run one in docker:

```sh
export DATABASE_URL=postgres://user:password@localhost:5432/your_database
mise run dev
```

Runs HTTPS at `https://localhost:3000` with watchexec rebuilds. To trust or remove the certificate in your macOS user keychain:

```sh
mise run cert:install-macos
mise run cert:uninstall-macos
```

## Release build with Compose

```sh
docker compose up --build
```

Compose starts Traefik, PostgreSQL, and builds the app. The app waits for a healthy database before starting, applies the bundled SQL migration, and seeds `admin`, `alice`, and `bob`.

The API is available at `https://localhost`. Compose checks `/health` before starting Traefik; this liveness check does not query the database.

## Process envs

| Variable | Default | Purpose |
| --- | --- | --- |
| `BIND_ADDR` | `0.0.0.0:3000` | Listen address. Native HTTPS dev defaults to `127.0.0.1:3000` |
| `DATABASE_URL` | Required | PostgreSQL connection URL. Compose supplies it for the app |
| `RUST_LOG` | `api_example=info,tower_http=debug` | Log filter |

## Routes

| Method | Path | Auth | Result |
| --- | --- | --- | --- |
| GET | `/health` | Public | `{"status":"ok"}` |
| POST | `/api/auth/login` | Public | User object and session cookie |
| POST | `/api/auth/logout` | Session | Deletes the current session, returns 204 |
| GET | `/api/users/me` | Session | Current user from PostgreSQL, or 401 |
| GET | `/api/users/search?q=ali` | Session | Matching users, sorted by username, up to 50 |
| GET | `/api/openapi.json` | Public, debug builds only | Generated OpenAPI spec |

Login accepts `{"username":"admin","password":"admin"}`. User objects contain `id`, `username`, and `display_name`.

Search requires `q` with 1 to 100 bytes after trimming. It matches a substring of the username or display name, ignoring ASCII case. SQL parameters are bound. `%` and `_` are literal characters.

```sh
curl -k https://localhost/health
curl -k -c /tmp/api-example.cookies \
  -H 'Content-Type: application/json' \
  -d '{"username":"admin","password":"admin"}' \
  https://localhost/api/auth/login
curl -k -b /tmp/api-example.cookies 'https://localhost/api/users/search?q=ali'
curl -k -b /tmp/api-example.cookies https://localhost/api/users/me
curl -k -b /tmp/api-example.cookies -X POST https://localhost/api/auth/logout
```

## OpenAPI Spec

```sh
mise run openapi
```

Writes `openapi.json` in the repository root using the registered routes, no server or database needs to be running. Debug builds also serve `/api/openapi.json`.

## Example code

- `src/main.rs`: tracing, configuration, listener, and shutdown.
- `src/lib.rs`: application assembly and middleware.
- `src/state.rs`: shared dependencies and `FromRef` substate extraction.
- `src/api/mod.rs`: combines domain routers.
- `src/api/auth/`: login and logout, each in its own file.
- `src/api/users/`: current user and search, each in its own file.
- `src/api/health.rs`: public liveness check.
- `src/api/openapi.rs`: API metadata and cookie auth declaration.
- `src/session.rs`: `SignedIn` extractor loads the current database user or returns 401.
- `src/api/users/schema.rs`: public user fields and `From<db::UserRecord>` conversion.
- `src/db.rs`: pooled PostgreSQL queries and row mapping.
- `src/error.rs`: JSON errors and internal error logging.
- `migrations/`: schema and seed data, embedded in the binary.
- `examples/dev-server.rs`: local HTTPS runner.
- `examples/openapi.rs`: exports the generated OpenAPI spec to a file.
- `mise.toml`: setup, watch mode, and explicit macOS trust tasks.
- `scripts/dev-cert.sh`: certificate generation and user keychain actions.
- `scripts/dev-watch.sh`: shared host and container watch command.
- `compose.dev.yaml`: mounted-code dev server and PostgreSQL.

SQLx uses `query_as` with `FromRow`, so building does not need a running database or SQLx CLI. Integration tests use `#[sqlx::test]` to create a separate PostgreSQL database per test and apply migrations.

`SignedIn` loads the current database user on protected requests. `db::UserRecord` includes `private_notes` and has no serialization or OpenAPI derives. Responses use `api::users::schema::UserResponse`, converted with `From<db::UserRecord>` to expose only public fields.

Each API domain registers its handlers with `OpenApiRouter::routes(routes!(handler))` in `mod.rs`. This registers the HTTP route and collects its OpenAPI definition from `#[utoipa::path]`. Handler files keep their request types and OpenAPI descriptions beside the handler. Each domain uses paths relative to its prefix. `src/api/mod.rs` nests auth and users under `/api/auth` and `/api/users`. The public `/health` check stays at the root. Add a domain by nesting its `OpenApiRouter` under the matching URL prefix. Nesting applies the prefix to both HTTP routes and OpenAPI paths.

All domain routers use `AppState`. Startup builds it and supplies it once through `.with_state(state)`. Add shared dependencies as fields in `src/state.rs`. Handlers extract the pool with `State<PgPool>` through `FromRef<AppState>`. A handler that needs the whole state can take `State<AppState>`. Cloning the state shares the existing SQLx pool.

> [!WARNING]
> Auth code is an example and not prod ready.
> 
> The fixed credentials are demonstration code. There is no password storage, signup, or account management. Search requires a session.
> 
> Sessions live in memory and expire after one hour of inactivity. Restarting clears sessions. The cookie uses `Secure`, `HttpOnly`, and `SameSite=Strict`. Before using this as a deployed app, replace the demo credentials and choose a persistent session store.

## Crate references

The implementation follows these upstream references:

- [Axum extractors](https://docs.rs/axum/latest/axum/extract/) and [Query](https://docs.rs/axum/latest/axum/extract/struct.Query.html)
- [Axum application state and substates](https://docs.rs/axum/latest/axum/extract/struct.State.html#substates)
- [SQLx query_as](https://docs.rs/sqlx/latest/sqlx/fn.query_as.html) and [embedded migrations](https://docs.rs/sqlx/latest/sqlx/macro.migrate.html)
- [Serde derive](https://serde.rs/derive.html) and [serde_json](https://docs.rs/serde_json/latest/serde_json/)
- [tower-sessions extractor pattern](https://docs.rs/tower-sessions/latest/tower_sessions/#extractor-pattern) and [session rotation](https://docs.rs/tower-sessions/latest/tower_sessions/struct.Session.html#method.cycle_id)
- [tracing](https://docs.rs/tracing/latest/tracing/), [EnvFilter](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html), and [HTTP tracing](https://docs.rs/tower-http/latest/tower_http/trace/)
- [Tokio shutdown](https://tokio.rs/tokio/topics/shutdown)
- [Utoipa paths](https://docs.rs/utoipa/latest/utoipa/attr.path.html) and [cookie security](https://docs.rs/utoipa/latest/utoipa/openapi/security/)
- [Utoipa Axum route registration](https://docs.rs/utoipa-axum/latest/utoipa_axum/)
- [Tower ServiceExt](https://docs.rs/tower/latest/tower/util/trait.ServiceExt.html) (router tests)
- [Axum Server TLS](https://docs.rs/axum-server/latest/axum_server/tls_rustls/)
- [mise tasks](https://mise.jdx.dev/tasks/)
- [Watchexec CLI](https://github.com/watchexec/watchexec/blob/main/doc/watchexec.1.md)
