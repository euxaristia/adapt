use anyhow::{Context, Result};
use std::process::{Command, Stdio};

pub fn update(noconfirm: bool) -> Result<()> {
    let has_flatpak = Command::new("flatpak")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
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
    cmd.stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    let status = cmd
        .status()
        .with_context(|| "Failed to execute flatpak update")?;
    if !status.success() {
        anyhow::bail!("flatpak update failed with status: {}", status);
    }

    Ok(())
}
