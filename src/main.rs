use std::io::IsTerminal;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
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
    config: Option<PathBuf>,
    /// Specify an alternate database location.
    #[arg(long, value_name = "DIR")]
    dbpath: Option<PathBuf>,
    /// Specify an alternate directory for GnuPG.
    #[arg(long, value_name = "DIR")]
    gpgdir: Option<PathBuf>,
}

#[derive(Debug, Clone, Subcommand)]
enum SubCmd {
    /// Install packages.
    ///
    /// Install the specified packages and all of their required dependencies.
    #[command(alias = "i")]
    Install {
        /// Packages to install.
        #[arg(value_name = "PACKAGES")]
        packages: Vec<String>,
        /// Reinstall packages that are already installed.
        #[arg(short, long)]
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
        #[arg(value_name = "PACKAGES")]
        packages: Vec<String>,
        // TODO: Better naming
        /// Remove all packages that depend on the packages as well.
        #[arg(short, long)]
        cascade: bool,
        /// Keep orphaned dependencies.
        #[arg(short = 'o', long)]
        keep_orphans: bool,
        /// Preserve configuration files.
        #[arg(short = 'c', long)]
        keep_configs: bool,
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
        #[arg(value_name = "REGEXES")]
        queries: Vec<String>,
        /// Search in installed packages.
        #[arg(short, long)]
        local: bool,
        /// Search for packages that own the specified file(s).
        #[arg(short, long)]
        file: bool,
        /// Do not use regex for filtering (files).
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
        #[arg(value_name = "PACKAGES")]
        packages: Vec<String>,
        /// Query the sync database instead of installed packages.
        #[arg(
            short,
            long,
            conflicts_with("package_file"),
            conflicts_with("changelog")
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
        #[arg(value_name = "PACKAGES")]
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
        let mut cmd = vec!["pacman".to_owned()];
        carg(&mut cmd, "--print", global.simulate);
        carg(&mut cmd, "--debug", global.debug);
        carg(&mut cmd, "--noconfirm", global.yes);
        if let Some(color) = global.color {
            cmd.push("--color".to_owned());
            cmd.push(color.to_string());
        }
        if let Some(config) = &global.config {
            cmd.push(format!(
                "--config {}",
                config.to_str().expect("non-unicode isn't supported (yet?)")
            ));
        }
        if let Some(dbpath) = &global.dbpath {
            cmd.push(format!(
                "--dbpath {}",
                dbpath.to_str().expect("non-unicode isn't supported (yet?)")
            ));
        }
        if let Some(gpgdir) = &global.gpgdir {
            cmd.push(format!(
                "--gpgdir {}",
                gpgdir.to_str().expect("non-unicode isn't supported (yet?)")
            ));
        }

        match self {
            SubCmd::Install {
                packages,
                reinstall,
                download,
            } => {
                let mut arg = String::from("-S");
                flag(&mut arg, 'q', global.quiet);
                flag(&mut arg, 'w', download);
                cmd.push(arg);
                carg(&mut cmd, "--needed", !reinstall);
                ([cmd, packages].concat(), true)
            }
            SubCmd::Remove {
                packages,
                keep_configs,
                keep_orphans,
                cascade: force,
            } => {
                let mut arg = String::from("-R");
                flag(&mut arg, 'n', !keep_configs);
                flag(&mut arg, 's', !keep_orphans);
                flag(&mut arg, 'c', force);
                cmd.push(arg);
                ([cmd, packages].concat(), true)
            }
            SubCmd::Upgrade {
                download,
                no_refresh,
                refresh,
            } => {
                let mut arg = String::from("-S");
                flag(&mut arg, 'q', global.quiet);
                flag(&mut arg, 'w', download);
                flag(&mut arg, 'y', !no_refresh);
                flag(&mut arg, 'u', !refresh);
                cmd.push(arg);
                (cmd, true)
            }
            SubCmd::Clean { all } => {
                let arg = if all { "-Scc" } else { "-Sc" };
                cmd.push(arg.to_owned());
                (cmd, true)
            }
            SubCmd::Pin { packages, remove } => {
                match global.quiet {
                    true => cmd.push("-Dq".to_owned()),
                    false => cmd.push("-D".to_owned()),
                }
                let arg = match remove {
                    true => "--asdeps",
                    false => "--asexplicit",
                };
                cmd.push(arg.to_owned());
                ([cmd, packages].concat(), true)
            }
            SubCmd::Search {
                queries,
                file,
                local,
                exact,
            } => {
                let arg = match (local, file) {
                    (true, true) => "-Qo",
                    (true, false) => "-Qs",
                    (false, true) => {
                        if exact {
                            "-F"
                        } else {
                            "-Fx"
                        }
                    }
                    (false, false) => "-Ss",
                };
                let mut arg = arg.to_owned();
                flag(&mut arg, 'q', global.quiet);
                cmd.push(arg);
                ([cmd, queries].concat(), false)
            }
            SubCmd::View {
                packages,
                sync,
                package_file,
                changelog,
                files,
                more,
            } => {
                let mut arg = match (sync, files) {
                    (true, false) => String::from("-S"),
                    (true, true) => String::from("-F"),
                    (false, _) => String::from("-Q"),
                };
                flag(&mut arg, 'q', global.quiet);
                flag(&mut arg, 'p', package_file);
                flag(&mut arg, 'c', changelog);
                flag(&mut arg, 'l', files);
                flag(&mut arg, 'i', !(changelog && files));
                flag(&mut arg, 'i', more);
                cmd.push(arg);
                ([cmd, packages].concat(), false)
            }
            SubCmd::Tree {
                package,
                ascii,
                depth,
                depth_optional,
                reverse,
            } => {
                cmd = vec!["pactree".to_owned()];
                carg(&mut cmd, "--debug", global.debug);
                if let Some(config) = &global.config {
                    cmd.push(format!(
                        "--config {}",
                        config.to_str().expect("non-unicode isn't supported (yet?)")
                    ));
                }
                if let Some(dbpath) = &global.dbpath {
                    cmd.push(format!(
                        "--dbpath {}",
                        dbpath.to_str().expect("non-unicode isn't supported (yet?)")
                    ));
                }
                if let Some(gpgdir) = &global.gpgdir {
                    cmd.push(format!(
                        "--gpgdir {}",
                        gpgdir.to_str().expect("non-unicode isn't supported (yet?)")
                    ));
                }
                let mut cmd_arg = String::from("-");
                flag(&mut cmd_arg, 'a', ascii);
                let color = global.color == Some(ColorChoice::Always)
                    || (global.color == Some(ColorChoice::Auto) || global.color.is_none())
                        && std::io::stdout().is_terminal();
                flag(&mut cmd_arg, 'c', color);
                flag(&mut cmd_arg, 'r', reverse);
                if let Some(d) = depth {
                    cmd.push("-d".to_owned());
                    cmd.push(format!("{d}"));
                }
                if cmd_arg != "-" {
                    cmd.push(cmd_arg);
                }
                if let Some(dopt) = depth_optional {
                    cmd.push(format!("--optional={dopt}"));
                }
                cmd.push(package);
                (cmd, false)
            }
            SubCmd::List {
                explicit,
                deps,
                no_sync,
                sync,
                free,
                upgrades,
            } => {
                let mut arg = String::from("-Q");
                flag(&mut arg, 'q', global.quiet);
                flag(&mut arg, 'e', explicit);
                flag(&mut arg, 'd', deps);
                flag(&mut arg, 'm', no_sync);
                flag(&mut arg, 'n', sync);
                flag(&mut arg, 't', free);
                flag(&mut arg, 'u', upgrades);
                cmd.push(arg);
                (cmd, false)
            }
        }
    }
}

fn flag(arg: &mut String, f: char, guard: bool) {
    if guard {
        arg.push(f);
    }
}

fn carg(cmd: &mut Vec<String>, a: &str, guard: bool) {
    if guard {
        cmd.push(a.to_owned());
    }
}
