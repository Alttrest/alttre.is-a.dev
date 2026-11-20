use anyhow::{Context, Result};
use ssh_portfolio::ssh::{self, ServerConfig};
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

/// Resolves on Ctrl+C or (unix) SIGTERM.
async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };
    #[cfg(unix)]
    let term = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut s) => {
                s.recv().await;
            }
            Err(_) => std::future::pending::<()>().await,
        }
    };
    #[cfg(not(unix))]
    let term = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = term => {},
    }
    tracing::info!("shutdown signal received");
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let cfg = ServerConfig::from_env();
    let key = ssh::load_or_generate_host_key(&cfg.host_key_path)?;
    let addr = cfg.listen_addr();
    let listener = TcpListener::bind(&addr)
        .await
        .with_context(|| format!("binding {addr}"))?;
    let local = listener.local_addr()?;

    tracing::info!("alttre.is-a.dev SSH portfolio listening on {local}");
    tracing::info!("host key fingerprint: {}", ssh::fingerprint(&key));
    tracing::info!(
        "max sessions: {}, idle timeout: {}s",
        cfg.max_sessions,
        cfg.inactivity_timeout_secs
    );
    tracing::info!("connect with: ssh -p {} localhost", local.port());

    ssh::serve(&cfg, key, listener, shutdown_signal()).await?;
    tracing::info!("bye");
    Ok(())
}
