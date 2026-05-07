use anyhow::{Context, Result};
use std::process::{Command, Stdio};

fn run_apt(args: &[&str], sudo: bool) -> Result<()> {
    let mut cmd = if sudo {
        let mut c = Command::new("sudo");
        c.arg("apt");
        c
    } else {
        Command::new("apt")
    };

    cmd.args(args);
    cmd.stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    let status = cmd.status().with_context(|| "Failed to execute apt")?;

    if !status.success() {
        anyhow::bail!("apt command failed with status: {}", status);
    }

    Ok(())
}

fn run_apt_with_vec(args: Vec<String>, sudo: bool) -> Result<()> {
    let mut cmd = if sudo {
        let mut c = Command::new("sudo");
        c.arg("apt");
        c
    } else {
        Command::new("apt")
    };

    cmd.args(&args);
    cmd.stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    let status = cmd.status().with_context(|| "Failed to execute apt")?;

    if !status.success() {
        anyhow::bail!("apt command failed with status: {}", status);
    }

    Ok(())
}

pub fn update() -> Result<()> {
    println!(":: Updating package lists...");
    run_apt(&["update"], true)
}

pub fn upgrade() -> Result<()> {
    println!(":: Upgrading packages...");
    run_apt(&["upgrade"], true)
}

pub fn full_upgrade() -> Result<()> {
    println!(":: Performing full upgrade...");
    run_apt(&["full-upgrade"], true)
}

pub fn install(packages: &[String]) -> Result<()> {
    if packages.is_empty() {
        anyhow::bail!("No packages specified for installation");
    }
    println!(":: Installing packages: {}", packages.join(" "));
    let mut args = vec!["install".to_string()];
    args.extend(packages.iter().cloned());
    run_apt_with_vec(args, true)
}

pub fn remove(packages: &[String], purge: bool) -> Result<()> {
    if packages.is_empty() {
        anyhow::bail!("No packages specified for removal");
    }
    let action = if purge { "purge" } else { "remove" };
    println!(":: Removing packages: {}", packages.join(" "));
    let mut args = vec![action.to_string()];
    args.extend(packages.iter().cloned());
    run_apt_with_vec(args, true)
}

pub fn autoremove() -> Result<()> {
    println!(":: Removing unused dependencies...");
    run_apt(&["autoremove"], true)
}

pub fn search(query: &str) -> Result<()> {
    run_apt(&["search", query], false)
}

pub fn show(package: &str) -> Result<()> {
    run_apt(&["show", package], false)
}

pub fn clean() -> Result<()> {
    println!(":: Cleaning package cache...");
    run_apt(&["clean"], true)
}

pub fn list_installed(filter: Option<&str>) -> Result<()> {
    let output = Command::new("dpkg-query")
        .args(&["-W", "-f=${Package}\t${Version}\n"])
        .output()
        .with_context(|| "Failed to list installed packages")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        anyhow::bail!("dpkg-query failed: {}", stderr);
    }

    for line in stdout.lines() {
        if let Some(f) = filter {
            if line.to_lowercase().contains(&f.to_lowercase()) {
                println!("{}", line);
            }
        } else {
            println!("{}", line);
        }
    }

    Ok(())
}

pub fn list_upgrades() -> Result<()> {
    println!(":: Checking for available upgrades...");
    let output = Command::new("apt")
        .args(&["list", "--upgradable"])
        .output()
        .with_context(|| "Failed to check upgrades")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        anyhow::bail!("apt list failed: {}", stderr);
    }

    println!("{}", stdout);
    Ok(())
}

pub fn edit_sources() -> Result<()> {
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());
    let mut cmd = Command::new("sudo");
    cmd.arg(&editor)
        .arg("/etc/apt/sources.list")
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    let status = cmd.status().with_context(|| "Failed to launch editor")?;

    if !status.success() {
        anyhow::bail!("Editor exited with status: {}", status);
    }

    Ok(())
}
