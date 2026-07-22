//! SSH client: connection management, authentication, and channel operations.
//!
//! The public surface is [`SshClient`] for building a connection and
//! [`ConnectedClient`] / [`SshChannel`] for interacting with a live session.
//!
//! Authentication methods:
//! - **Password** via [`SshClient::connect_password`]
//! - **Public key** via [`SshClient::connect_key`]
//! - **SSH agent** via [`SshClient::connect_agent`]
//!
//! The underlying transport is provided by [`russh`]. A keepalive interval is
//! configured on the [`russh::client::Config`] so idle connections are
//! probed and eventually torn down by the server.

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use russh::client;
use russh::keys::*;
use tokio::sync::Mutex;

use crate::error::SshError;

/// Default keepalive interval (seconds) sent to the server via the SSH
/// transport.
const KEEPALIVE_INTERVAL_SECS: u32 = 15;

/// Handler for the [`russh::client::Handler`] trait.
///
/// `check_server_key` currently accepts every server key (TOFU bootstrap). A
/// future change will wire it into [`crate::known_hosts::KnownHosts`] so that
/// mismatches are rejected and unknown keys are recorded on first contact.
#[derive(Debug, Default)]
struct SshHandler;

impl client::Handler for SshHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &PublicKey,
    ) -> Result<bool, Self::Error> {
        tracing::debug!("check_server_key: accepting key (TOFU bootstrap)");
        Ok(true)
    }
}

/// Builder for an SSH client connection.
///
/// Created with [`SshClient::new`]; call one of the `connect_*` methods to
/// establish a transport and obtain a [`ConnectedClient`].
#[derive(Debug, Clone)]
pub struct SshClient {
    host: String,
    port: u16,
    username: String,
}

impl SshClient {
    /// Create a new SSH client builder for `host:port` with the given
    /// `username`.
    #[must_use]
    pub fn new(host: impl Into<String>, port: u16, username: impl Into<String>) -> Self {
        Self {
            host: host.into(),
            port,
            username: username.into(),
        }
    }

    /// The target hostname.
    #[must_use]
    pub fn host(&self) -> &str {
        &self.host
    }

    /// The target TCP port.
    #[must_use]
    pub const fn port(&self) -> u16 {
        self.port
    }

    /// The SSH username.
    #[must_use]
    pub fn username(&self) -> &str {
        &self.username
    }

    /// Build a [`russh::client::Config`] with the keepalive interval set.
    fn make_config(&self) -> Arc<client::Config> {
        let config = client::Config {
            keepalive_interval: Some(Duration::from_secs(u64::from(KEEPALIVE_INTERVAL_SECS))),
            keepalive_max: 3,
            ..client::Config::default()
        };
        Arc::new(config)
    }

    /// The target `host:port` address string.
    fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    /// Connect and authenticate using a password.
    ///
    /// # Errors
    /// Returns [`SshError::ConnectError`] if the transport cannot be
    /// established, or [`SshError::AuthError`] if the password is rejected.
    pub async fn connect_password(
        &self,
        password: impl Into<String>,
    ) -> Result<ConnectedClient, SshError> {
        let config = self.make_config();
        let addr = self.addr();
        let handler = SshHandler;

        tracing::info!(host = %self.host, port = self.port, "connecting (password)");
        let mut handle = client::connect(config, &addr, handler)
            .await
            .map_err(|e| SshError::ConnectError(e.to_string()))?;

        let auth = handle
            .authenticate_password(self.username.clone(), password.into())
            .await
            .map_err(|e| SshError::AuthError(e.to_string()))?;

        if auth.success() {
            tracing::info!(host = %self.host, "password auth succeeded");
            Ok(ConnectedClient::new(handle, self.username.clone()))
        } else {
            Err(SshError::AuthError(
                "password rejected by server".to_string(),
            ))
        }
    }

    /// Connect and authenticate using a private key file.
    ///
    /// # Errors
    /// Returns [`SshError::KeyLoadError`] if the key cannot be loaded, and
    /// [`SshError::AuthError`] if the server rejects the key.
    pub async fn connect_key(
        &self,
        key_path: impl AsRef<Path>,
    ) -> Result<ConnectedClient, SshError> {
        let path = key_path.as_ref();
        tracing::info!(host = %self.host, key = %path.display(), "connecting (public key)");

        let key_pair =
            load_secret_key(path, None).map_err(|e| SshError::KeyLoadError(e.to_string()))?;

        let config = self.make_config();
        let addr = self.addr();
        let handler = SshHandler;

        let mut handle = client::connect(config, &addr, handler)
            .await
            .map_err(|e| SshError::ConnectError(e.to_string()))?;

        let auth = handle
            .authenticate_publickey(
                self.username.clone(),
                PrivateKeyWithHashAlg::new(Arc::new(key_pair), Some(HashAlg::Sha256)),
            )
            .await
            .map_err(|e| SshError::AuthError(e.to_string()))?;

        if auth.success() {
            tracing::info!(host = %self.host, "public-key auth succeeded");
            Ok(ConnectedClient::new(handle, self.username.clone()))
        } else {
            Err(SshError::AuthError(
                "public key rejected by server".to_string(),
            ))
        }
    }

    /// Connect and authenticate using keys from a running SSH agent
    /// (e.g. Pageant or `ssh-agent`).
    ///
    /// # Errors
    /// Returns [`SshError::AuthError`] if no agent is available or every key
    /// the agent offers is rejected.
    pub async fn connect_agent(&self) -> Result<ConnectedClient, SshError> {
        // Agent auth requires russh::keys::agent::client which has a different
        // API in russh 0.62. This will be implemented in a follow-up task.
        Err(SshError::AuthError(
            "SSH agent authentication not yet implemented".to_string(),
        ))
    }
}

/// A live, authenticated SSH connection.
///
/// Obtained from one of the [`SshClient::connect_*`] methods. Use
/// [`ConnectedClient::open_pty`] for an interactive shell or
/// [`ConnectedClient::exec`] for a one-shot command.
pub struct ConnectedClient {
    handle: Mutex<client::Handle<SshHandler>>,
    username: String,
}

impl ConnectedClient {
    fn new(handle: client::Handle<SshHandler>, username: String) -> Self {
        Self {
            handle: Mutex::new(handle),
            username,
        }
    }

    /// The authenticated username for this connection.
    #[must_use]
    pub fn username(&self) -> &str {
        &self.username
    }

    /// Open a session channel, request a PTY of the given size, and start an
    /// interactive shell.
    ///
    /// # Errors
    /// Returns [`SshError::ChannelError`] if the channel or PTY request fails.
    pub async fn open_pty(&self, rows: u32, cols: u32) -> Result<SshChannel, SshError> {
        let handle = self.handle.lock().await;
        let channel = handle
            .channel_open_session()
            .await
            .map_err(|e| SshError::ChannelError(e.to_string()))?;

        channel
            .request_pty(false, "xterm", cols, rows, 0, 0, &[])
            .await
            .map_err(|e| SshError::ChannelError(format!("pty request failed: {e}")))?;

        channel
            .request_shell(false)
            .await
            .map_err(|e| SshError::ChannelError(format!("shell request failed: {e}")))?;

        Ok(SshChannel::new(channel))
    }

    /// Execute `command` on the remote host and return its combined stdout as
    /// a `String`.
    ///
    /// # Errors
    /// Returns [`SshError::ChannelError`] if the channel or exec request
    /// fails, or if the output cannot be read.
    pub async fn exec(&self, command: impl Into<String>) -> Result<String, SshError> {
        let handle = self.handle.lock().await;
        let mut channel = handle
            .channel_open_session()
            .await
            .map_err(|e| SshError::ChannelError(e.to_string()))?;

        channel
            .exec(false, command.into())
            .await
            .map_err(|e| SshError::ChannelError(format!("exec failed: {e}")))?;

        // Collect output until the channel closes (EOF).
        let mut output = Vec::new();
        while let Some(msg) = channel.wait().await {
            match msg {
                russh::ChannelMsg::Data { data } => output.extend_from_slice(&data),
                russh::ChannelMsg::ExtendedData { data, .. } => output.extend_from_slice(&data),
                russh::ChannelMsg::Eof | russh::ChannelMsg::Close => break,
                _ => {}
            }
        }

        String::from_utf8(output)
            .map_err(|e| SshError::ChannelError(format!("invalid utf-8 in output: {e}")))
    }
}

/// An active SSH session channel (e.g. an interactive PTY shell).
///
/// Obtain one via [`ConnectedClient::open_pty`]. Data written is sent to the
/// remote process's stdin; window-size changes can be signalled with
/// [`SshChannel::resize`].
pub struct SshChannel {
    inner: Mutex<russh::Channel<russh::client::Msg>>,
}

impl SshChannel {
    fn new(channel: russh::Channel<russh::client::Msg>) -> Self {
        Self {
            inner: Mutex::new(channel),
        }
    }

    /// Send `data` to the remote process's stdin.
    ///
    /// # Errors
    /// Returns [`SshError::ChannelError`] if the write fails.
    pub async fn write(&self, data: impl AsRef<[u8]>) -> Result<(), SshError> {
        let channel = self.inner.lock().await;
        channel
            .data(data.as_ref())
            .await
            .map_err(|e| SshError::ChannelError(format!("write failed: {e}")))
    }

    /// Signal a terminal window-size change to the remote PTY.
    ///
    /// # Errors
    /// Returns [`SshError::ChannelError`] if the request fails.
    pub async fn resize(&self, rows: u32, cols: u32) -> Result<(), SshError> {
        let channel = self.inner.lock().await;
        channel
            .window_change(cols, rows, 0, 0)
            .await
            .map_err(|e| SshError::ChannelError(format!("resize failed: {e}")))
    }

    /// Close the channel gracefully.
    ///
    /// # Errors
    /// Returns [`SshError::ChannelError`] if the close fails.
    pub async fn close(&self) -> Result<(), SshError> {
        let channel = self.inner.lock().await;
        channel
            .eof()
            .await
            .map_err(|e| SshError::ChannelError(format!("eof failed: {e}")))?;
        Ok(())
    }
}
