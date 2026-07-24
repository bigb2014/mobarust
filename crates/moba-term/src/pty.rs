//! PTY shell manager.
//!
//! Spawns and controls a pseudo-terminal child process using
//! [`portable_pty`]. The [`PtySession`] struct owns the master end of the
//! PTY and the child handle, providing read/write/resize/kill operations
//! suitable for driving an interactive terminal emulator.

use std::io::{Read, Write};
use std::sync::Mutex;

use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize};
use thiserror::Error;

/// Errors that can occur while managing a PTY session.
#[allow(clippy::enum_variant_names)]
#[derive(Debug, Error)]
pub enum TermError {
    /// Wraps an [`io::Error`] from the underlying PTY transport.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// A PTY-specific error (open, resize, etc.).
    #[error("pty error: {0}")]
    Pty(String),

    /// The child process could not be spawned.
    #[error("spawn error: {0}")]
    Spawn(String),
}

/// Manages a PTY child process.
///
/// Created with [`PtySession::new`] (default shell) or
/// [`PtySession::new_with_command`] (custom command). The session owns the
/// master PTY and the child handle, and can be used to read/write data,
/// resize the terminal, and control the child's lifecycle.
pub struct PtySession {
    /// The master end of the PTY.
    master: Box<dyn MasterPty + Send>,

    /// Readable handle cloned from the master.
    reader: Box<dyn Read + Send>,

    /// Writable handle taken from the master.
    writer: Box<dyn Write + Send>,

    /// The spawned child process, wrapped in a [`Mutex`] so that
    /// [`PtySession::is_alive`] can perform a non-blocking poll without
    /// needing `&mut self`.
    child: Mutex<Box<dyn Child + Send + Sync>>,

    /// Current number of rows.
    rows: usize,

    /// Current number of columns.
    cols: usize,
}

/// Builds a [`PtySize`] with zero pixel dimensions.
fn pty_size(rows: usize, cols: usize) -> PtySize {
    PtySize {
        rows: rows as u16,
        cols: cols as u16,
        pixel_width: 0,
        pixel_height: 0,
    }
}

impl PtySession {
    /// Spawns the platform-default shell at the given size.
    ///
    /// On Unix this is typically the user's login shell as determined by
    /// portable-pty's default-prog logic.
    ///
    /// # Errors
    ///
    /// Returns [`TermError::Pty`] if the PTY cannot be opened, or
    /// [`TermError::Spawn`] if the child cannot be spawned.
    pub fn new(rows: usize, cols: usize) -> Result<Self, TermError> {
        Self::new_with_command(rows, cols, "", &[])
    }

    /// Spawns `cmd` with `args` at the given PTY size.
    ///
    /// If `cmd` is empty, the platform default program is used.
    ///
    /// # Errors
    ///
    /// Returns [`TermError::Pty`] if the PTY cannot be opened, or
    /// [`TermError::Spawn`] if the child cannot be spawned.
    pub fn new_with_command(
        rows: usize,
        cols: usize,
        cmd: &str,
        args: &[&str],
    ) -> Result<Self, TermError> {
        tracing::debug!(rows, cols, cmd, args = ?args, "spawning pty session");

        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(pty_size(rows, cols))
            .map_err(|e| TermError::Pty(format!("open_pty: {e}")))?;

        let mut builder = if cmd.is_empty() {
            CommandBuilder::new_default_prog()
        } else {
            CommandBuilder::new(cmd)
        };
        for arg in args {
            builder.arg(arg);
        }

        let child = pair
            .slave
            .spawn_command(builder)
            .map_err(|e| TermError::Spawn(format!("spawn_command: {e}")))?;

        let reader = pair
            .master
            .try_clone_reader()
            .map_err(|e| TermError::Pty(format!("try_clone_reader: {e}")))?;

        let writer = pair
            .master
            .take_writer()
            .map_err(|e| TermError::Pty(format!("take_writer: {e}")))?;

        // Drop the slave so that EOF is delivered to the master when the
        // child exits.
        drop(pair.slave);

        Ok(PtySession {
            master: pair.master,
            reader,
            writer,
            child: Mutex::new(child),
            rows,
            cols,
        })
    }

    /// Reads bytes from the PTY master into `buf`.
    ///
    /// Returns the number of bytes read (0 indicates EOF).
    ///
    /// # Errors
    ///
    /// Returns [`TermError::Io`] on read failure.
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, TermError> {
        let n = self.reader.read(buf)?;
        tracing::trace!(bytes = n, "pty read");
        Ok(n)
    }

    /// Takes ownership of the reader handle so it can be moved to a
    /// background thread. After calling this, `read()` will return an error.
    pub fn take_reader(&mut self) -> Option<Box<dyn Read + Send>> {
        std::mem::replace(&mut self.reader, Box::new(std::io::empty())).into()
    }

    /// Writes `data` to the PTY master.
    ///
    /// Returns the number of bytes written.
    ///
    /// # Errors
    ///
    /// Returns [`TermError::Io`] on write failure.
    pub fn write(&mut self, data: &[u8]) -> Result<usize, TermError> {
        let n = self.writer.write(data)?;
        tracing::trace!(bytes = n, "pty write");
        Ok(n)
    }

    /// Resizes the PTY to `rows` x `cols`.
    ///
    /// # Errors
    ///
    /// Returns [`TermError::Pty`] if the PTY cannot be resized.
    pub fn resize(&mut self, rows: usize, cols: usize) -> Result<(), TermError> {
        self.master
            .resize(pty_size(rows, cols))
            .map_err(|e| TermError::Pty(format!("resize: {e}")))?;
        self.rows = rows;
        self.cols = cols;
        tracing::debug!(rows, cols, "pty resized");
        Ok(())
    }

    /// Returns `true` if the child process is still running.
    ///
    /// This performs a non-blocking check via [`Child::try_wait`]. The child
    /// is *not* reaped -- [`PtySession::wait`] must still be called to
    /// collect the final exit status.
    pub fn is_alive(&self) -> bool {
        let mut child = match self.child.lock() {
            Ok(guard) => guard,
            Err(_) => return false,
        };
        match child.try_wait() {
            Ok(Some(_)) => false,
            Ok(None) => true,
            Err(_) => false,
        }
    }

    /// Kills the child process.
    ///
    /// Sends a termination signal to the child. To reap the child and obtain
    /// its exit code, call [`PtySession::wait`] afterwards.
    ///
    /// # Errors
    ///
    /// Returns [`TermError::Pty`] if the child cannot be killed.
    pub fn kill(&mut self) -> Result<(), TermError> {
        let mut child = self
            .child
            .lock()
            .map_err(|e| TermError::Pty(format!("lock: {e}")))?;
        child
            .kill()
            .map_err(|e| TermError::Pty(format!("kill: {e}")))?;
        tracing::debug!("pty child killed");
        Ok(())
    }

    /// Waits for the child to exit and returns its exit code.
    ///
    /// Blocks until the child terminates.
    ///
    /// # Errors
    ///
    /// Returns [`TermError::Pty`] if waiting fails.
    pub fn wait(&mut self) -> Result<i32, TermError> {
        let mut child = self
            .child
            .lock()
            .map_err(|e| TermError::Pty(format!("lock: {e}")))?;
        let status = child
            .wait()
            .map_err(|e| TermError::Pty(format!("wait: {e}")))?;
        let code = status.exit_code() as i32;
        tracing::debug!(exit_code = code, "pty child exited");
        Ok(code)
    }

    /// Returns the current number of rows.
    #[allow(dead_code)]
    pub fn rows(&self) -> usize {
        self.rows
    }

    /// Returns the current number of columns.
    #[allow(dead_code)]
    pub fn cols(&self) -> usize {
        self.cols
    }
}
