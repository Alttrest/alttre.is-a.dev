//! SSH server layer: config, host key handling and the `russh` server.

pub mod input;
pub mod session;
pub mod terminal;

use std::future::Future;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use russh::keys::ssh_key::LineEnding;
use russh::keys::{Algorithm, HashAlg, PrivateKey, load_secret_key};
use russh::server::{Config, Server};
use tokio::net::TcpListener;
use tokio::sync::Semaphore;

use self::session::ClientSession;

/// Runtime configuration of the SSH portfolio server.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub bind_addr: String,
    pub port: u16,
    /// Path of the Ed25519 host key. Empty string = ephemeral key.
    pub host_key_path: String,
    pub max_sessions: usize,
    pub inactivity_timeout_secs: u64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_addr: "0.0.0.0".into(),
            port: 2222,
            host_key_path: "host_key".into(),
            max_sessions: 100,
            inactivity_timeout_secs: 600,
        }
    }
}

impl ServerConfig {
    /// Build from environment variables (see `main.rs`), falling back to defaults.
    pub fn from_env() -> Self {
        fn get<T: std::str::FromStr>(name: &str, default: T) -> T {
            std::env::var(name)
                .ok()
                .and_then(|v| v.trim().parse().ok())
                .unwrap_or(default)
        }
        let d = Self::default();
        Self {
            bind_addr: std::env::var("SSH_BIND")
                .ok()
                .filter(|s| !s.trim().is_empty())
                .unwrap_or(d.bind_addr),
            port: get("SSH_PORT", d.port),
            host_key_path: std::env::var("SSH_HOST_KEY").unwrap_or(d.host_key_path),
            max_sessions: get("MAX_SESSIONS", d.max_sessions),
            inactivity_timeout_secs: get("IDLE_TIMEOUT_SECS", d.inactivity_timeout_secs),
        }
    }

    pub fn listen_addr(&self) -> String {
        if self.bind_addr.contains(':') && !self.bind_addr.starts_with('[') {
            format!("[{}]:{}", self.bind_addr, self.port) // bare IPv6
        } else {
            format!("{}:{}", self.bind_addr, self.port)
        }
    }
}

/// SHA-256 fingerprint (`SHA256:...`) of a host key.
pub fn fingerprint(key: &PrivateKey) -> String {
    key.public_key().fingerprint(HashAlg::Sha256).to_string()
}

fn generate_key() -> Result<PrivateKey> {
    PrivateKey::random(&mut rand::rng(), Algorithm::Ed25519)
        .map_err(|e| anyhow::anyhow!("host key generation failed: {e}"))
}

/// Load the host key from `path`, generating + persisting a new Ed25519 key
/// if the file does not exist. An empty path yields an ephemeral key.
pub fn load_or_generate_host_key(path: &str) -> Result<PrivateKey> {
    if path.trim().is_empty() {
        tracing::warn!("no host key path set: using an ephemeral host key");
        return generate_key();
    }
    let p = Path::new(path);
    if p.exists() {
        return load_secret_key(p, None).with_context(|| format!("loading host key {path}"));
    }
    let key = generate_key()?;
    if let Some(parent) = p.parent().filter(|d| !d.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent).with_context(|| format!("creating {parent:?}"))?;
    }
    key.write_openssh_file(p, LineEnding::LF)
        .with_context(|| format!("writing host key {path}"))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(p, std::fs::Permissions::from_mode(0o600))
            .with_context(|| format!("chmod 600 {path}"))?;
    }
    tracing::info!("generated new Ed25519 host key at {path}");
    Ok(key)
}

/// Build the `russh` server config.
pub fn russh_config(cfg: &ServerConfig, key: PrivateKey) -> Config {
    let idle = Duration::from_secs(cfg.inactivity_timeout_secs.max(1));
    Config {
        keys: vec![key],
        // Transport-level backstop; the session actor enforces the real idle limit.
        inactivity_timeout: Some(idle + Duration::from_secs(60)),
        keepalive_interval: Some(Duration::from_secs(30)),
        keepalive_max: 3,
        auth_rejection_time: Duration::from_secs(1),
        auth_rejection_time_initial: Some(Duration::from_secs(0)),
        nodelay: true,
        ..Default::default()
    }
}

/// Creates one isolated [`ClientSession`] per TCP connection.
#[derive(Clone)]
pub struct PortfolioServer {
    sessions: Arc<Semaphore>,
    idle_timeout: Duration,
}

impl PortfolioServer {
    pub fn new(cfg: &ServerConfig) -> Self {
        Self {
            sessions: Arc::new(Semaphore::new(cfg.max_sessions.max(1))),
            idle_timeout: Duration::from_secs(cfg.inactivity_timeout_secs.max(1)),
        }
    }
}

impl Server for PortfolioServer {
    type Handler = ClientSession;

    fn new_client(&mut self, peer_addr: Option<std::net::SocketAddr>) -> ClientSession {
        tracing::debug!(peer = ?peer_addr, "new connection");
        ClientSession::new(peer_addr, self.sessions.clone(), self.idle_timeout)
    }

    fn handle_session_error(&mut self, error: anyhow::Error) {
        tracing::debug!("session error: {error:#}");
    }
}

/// Serve forever on `listener` (loads / generates the host key itself).
pub async fn run_server(cfg: ServerConfig, listener: TcpListener) -> Result<()> {
    run_server_until(cfg, listener, std::future::pending::<()>()).await
}

/// Serve on `listener` until `shutdown` completes, then disconnect clients gracefully.
pub async fn run_server_until(
    cfg: ServerConfig,
    listener: TcpListener,
    shutdown: impl Future<Output = ()>,
) -> Result<()> {
    let key = load_or_generate_host_key(&cfg.host_key_path)?;
    serve(&cfg, key, listener, shutdown).await
}

/// Like [`run_server_until`] but with an already loaded host key.
pub async fn serve(
    cfg: &ServerConfig,
    key: PrivateKey,
    listener: TcpListener,
    shutdown: impl Future<Output = ()>,
) -> Result<()> {
    let config = Arc::new(russh_config(cfg, key));
    let mut server = PortfolioServer::new(cfg);
    let mut running = server.run_on_socket(config, &listener);
    let handle = running.handle();
    tokio::pin!(shutdown);
    tokio::select! {
        res = &mut running => res.context("ssh server stopped")?,
        _ = &mut shutdown => {
            handle.shutdown("server shutting down".into());
            let _ = tokio::time::timeout(Duration::from_secs(5), &mut running).await;
            // Give per-connection tasks a moment to send their disconnect.
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ephemeral_key() {
        let k = load_or_generate_host_key("").unwrap();
        assert!(fingerprint(&k).starts_with("SHA256:"));
    }

    #[test]
    fn key_persists_and_reloads() {
        let dir = std::env::temp_dir().join(format!("sshp-key-{}", std::process::id()));
        let path = dir.join("nested").join("host_key");
        let path = path.to_str().unwrap();
        let a = load_or_generate_host_key(path).unwrap();
        assert!(Path::new(path).exists());
        let b = load_or_generate_host_key(path).unwrap();
        assert_eq!(fingerprint(&a), fingerprint(&b));
        let _ = std::fs::remove_dir_all(dir);
    }

    // ---- end-to-end tests with a real russh client over loopback ----

    use russh::ChannelMsg;
    use russh::client;
    use tokio::sync::oneshot;

    struct TestClient;
    impl client::Handler for TestClient {
        type Error = anyhow::Error;
        async fn check_server_key(
            &mut self,
            _key: &russh::keys::PublicKeyOrCertificate,
        ) -> Result<bool, Self::Error> {
            Ok(true)
        }
    }

    async fn start(
        max_sessions: usize,
    ) -> (
        std::net::SocketAddr,
        oneshot::Sender<()>,
        tokio::task::JoinHandle<Result<()>>,
    ) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let cfg = ServerConfig {
            bind_addr: "127.0.0.1".into(),
            port: 0,
            host_key_path: String::new(),
            max_sessions,
            inactivity_timeout_secs: 30,
        };
        let (tx, rx) = oneshot::channel::<()>();
        let task = tokio::spawn(run_server_until(cfg, listener, async move {
            let _ = rx.await;
        }));
        (addr, tx, task)
    }

    async fn connect(addr: std::net::SocketAddr) -> client::Handle<TestClient> {
        let mut c = client::connect(Arc::new(client::Config::default()), addr, TestClient)
            .await
            .unwrap();
        assert!(c.authenticate_none("visitor").await.unwrap().success());
        c
    }

    /// Read channel messages until `pred(output)` or the channel ends.
    async fn read_until(
        ch: &mut russh::Channel<client::Msg>,
        out: &mut Vec<u8>,
        status: &mut Option<u32>,
        pred: impl Fn(&str) -> bool,
    ) -> bool {
        loop {
            if pred(&String::from_utf8_lossy(out)) {
                return true;
            }
            match ch.wait().await {
                Some(ChannelMsg::Data { data }) => out.extend_from_slice(&data),
                Some(ChannelMsg::ExitStatus { exit_status }) => *status = Some(exit_status),
                Some(_) => {}
                None => return pred(&String::from_utf8_lossy(out)),
            }
        }
    }

    #[tokio::test]
    async fn e2e_exec_gets_business_card() {
        let (addr, stop, server) = start(10).await;
        tokio::time::timeout(Duration::from_secs(15), async {
            let c = connect(addr).await;
            let mut ch = c.channel_open_session().await.unwrap();
            ch.exec(true, "whoami").await.unwrap();
            let (mut out, mut status) = (Vec::new(), None);
            read_until(&mut ch, &mut out, &mut status, |_| false).await;
            let s = String::from_utf8_lossy(&out);
            assert!(s.contains("Ali Turan") && s.contains("alttre.is-a.dev"), "{s:?}");
            assert_eq!(status, Some(0));
        })
        .await
        .expect("timeout");
        let _ = stop.send(());
        let _ = server.await;
    }

    #[tokio::test]
    async fn e2e_interactive_session_resize_and_quit() {
        let (addr, stop, server) = start(10).await;
        tokio::time::timeout(Duration::from_secs(15), async {
            let c = connect(addr).await;
            let mut ch = c.channel_open_session().await.unwrap();
            ch.request_pty(true, "xterm-256color", 100, 30, 0, 0, &[])
                .await
                .unwrap();
            ch.request_shell(true).await.unwrap();
            let (mut out, mut status) = (Vec::new(), None);
            assert!(
                read_until(&mut ch, &mut out, &mut status, |s| s
                    .contains(terminal::ENTER_SEQ))
                .await,
                "no alt-screen enter sequence"
            );
            // Resize + a click + a wheel tick + junk must not break anything.
            ch.window_change(60, 20, 0, 0).await.unwrap();
            ch.data(&b"\x1b[<0;5;1M\x1b[<65;5;5M\x1b[Z"[..]).await.unwrap();
            // Ctrl+C quits and restores the terminal.
            ch.data(&[0x03u8][..]).await.unwrap();
            assert!(
                read_until(&mut ch, &mut out, &mut status, |s| s
                    .contains(terminal::RESTORE_SEQ))
                .await,
                "no restore sequence"
            );
            // Channel is closed by the server.
            read_until(&mut ch, &mut out, &mut status, |_| false).await;
            assert_eq!(status, Some(0));
        })
        .await
        .expect("timeout");
        let _ = stop.send(());
        let _ = server.await;
    }

    #[tokio::test]
    async fn e2e_session_limit_is_polite() {
        let (addr, stop, server) = start(1).await;
        tokio::time::timeout(Duration::from_secs(20), async {
            let c1 = connect(addr).await;
            let mut ch1 = c1.channel_open_session().await.unwrap();
            ch1.request_pty(true, "xterm", 80, 24, 0, 0, &[]).await.unwrap();
            ch1.request_shell(true).await.unwrap();
            let (mut o1, mut s1) = (Vec::new(), None);
            assert!(
                read_until(&mut ch1, &mut o1, &mut s1, |s| s.contains(terminal::ENTER_SEQ)).await
            );

            let c2 = connect(addr).await;
            let mut ch2 = c2.channel_open_session().await.unwrap();
            ch2.request_pty(true, "xterm", 80, 24, 0, 0, &[]).await.unwrap();
            ch2.request_shell(true).await.unwrap();
            let (mut o2, mut s2) = (Vec::new(), None);
            read_until(&mut ch2, &mut o2, &mut s2, |_| false).await;
            let s = String::from_utf8_lossy(&o2);
            assert!(s.contains("busy"), "{s:?}");
            assert_eq!(s2, Some(1));

            ch1.data(&[0x03u8][..]).await.unwrap();
            read_until(&mut ch1, &mut o1, &mut s1, |_| false).await;
        })
        .await
        .expect("timeout");
        let _ = stop.send(());
        let _ = server.await;
    }

    #[test]
    fn listen_addr_formats() {
        let mut c = ServerConfig::default();
        assert_eq!(c.listen_addr(), "0.0.0.0:2222");
        c.bind_addr = "::1".into();
        assert_eq!(c.listen_addr(), "[::1]:2222");
    }
}
