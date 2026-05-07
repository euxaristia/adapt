use anyhow::{Context, Result};
use std::ffi::OsStr;
use std::process::{Command, Stdio};

fn is_root() -> bool {
    unsafe { libc::getuid() == 0 }
}

fn run_apt<I, S>(args: I, use_sudo: bool, noconfirm: bool) -> Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut cmd = if use_sudo && !is_root() {
        let mut c = Command::new("sudo");
        c.arg("apt");
        c
    } else {
        Command::new("apt")
    };

    if noconfirm {
        cmd.arg("-y");
    }

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

pub fn update(noconfirm: bool) -> Result<()> {
    println!(":: Updating package lists...");
    run_apt(&["update"], true, noconfirm)
}

pub fn upgrade(noconfirm: bool) -> Result<()> {
    println!(":: Upgrading packages...");
    run_apt(&["upgrade"], true, noconfirm)
}

pub fn full_upgrade(noconfirm: bool) -> Result<()> {
    println!(":: Performing full upgrade...");
    run_apt(&["full-upgrade"], true, noconfirm)
}

pub fn install(packages: &[String], noconfirm: bool) -> Result<()> {
    if packages.is_empty() {
        anyhow::bail!("No packages specified for installation");
    }
    println!(":: Installing packages: {}", packages.join(" "));
    let mut args = vec!["install".to_string(), "--".to_string()];
    args.extend(packages.iter().cloned());
    run_apt(args, true, noconfirm)
}

pub fn remove(packages: &[String], purge: bool, noconfirm: bool) -> Result<()> {
    if packages.is_empty() {
        anyhow::bail!("No packages specified for removal");
    }
    let action = if purge { "purge" } else { "remove" };
    println!(":: Removing packages: {}", packages.join(" "));
    let mut args = vec![action.to_string(), "--".to_string()];
    args.extend(packages.iter().cloned());
    run_apt(args, true, noconfirm)
}

pub fn autoremove(noconfirm: bool) -> Result<()> {
    println!(":: Removing unused dependencies...");
    run_apt(&["autoremove"], true, noconfirm)
}

pub fn search(query: &str, _noconfirm: bool) -> Result<()> {
    run_apt(&["search", query], false, false)
}

pub fn show(package: &str, _noconfirm: bool) -> Result<()> {
    run_apt(&["show", package], false, false)
}

pub fn clean(_noconfirm: bool) -> Result<()> {
    println!(":: Cleaning package cache...");
    run_apt(&["clean"], true, false)
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

pub fn list_upgrades(_noconfirm: bool) -> Result<()> {
    println!(":: Checking for available upgrades...");
    run_apt(&["list", "--upgradable"], false, false)
}

pub fn edit_sources() -> Result<()> {
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());
    let shell_cmd = format!("{} /etc/apt/sources.list", editor);

    let mut cmd = if is_root() {
        Command::new("sh")
    } else {
        let mut c = Command::new("sudo");
        c.arg("sh");
        c
    };

    cmd.arg("-c")
        .arg(&shell_cmd)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    let status = cmd.status().with_context(|| "Failed to launch editor")?;

    if !status.success() {
        anyhow::bail!("Editor exited with status: {}", status);
    }

    Ok(())
}
