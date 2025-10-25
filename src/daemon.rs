use anyhow::Result;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use tracing::{debug, error, info};

/// Path to the Unix socket for daemon communication
pub fn socket_path() -> PathBuf {
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(runtime_dir).join("native-launcher.sock")
}

/// Check if daemon is running by attempting to connect to socket
pub fn is_daemon_running() -> bool {
    let sock_path = socket_path();
    sock_path.exists() && UnixStream::connect(&sock_path).is_ok()
}

/// Send signal to running daemon to show the launcher window
pub fn send_show_signal() -> Result<()> {
    let sock_path = socket_path();

    if !sock_path.exists() {
        anyhow::bail!("Daemon socket not found at {:?}", sock_path);
    }

    debug!("Connecting to daemon socket at {:?}", sock_path);
    let mut stream = UnixStream::connect(&sock_path)?;

    // Send "show" command
    use std::io::Write;
    stream.write_all(b"show\n")?;
    stream.flush()?;

    info!("Sent show signal to daemon");
    Ok(())
}

/// Start Unix socket listener for daemon mode
/// Returns a receiver channel that gets notified when show signal arrives
pub fn start_socket_listener() -> Result<std::sync::mpsc::Receiver<String>> {
    let sock_path = socket_path();

    // Remove old socket if it exists
    if sock_path.exists() {
        info!("Removing old socket at {:?}", sock_path);
        std::fs::remove_file(&sock_path)?;
    }

    info!("Creating daemon socket at {:?}", sock_path);
    let listener = UnixListener::bind(&sock_path)?;

    // Set permissions so only the user can connect
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&sock_path)?.permissions();
        perms.set_mode(0o600); // rw-------
        std::fs::set_permissions(&sock_path, perms)?;
    }

    let (tx, rx) = std::sync::mpsc::channel();

    // Spawn listener thread
    std::thread::spawn(move || {
        info!("Daemon socket listener started");

        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    debug!("Received connection on daemon socket");

                    // Read command from client
                    use std::io::Read;
                    let mut buffer = [0u8; 1024];

                    match stream.read(&mut buffer) {
                        Ok(n) => {
                            let command = String::from_utf8_lossy(&buffer[..n]).trim().to_string();
                            debug!("Daemon received command: {}", command);

                            // Send command to main thread via channel
                            if let Err(e) = tx.send(command) {
                                error!("Failed to send command to main thread: {}", e);
                            }
                        }
                        Err(e) => {
                            error!("Failed to read from socket: {}", e);
                        }
                    }
                }
                Err(e) => {
                    error!("Socket connection error: {}", e);
                }
            }
        }

        info!("Daemon socket listener stopped");
    });

    Ok(rx)
}

/// Cleanup daemon socket on exit
pub fn cleanup_socket() {
    let sock_path = socket_path();
    if sock_path.exists() {
        debug!("Cleaning up daemon socket at {:?}", sock_path);
        let _ = std::fs::remove_file(&sock_path);
    }
}
