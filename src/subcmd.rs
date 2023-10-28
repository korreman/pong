use clap::Subcommand;

mod gen;

#[derive(Debug, Clone, Subcommand)]
pub enum SubCmd {
    /// Install packages.
    ///
    /// Install the specified packages and all of their required dependencies.
    #[command(alias = "i")]
    Install {
        /// Packages to install.
        #[arg(value_name = "PACKAGE")]
        packages: Vec<String>,
        /// Reinstall packages that are already installed.
        #[arg(short, long, conflicts_with("download"))]
        reinstall: bool,
        /// Retrieve packages, but do not install them.
        #[arg(short, long)]
        download: bool,
        /// Install from the AUR in addition to official sources.
        #[arg(short = 'u', long)]
        aur: bool
    },

    /// Remove packages.
    ///
    /// Remove all specified packages and recursively remove any orphaned dependencies.
    #[command(alias = "r")]
    Remove {
        /// Packages to remove.
        #[arg(value_name = "PACKAGE")]
        packages: Vec<String>,
        // TODO: Better naming
        /// Remove all packages that depend on the packages as well.
        #[arg(short, long)]
        cascade: bool,
        /// Keep orphaned dependencies.
        #[arg(short, long)]
        keep_orphans: bool,
        /// Ignore explicit marks on orphaned dependencies.
        ///
        /// Remove orphaned dependencies,
        /// even if they are marked as explicitly installed.
        #[arg(short, long, conflicts_with = "keep_orphans")]
        explicit: bool,
        /// Save configuration files.
        #[arg(short, long)]
        save: bool,
        /// Do not perform AUR-specific operations when removing AUR packages.
        #[arg(long)]
        no_aur: bool,
    },

    /// Refresh the sync database and upgrade packages.
    #[command(alias = "u")]
    Upgrade {
        // TODO: Better name for this option.
        /// Only upgrade packages, do not refresh the sync database.
        #[arg(short, long, conflicts_with("refresh"))]
        no_refresh: bool,
        /// Only refresh the sync database, do not perform upgrades.
        #[arg(short, long)]
        refresh: bool,
        /// Retrieve packages, but do not perform upgrades.
        #[arg(short, long)]
        download: bool,
        /// Do not upgrade AUR packages.
        #[arg(long)]
        no_aur: bool,
    },

    /// Clean the package caches.
    ///
    /// Remove packages that are no longer installed from the cache
    /// as well as unused sync databases.
    #[command(alias = "c")]
    Clean {
        /// Also remove installed packages from the cache.
        #[arg(short, long)]
        all: bool,
        /// Do not perform AUR-specific cleaning.
        #[arg(long)]
        no_aur: bool,
    },

    /// Search for a package.
    #[command(alias = "s")]
    Search {
        /// Query regexes to search for.
        #[arg(value_name = "REGEX")]
        queries: Vec<String>,
        /// Search in installed packages.
        #[arg(short, long)]
        installed: bool,
        /// Search the AUR along with official repositories.
        #[arg(short = 'u', long)]
        aur: bool
    },

    /// List installed packages.
    #[command(alias = "l")]
    List {
        /// Only list packages installed explicitly.
        #[arg(short, long, conflicts_with("deps"))]
        explicit: bool,
        /// Only list packages installed as dependencies.
        #[arg(short, long)]
        deps: bool,
        // TODO: Better name.
        /// Only list packages not required by any installed packages.
        #[arg(short, long)]
        free: bool,
        /// Only list packages found in the sync database(s).
        #[arg(short, long, conflicts_with("no_sync"))]
        sync: bool,
        // TODO: Better name.
        /// Only list packages not found in the sync database(s).
        #[arg(short, long)]
        no_sync: bool,
        /// Only list packages that are out of date.
        #[arg(short, long)]
        upgrades: bool,
    },

    /// Find packages that own files.
    #[command(alias = "w")]
    Which {
        /// Files to search for.
        #[arg(value_name = "FILE")]
        files: Vec<String>,
        /// Search through the sync database(s).
        #[arg(short, long)]
        sync: bool,
        /// Include packages from the AUR (implies --sync).
        #[arg(short = 'u', requires("sync"))]
        aur: bool,
        /// Use a regex for filtering (requires --sync).
        #[arg(short = 'x', long, requires("sync"))]
        regex: bool,
    },
    /// Display various information about packages.
    #[command(alias = "v")]
    View {
        /// Packages to display information about.
        #[arg(value_name = "PACKAGE")]
        packages: Vec<String>,
        /// Query the sync database instead of installed packages.
        #[arg(
            short,
            long,
            conflicts_with_all(["package_file", "changelog"]),
        )]
        sync: bool,
        /// Query package files instead of installed packages.
        #[arg(short, long)]
        package_file: bool,
        /// Print more information.
        ///
        /// This includes:
        /// - Packages that require the named packages.
        /// - Backup files and their modification states.
        #[arg(short, long, conflicts_with("files"), conflicts_with("changelog"))]
        more: bool,
        /// List the files that the packages provide.
        #[arg(short, long, conflicts_with("changelog"))]
        files: bool,
        /// Print the ChangeLog of a local package.
        #[arg(short, long)]
        changelog: bool,
    },

    /// Show the dependency tree of a package.
    #[command(alias = "t")]
    Tree {
        /// The package to show a dependency tree for.
        package: String,
        /// Use ASCII characters for tree formatting.
        #[arg(short, long)]
        ascii: bool,
        /// Limit the depth of recursion.
        #[arg(short, long, value_name = "NUMBER")]
        depth: Option<u32>,
        /// Limit recursion depth for optional dependencies.
        #[arg(short = 'o', long, value_name = "NUMBER")]
        depth_optional: Option<u32>,
        /// Show a tree of reverse dependencies.
        ///
        /// In this tree,
        /// rather than the parents depending on the children,
        /// the children depend on the parents.
        #[arg(short, long)]
        reverse: bool,
    },

    /// Mark/unmark packages as explicitly installed.
    ///
    /// By changing the install reason for a package to 'explicit',
    /// packages that were originally installed as dependencies
    /// can avoid being orphaned and removed indirectly.
    #[command(alias = "p")]
    Pin {
        /// Packages to mark.
        #[arg(value_name = "PACKAGE")]
        packages: Vec<String>,
        /// Mark the packages as dependencies instead.
        #[arg(
            short,
            long,
            long_help = "Mark the packages as dependencies instead, allowing indirect removal."
        )]
        remove: bool,
    },
}
