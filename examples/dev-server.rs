//! Local HTTPS runner. Its TLS dependency is a dev-dependency.

use std::{net::SocketAddr, path::Path, time::Duration};

use api_example::{db, router, state::AppState};
use axum_server::{Handle, tls_rustls::RustlsConfig};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "api_example=info,dev_server=info,tower_http=debug".into()),
        )
        .init();

    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let address: SocketAddr = std::env::var("BIND_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:3000".into())
        .parse()?;
    let certs = Path::new(env!("CARGO_MANIFEST_DIR")).join(".dev/certs");
    if !certs.join("localhost.pem").is_file() || !certs.join("localhost-key.pem").is_file() {
        return Err("Local certificate missing. Run mise run setup or mise run dev.".into());
    }
    // SQLx already enables Rustls's ring provider, so TLS uses that provider too.
    let tls =
        RustlsConfig::from_pem_file(certs.join("localhost.pem"), certs.join("localhost-key.pem"))
            .await?;
    let pool = db::connect(&url).await?;
    let state = AppState { pool: pool.clone() };
    let handle = Handle::new();
    let shutdown = tokio::spawn(shutdown(handle.clone()));
    tracing::info!(%address, "local HTTPS server starting");
    axum_server::bind_rustls(address, tls)
        .handle(handle)
        .serve(router(state).into_make_service())
        .await?;
    shutdown.abort();
    pool.close().await;
    Ok(())
}

async fn shutdown(handle: Handle<SocketAddr>) {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("install Ctrl-C handler");
    };
    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("install SIGTERM handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
    handle.graceful_shutdown(Some(Duration::from_secs(5)));
}
