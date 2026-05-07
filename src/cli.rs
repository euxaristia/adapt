use clap::{Parser, Subcommand, Args};

#[derive(Parser, Debug)]
#[command(name = "adapt", about = "paru, but for apt", version, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    #[command(flatten)]
    pub flags: PacmanFlags,

    /// Assume yes for all apt prompts (pass -y to apt)
    #[arg(long, global = true)]
    pub noconfirm: bool,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    /// Update package lists
    Update,
    /// Upgrade packages
    Upgrade {
        /// Perform a full upgrade (dist-upgrade)
        #[arg(short = 'f', long)]
        full: bool,
    },
    /// Full upgrade (dist-upgrade)
    #[command(name = "full-upgrade")]
    FullUpgrade,
    /// Install packages
    Install {
        /// Package(s) to install
        #[arg(required = true)]
        packages: Vec<String>,
    },
    /// Remove packages
    Remove {
        /// Package(s) to remove
        #[arg(required = true)]
        packages: Vec<String>,
    },
    /// Purge packages (remove with config files)
    Purge {
        /// Package(s) to purge
        #[arg(required = true)]
        packages: Vec<String>,
    },
    /// Search for packages
    Search {
        /// Search query
        query: String,
    },
    /// Show package details
    Show {
        /// Package name
        package: String,
    },
    /// Remove automatically installed packages
    Autoremove,
    /// Clean downloaded packages
    Clean,
    /// List installed packages
    List {
        /// Filter by name
        filter: Option<String>,
    },
    /// Edit sources.list
    #[command(name = "edit-sources")]
    EditSources,
}

#[derive(Args, Debug, Clone)]
pub struct PacmanFlags {
    /// Sync / install packages (-S, -Sy, -Syu, -S <pkg>)
    #[arg(short = 'S', long, group = "operation")]
    pub sync: bool,

    /// Remove packages (-R, -Rs, -R <pkg>)
    #[arg(short = 'R', long, group = "operation")]
    pub remove: bool,

    /// Query installed packages (-Q, -Qs, -Qu)
    #[arg(short = 'Q', long, group = "operation")]
    pub query: bool,

    /// Search for packages (-Ss)
    #[arg(short = 's', long)]
    pub search: bool,

    /// Show package info (-Si)
    #[arg(short = 'i', long)]
    pub info: bool,

    /// Update package lists (-Sy, -Syu)
    #[arg(short = 'y', long)]
    pub refresh: bool,

    /// Upgrade packages (-Su, -Syu)
    #[arg(short = 'u', long)]
    pub upgrade: bool,

    /// Clean cache (-Sc)
    #[arg(short = 'c', long)]
    pub clean: bool,

    /// Remove unused dependencies
    #[arg(long)]
    pub recursive: bool,

    /// Packages or queries to operate on
    #[arg(value_name = "TARGET")]
    pub targets: Vec<String>,
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}
