use std::path::PathBuf;
use tokio::process::Command;

/// Path to the pre-compiled cmuxd-remote binary.
/// Looks in: ~/.local/share/cmux/bin/cmuxd-remote-linux-amd64
pub fn local_daemon_path() -> PathBuf {
    let data_dir = std::env::var("XDG_DATA_HOME")
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
            format!("{home}/.local/share")
        });
    // Prefer the per-user build (scripts/install-cmuxd-remote.sh).
    let xdg = PathBuf::from(data_dir).join("cmux/bin/cmuxd-remote-linux-amd64");
    if xdg.exists() {
        return xdg;
    }
    // Fall back to the packaged location: the .deb installs the daemon here.
    let packaged = PathBuf::from("/usr/lib/cmux/cmuxd-remote");
    if packaged.exists() {
        return packaged;
    }
    // Neither present — return the XDG path so the "not found" error is actionable.
    xdg
}

/// Deploy cmuxd-remote to remote host via scp.
/// Copies to ~/.local/bin/cmuxd-remote on the remote.
pub async fn deploy_remote(target: &str) -> Result<(), String> {
    let local_path = local_daemon_path();
    if !local_path.exists() {
        return Err(format!(
            "cmuxd-remote binary not found at {}. Run: ./scripts/install-cmuxd-remote.sh",
            local_path.display()
        ));
    }

    // Ensure remote directory exists
    let mkdir_status = Command::new("ssh")
        .args([target, "mkdir", "-p", "~/.local/bin"])
        .status()
        .await
        .map_err(|e| format!("SSH mkdir failed: {e}"))?;
    if !mkdir_status.success() {
        return Err("Failed to create remote directory".to_string());
    }

    // scp the binary
    let remote_dest = format!("{target}:~/.local/bin/cmuxd-remote");
    let scp_status = Command::new("scp")
        .args([local_path.to_str().unwrap(), &remote_dest])
        .status()
        .await
        .map_err(|e| format!("scp failed: {e}"))?;
    if !scp_status.success() {
        return Err(format!("Failed to deploy remote daemon to {target}"));
    }

    // Make executable
    let chmod_status = Command::new("ssh")
        .args([target, "chmod", "+x", "~/.local/bin/cmuxd-remote"])
        .status()
        .await
        .map_err(|e| format!("SSH chmod failed: {e}"))?;
    if !chmod_status.success() {
        return Err("Failed to set executable permissions".to_string());
    }

    Ok(())
}
