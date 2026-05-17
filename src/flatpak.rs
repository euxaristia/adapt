use anyhow::Result;
use std::process::Command;

pub fn update(noconfirm: bool) -> Result<()> {
    // Check if flatpak is installed
    let has_flatpak = Command::new("sh")
        .args(["-c", "command -v flatpak"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if !has_flatpak {
        return Ok(());
    }

    println!(":: Updating flatpak packages...");
    let mut cmd = Command::new("flatpak");
    cmd.args(["update"]);
    if noconfirm {
        cmd.arg("-y");
    }
    cmd.stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit());

    let status = cmd.status()?;
    if !status.success() {
        anyhow::bail!("flatpak update failed with status: {}", status);
    }

    Ok(())
}
