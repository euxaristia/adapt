mod apt;
mod cli;

use anyhow::Result;
use cli::{Cli, Command, PacmanFlags};

fn main() -> Result<()> {
    let cli = Cli::parse_args();

    match cli.command {
        Some(cmd) => handle_subcommand(cmd, cli.noconfirm)?,
        None => handle_flags(&cli.flags, cli.noconfirm)?,
    }

    Ok(())
}

fn handle_subcommand(cmd: Command, noconfirm: bool) -> Result<()> {
    match cmd {
        Command::Update => apt::update(noconfirm)?,
        Command::Upgrade { full } => {
            if full {
                apt::full_upgrade(noconfirm)?;
            } else {
                apt::upgrade(noconfirm)?;
            }
        }
        Command::FullUpgrade => apt::full_upgrade(noconfirm)?,
        Command::Install { packages } => apt::install(&packages, noconfirm)?,
        Command::Remove { packages } => apt::remove(&packages, false, noconfirm)?,
        Command::Purge { packages } => apt::remove(&packages, true, noconfirm)?,
        Command::Search { query } => apt::search(&query, noconfirm)?,
        Command::Show { package } => apt::show(&package, noconfirm)?,
        Command::Autoremove => apt::autoremove(noconfirm)?,
        Command::Clean => apt::clean(noconfirm)?,
        Command::List { filter } => apt::list_installed(filter.as_deref())?,
        Command::EditSources => apt::edit_sources()?,
    }
    Ok(())
}

fn handle_flags(flags: &PacmanFlags, noconfirm: bool) -> Result<()> {
    let no_operation = !flags.sync
        && !flags.remove
        && !flags.query
        && !flags.search
        && !flags.info
        && !flags.clean
        && !flags.refresh
        && !flags.upgrade;

    if no_operation && flags.targets.is_empty() {
        println!(":: adapter - AUR helper style wrapper for apt");
        println!(":: No operation specified, running update && upgrade...\n");
        apt::update(noconfirm)?;
        apt::upgrade(noconfirm)?;
        return Ok(());
    }

    if flags.sync {
        if flags.search && !flags.targets.is_empty() {
            apt::search(&flags.targets.join(" "), noconfirm)?;
        } else if flags.info && !flags.targets.is_empty() {
            apt::show(&flags.targets[0], noconfirm)?;
        } else if flags.clean {
            apt::clean(noconfirm)?;
        } else if flags.refresh && flags.upgrade && flags.targets.is_empty() {
            apt::update(noconfirm)?;
            apt::upgrade(noconfirm)?;
        } else if flags.refresh && flags.targets.is_empty() {
            apt::update(noconfirm)?;
        } else if flags.upgrade && flags.targets.is_empty() {
            apt::upgrade(noconfirm)?;
        } else if !flags.targets.is_empty() {
            if flags.refresh {
                apt::update(noconfirm)?;
            }
            if flags.upgrade {
                apt::upgrade(noconfirm)?;
            }
            apt::install(&flags.targets, noconfirm)?;
        } else if flags.refresh {
            apt::update(noconfirm)?;
        } else if flags.upgrade {
            apt::upgrade(noconfirm)?;
        } else {
            println!(":: Sync operation. Use -Syu to update & upgrade, -S <pkg> to install, -Ss <query> to search.");
        }
    } else if flags.remove {
        if flags.targets.is_empty() {
            anyhow::bail!("No packages specified for removal. Usage: adapter -R <package>");
        }
        apt::remove(&flags.targets, false, noconfirm)?;
        if flags.recursive {
            apt::autoremove(noconfirm)?;
        }
    } else if flags.query {
        if flags.info && !flags.targets.is_empty() {
            apt::show(&flags.targets[0], noconfirm)?;
        } else if flags.search && !flags.targets.is_empty() {
            apt::list_installed(Some(&flags.targets.join(" ")))?;
        } else if flags.upgrade {
            apt::list_upgrades(noconfirm)?;
        } else if !flags.targets.is_empty() {
            apt::list_installed(Some(&flags.targets.join(" ")))?;
        } else {
            apt::list_installed(None)?;
        }
    } else if flags.search {
        if flags.targets.is_empty() {
            anyhow::bail!("No search query specified. Usage: adapter -Ss <query>");
        }
        apt::search(&flags.targets.join(" "), noconfirm)?;
    } else if flags.info {
        if flags.targets.is_empty() {
            anyhow::bail!("No package specified. Usage: adapter -Si <package>");
        }
        apt::show(&flags.targets[0], noconfirm)?;
    } else if flags.clean {
        apt::clean(noconfirm)?;
    } else {
        println!(":: adapter - AUR helper style wrapper for apt");
        println!(":: Try 'adapter --help' for usage information.");
    }

    Ok(())
}
