// builder/src/subprocess.rs
//
// Platform-safe cargo subprocess management.
// Uses background thread + crossbeam-channel for non-blocking output reads.
// Termination via child.kill() — no SIGTERM, no platform-specific signal APIs.

use std::io::{BufRead, BufReader};
use std::process::{Child, ChildStdout, ChildStderr, Command, ExitStatus, Stdio};
use std::thread;
use crossbeam_channel::{bounded, Receiver};

/// Live output line from cargo stdout or stderr.
#[derive(Debug, Clone)]
pub enum OutputLine {
    Stdout(String),
    Stderr(String),
}

/// Handle to a running cargo subprocess.
/// Spawned via `BuildProcess::spawn`. Terminated via `kill()`.
pub struct BuildProcess {
    child: Child,
    output_rx: Receiver<OutputLine>,
    finished: bool,
    exit_status: Option<ExitStatus>,
}

impl BuildProcess {
    /// Spawn a new cargo subprocess from the given `Command`.
    /// The command must not have stdout/stderr configured — this method sets them.
    pub fn spawn(cmd: &mut Command) -> Result<BuildProcess, std::io::Error> {
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

        let mut child = cmd.spawn()?;

        let stdout: ChildStdout = child.stdout.take().expect("stdout missing after piped spawn");
        let stderr: ChildStderr = child.stderr.take().expect("stderr missing after piped spawn");

        let (tx, rx) = bounded::<OutputLine>(4096);

        // Stdout reader thread
        {
            let tx_out = tx.clone();
            thread::spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines() {
                    match line {
                        Ok(l) => {
                            if tx_out.send(OutputLine::Stdout(l)).is_err() {
                                break;
                            }
                        }
                        Err(_) => break,
                    }
                }
            });
        }

        // Stderr reader thread
        {
            let tx_err = tx;
            thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines() {
                    match line {
                        Ok(l) => {
                            if tx_err.send(OutputLine::Stderr(l)).is_err() {
                                break;
                            }
                        }
                        Err(_) => break,
                    }
                }
            });
        }

        Ok(BuildProcess {
            child,
            output_rx: rx,
            finished: false,
            exit_status: None,
        })
    }

    /// Drain all pending output lines without blocking.
    /// Returns whatever lines the background threads have produced since last call.
    pub fn poll_output(&mut self) -> Vec<OutputLine> {
        let mut lines = Vec::new();
        loop {
            match self.output_rx.try_recv() {
                Ok(line) => lines.push(line),
                Err(_) => break,
            }
        }
        // Update finished state without blocking.
        if !self.finished {
            if let Ok(status) = self.child.try_wait() {
                if let Some(s) = status {
                    self.finished = true;
                    self.exit_status = Some(s);
                }
            }
        }
        lines
    }

    /// Terminate the subprocess immediately.
    /// Uses `Child::kill()` — works on both Unix and Windows.
    /// Does NOT send SIGTERM; Windows has no concept of SIGTERM.
    pub fn kill(&mut self) -> Result<(), std::io::Error> {
        self.finished = true;
        self.child.kill()
    }

    /// Returns `true` while the cargo process has not yet exited.
    pub fn is_running(&mut self) -> bool {
        if self.finished {
            return false;
        }
        match self.child.try_wait() {
            Ok(Some(status)) => {
                self.finished = true;
                self.exit_status = Some(status);
                false
            }
            Ok(None) => true,
            Err(_) => {
                self.finished = true;
                false
            }
        }
    }

    /// Returns the exit status once the process has finished, or `None` if still running.
    pub fn exit_status(&self) -> Option<ExitStatus> {
        self.exit_status
    }
}
