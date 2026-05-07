mod apt;
mod cli;

use anyhow::Result;
use cli::{Cli, Command, PacmanFlags};

fn main() -> Result<()> {
    let cli = Cli::parse_args();

    match cli.command {
        Some(cmd) => handle_subcommand(cmd)?,
        None => handle_flags(&cli.flags)?,
    }

    Ok(())
}

fn handle_subcommand(cmd: Command) -> Result<()> {
    match cmd {
        Command::Update => apt::update()?,
        Command::Upgrade { full } => {
            if full {
                apt::full_upgrade()?;
            } else {
                apt::upgrade()?;
            }
        }
        Command::FullUpgrade => apt::full_upgrade()?,
        Command::Install { packages } => apt::install(&packages)?,
        Command::Remove { packages } => apt::remove(&packages, false)?,
        Command::Purge { packages } => apt::remove(&packages, true)?,
        Command::Search { query } => apt::search(&query)?,
        Command::Show { package } => apt::show(&package)?,
        Command::Autoremove => apt::autoremove()?,
        Command::Clean => apt::clean()?,
        Command::List { filter } => apt::list_installed(filter.as_deref())?,
        Command::EditSources => apt::edit_sources()?,
    }
    Ok(())
}

fn handle_flags(flags: &PacmanFlags) -> Result<()> {
    // Default behavior: no flags -> update && upgrade (like paru with no args)
    let no_operation = !flags.sync && !flags.remove && !flags.query && !flags.search && !flags.info && !flags.clean;
    
    if no_operation && flags.targets.is_empty() {
        println!(":: adapter - AUR helper style wrapper for apt");
        println!(":: No operation specified, running update && upgrade...\n");
        apt::update()?;
        apt::upgrade()?;
        return Ok(());
    }

    if flags.sync {
        // -S operations
        if flags.search && !flags.targets.is_empty() {
            // -Ss <query>
            apt::search(&flags.targets.join(" "))?;
        } else if flags.info && !flags.targets.is_empty() {
            // -Si <pkg>
            apt::show(&flags.targets[0])?;
        } else if flags.clean {
            // -Sc
            apt::clean()?;
        } else if flags.refresh && flags.upgrade && flags.targets.is_empty() {
            // -Syu
            apt::update()?;
            apt::upgrade()?;
        } else if flags.refresh && flags.targets.is_empty() {
            // -Sy
            apt::update()?;
        } else if flags.upgrade && flags.targets.is_empty() {
            // -Su
            apt::upgrade()?;
        } else if !flags.targets.is_empty() {
            // -S <pkg>
            apt::install(&flags.targets)?;
        } else if flags.refresh {
            apt::update()?;
        } else if flags.upgrade {
            apt::upgrade()?;
        } else {
            println!(":: Sync operation. Use -Syu to update & upgrade, -S <pkg> to install, -Ss <query> to search.");
        }
    } else if flags.remove {
        if flags.targets.is_empty() {
            anyhow::bail!("No packages specified for removal. Usage: adapter -R <package>");
        }
        apt::remove(&flags.targets, false)?;
        if flags.recursive {
            apt::autoremove()?;
        }
    } else if flags.query {
        if flags.search && !flags.targets.is_empty() {
            // -Qs <query>
            apt::list_installed(Some(&flags.targets.join(" ")))?;
        } else if flags.upgrade {
            // -Qu
            apt::list_upgrades()?;
        } else if !flags.targets.is_empty() {
            // -Q <pkg> or just show installed
            apt::list_installed(Some(&flags.targets.join(" ")))?;
        } else {
            // -Q
            apt::list_installed(None)?;
        }
    } else if flags.search {
        if flags.targets.is_empty() {
            anyhow::bail!("No search query specified. Usage: adapter -Ss <query>");
        }
        // -Ss <query>
        apt::search(&flags.targets.join(" "))?;
    } else if flags.info {
        if flags.targets.is_empty() {
            anyhow::bail!("No package specified. Usage: adapter -Si <package>");
        }
        // -Si <pkg>
        apt::show(&flags.targets[0])?;
    } else if flags.clean {
        apt::clean()?;
    } else {
        println!(":: adapter - AUR helper style wrapper for apt");
        println!(":: Try 'adapter --help' for usage information.");
    }

    Ok(())
}
