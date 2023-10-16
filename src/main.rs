use std::io::IsTerminal;
use std::os::unix::process::CommandExt;
use std::process::{Command, ExitCode};

use clap::{Args, ColorChoice, Parser, Subcommand};

#[cfg(test)]
mod tests;

fn main() -> ExitCode {
    let args = Cmd::parse();
    let (mut command, sudo) = args.sub.generate_command(&args.opts);
    if args.generate_command {
        println!("{}", command.join(" "));
        ExitCode::SUCCESS
    } else {
        if sudo {
            match sudo::escalate_if_needed() {
                Ok(sudo::RunningAs::Root) | Ok(sudo::RunningAs::Suid) => (),
                _ => {
                    eprintln!("failed to gain root privileges");
                    return ExitCode::FAILURE;
                }
            }
        }
        let mut process = Command::new(command.remove(0));
        for arg in &command {
            process.arg(arg);
        }
        process.exec();
        ExitCode::FAILURE
    }
}

#[derive(Debug, Clone, Parser)]
#[command(author, version, about, max_term_width = 80)]
struct Cmd {
    /// Print the underlying command without executing it.
    #[arg(short, long)]
    generate_command: bool,

    #[command(flatten)]
    opts: GlobalOpts,

    #[command(subcommand)]
    sub: SubCmd,
}

#[derive(Debug, Clone, Args)]
struct GlobalOpts {
    /// Display debug messages.
    #[arg(short, long)]
    debug: bool,
    /// Simulate a test run without performing any changes.
    #[arg(short, long)]
    simulate: bool,
    /// Show less information for certain operations.
    #[arg(short, long)]
    quiet: bool,
    /// Never ask for confirmation.
    #[arg(short, long)]
    yes: bool,
    /// Colorize output.
    #[arg(short, long, value_enum)]
    color: Option<ColorChoice>,
    /// Specify an alternate configuration file.
    #[arg(long, value_name = "FILE")]
    config: Option<String>,
    /// Specify an alternate database location.
    #[arg(long, value_name = "DIR")]
    dbpath: Option<String>,
    /// Specify an alternate directory for GnuPG.
    #[arg(long, value_name = "DIR")]
    gpgdir: Option<String>,
}

#[derive(Debug, Clone, Subcommand)]
enum SubCmd {
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
    },

    /// Search for a package.
    #[command(alias = "s")]
    Search {
        /// Query regexes to search for.
        #[arg(value_name = "REGEX")]
        queries: Vec<String>,
        /// Search in installed packages.
        #[arg(short, long)]
        local: bool,
        // TODO: Regexes aren't used when searching for files.
        /// Search for packages that own the specified file(s).
        #[arg(short, long)]
        file: bool,
        /// Do not use regex for filtering.
        #[arg(short, long, conflicts_with("local"))]
        exact: bool,
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
        /// Only list packages not required by any installed packages.
        #[arg(short, long)]
        free: bool,
        /// Only list packages found in the sync database(s).
        #[arg(short, long, conflicts_with("no_sync"))]
        sync: bool,
        // TODO: Better name for below.
        /// Only list packages not found in the sync database(s).
        #[arg(short, long)]
        no_sync: bool,
        /// Only list packages that are out of date.
        #[arg(short, long)]
        upgrades: bool,
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

impl SubCmd {
    /// Generate the corresponding underlying command,
    /// and tell whether root user privileges are required to run it.
    fn generate_command(self, global: &GlobalOpts) -> (Vec<String>, bool) {
        let mut root = false;
        let mut cli = Cli::new("pacman");
        cli.arg("--print", global.simulate);
        cli.arg("--debug", global.debug);
        cli.arg("--noconfirm", global.yes);
        cli.arg_opt("--color", &global.color);
        cli.arg_opt("--config", &global.config);
        cli.arg_opt("--dbpath", &global.dbpath);
        cli.arg_opt("--gpgdir", &global.gpgdir);

        match self {
            SubCmd::Install {
                packages,
                reinstall,
                download,
            } => {
                root = true;
                cli.arg("-S", true);
                cli.flag('q', global.quiet);
                cli.flag('w', download);
                cli.arg("--needed", !reinstall);
                cli.args(packages);
            }
            SubCmd::Remove {
                packages,
                save,
                explicit,
                keep_orphans,
                cascade,
            } => {
                root = true;
                cli.arg("-R", true);
                cli.flag('n', !save);
                cli.flag('s', !keep_orphans);
                cli.flag('s', explicit);
                cli.flag('c', cascade);
                cli.args(packages);
            }
            SubCmd::Upgrade {
                download,
                no_refresh,
                refresh,
            } => {
                root = true;
                cli.arg("-S", true);
                cli.flag('q', global.quiet);
                cli.flag('w', download);
                cli.flag('y', !no_refresh);
                cli.flag('u', !refresh);
            }
            SubCmd::Clean { all } => {
                root = true;
                cli.arg("-Sc", true);
                cli.flag('c', all);
            }
            SubCmd::Pin { packages, remove } => {
                root = true;
                cli.arg("-D", true);
                cli.flag('q', global.quiet);
                cli.arg("--asexplicit", !remove);
                cli.arg("--asdeps", remove);
                cli.args(packages);
            }
            SubCmd::Search {
                queries,
                file,
                local,
                exact,
            } => {
                let arg = match (local, file, exact) {
                    (true, true, _) => "-Qo",
                    (true, false, _) => "-Qs",
                    (false, true, true) => "-F",
                    (false, true, false) => "-Fx",
                    (false, false, _) => "-Ss",
                };
                cli.arg(arg, true);
                cli.flag('q', global.quiet);
                cli.args(queries);
            }
            SubCmd::View {
                packages,
                sync,
                package_file,
                changelog,
                files,
                more,
            } => {
                let arg = match (sync, files) {
                    (true, false) => "-S",
                    (true, true) => "-F",
                    (false, _) => "-Q",
                };
                cli.arg(arg, true);
                cli.flag('q', global.quiet);
                cli.flag('p', package_file);
                cli.flag('c', changelog);
                cli.flag('l', files);
                cli.flag('i', !(changelog && files));
                cli.flag('i', more);
                cli.args(packages);
            }
            SubCmd::Tree {
                package,
                ascii,
                depth,
                depth_optional,
                reverse,
            } => {
                cli = Cli::new("pactree");
                cli.arg("--debug", global.debug);
                cli.arg_opt("--config", &global.config);
                cli.arg_opt("--dbpath", &global.dbpath);
                cli.arg_opt("--gpgdir", &global.gpgdir);

                let color = {
                    let terminal = std::io::stdout().is_terminal();
                    let auto = global.color == Some(ColorChoice::Auto) || global.color.is_none();
                    let always = global.color == Some(ColorChoice::Always);
                    always || auto && terminal
                };

                cli.arg("-", true);
                cli.flag('a', ascii);
                cli.flag('c', color);
                cli.flag('r', reverse);
                if cli.0.last().unwrap() == "-" {
                    cli.0.pop().unwrap();
                }

                cli.arg_opt("-d", &depth);
                cli.arg_opt("--optional", &depth_optional);
                cli.args(vec![package]);
            }
            SubCmd::List {
                explicit,
                deps,
                no_sync,
                sync,
                free,
                upgrades,
            } => {
                cli.arg("-Q", true);
                cli.flag('q', global.quiet);
                cli.flag('e', explicit);
                cli.flag('d', deps);
                cli.flag('m', no_sync);
                cli.flag('n', sync);
                cli.flag('t', free);
                cli.flag('u', upgrades);
            }
        }
        (cli.0, root)
    }
}

struct Cli(Vec<String>);

impl Cli {
    fn new(base: &str) -> Self {
        Self(vec![base.to_owned()])
    }

    fn flag(&mut self, f: char, guard: bool) {
        if guard {
            self.0.last_mut().unwrap().push(f);
        }
    }

    fn arg(&mut self, a: &str, guard: bool) {
        if guard {
            self.0.push(a.to_owned());
        }
    }

    fn arg_opt<T: std::fmt::Display>(&mut self, a: &str, value: &Option<T>) {
        if let Some(value) = value {
            self.0.push(a.to_owned());
            self.0.push(format!("{value}"));
        }
    }

    fn args(&mut self, mut args: Vec<String>) {
        self.0.append(&mut args);
    }
}
