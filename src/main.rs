use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::{Command, ExitCode};

use clap::{Args, ColorChoice, Parser, Subcommand};

#[cfg(test)]
mod tests;

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
    /// Install/reinstall packages.
    ///
    /// Install the specified packages and all of their required dependencies.
    #[command(alias = "i")]
    Install {
        /// Packages to install.
        #[arg(value_name = "PACKAGE")]
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
    /// Remove all specified packages and recursively remove all orphaned dependencies.
    /// Will refuse to remove packages that are dependencies of others by default.
    #[command(alias = "r")]
    Remove {
        /// Packages to remove.
        #[arg(value_name = "PACKAGE")]
        packages: Vec<String>,
        /// Recursively remove all packages that depend on the packages being removed.
        #[arg(short, long)]
        uproot: bool,
        /// Do not remove orphaned dependencies.
        #[arg(short = 'o', long)]
        keep_orphans: bool,
        /// Preserve configuration files.
        #[arg(short = 'c', long)]
        keep_configs: bool,
    },

    /// Refresh the sync database and upgrade packages.
    #[command(alias = "u")]
    Upgrade {
        /// Only upgrade packages, do not refresh the sync database.
        #[arg(short, long)]
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

    // TODO: Expand this command by *a lot*.
    /// Search for a package.
    #[command(aliases = ["s", "l"], visible_alias = "list")]
    Search {
        /// Query regexes to search for.
        #[arg(value_name = "QUERY")]
        queries: Vec<String>,
    },

    // TODO: Should we query the sync database or the package database by default?
    /// Display various information about packages.
    #[command(alias = "v")]
    View {
        /// Packages to display information about.
        #[arg(value_name = "PACKAGE")]
        packages: Vec<String>,
        /// Query the sync database instead of installed packages.
        #[arg(short, long)]
        remote: bool,
        /// Query package files instead of installed packages.
        #[arg(short, long)]
        package_file: bool,
        /// Print more information.
        ///
        /// This includes:
        /// - Packages that require the named packages.
        /// - Backup files and their modification states.
        #[arg(short, long)]
        more: bool,
        /// List the files that the packages provide.
        #[arg(short, long)]
        files: bool,
        /// Print the ChangeLog of a package (implies --local).
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

    /// Mark packages as explicitly installed.
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
        unpin: bool,
    },
}

fn quit(e: impl AsRef<str>) -> ! {
    eprintln!("{}", e.as_ref());
    std::process::exit(1)
}

impl SubCmd {
    /// Generate the corresponding underlying command,
    /// and tell whether root user privileges are required to run it.
    fn generate_command(self, global: &GlobalOpts) -> (Vec<String>, bool) {
        let mut cmd = vec!["pacman".to_owned()];
        if global.simulate {
            cmd.push("--print".to_owned());
        };
        if global.debug {
            cmd.push("--debug".to_owned());
        }
        if let Some(color) = global.color {
            cmd.push("--color".to_owned());
            cmd.push(color.to_string());
        }

        let incompatible = |(a, a_str): (bool, &str), (b, b_str): (bool, &str)| {
            if a && b {
                quit(format!("incompatible options: '--{a_str}' and '--{b_str}'"));
            }
        };

        match self {
            SubCmd::Install {
                packages,
                reinstall,
                download,
            } => {
                let mut arg = String::from("-S");
                if download {
                    arg.push('w');
                }
                cmd.push(arg);
                if !reinstall {
                    cmd.push("--needed".to_owned());
                }
                ([cmd, packages].concat(), true)
            }
            SubCmd::Remove {
                packages,
                keep_configs,
                keep_orphans,
                uproot,
            } => {
                let mut arg = String::from("-R");
                if !keep_orphans {
                    arg.push('s');
                }
                if !keep_configs {
                    arg.push('n');
                }
                if uproot {
                    arg.push('c');
                }
                cmd.push(arg);
                ([cmd, packages].concat(), true)
            }
            SubCmd::Upgrade {
                download,
                no_refresh,
                refresh,
            } => {
                incompatible((refresh, "refresh"), (no_refresh, "no-refresh"));

                let mut arg = String::from("-S");
                if download {
                    arg.push('w');
                }
                if !no_refresh {
                    arg.push('y');
                }
                if !refresh {
                    arg.push('u');
                }
                cmd.push(arg);
                (cmd, true)
            }
            SubCmd::Clean { all } => {
                let arg = if all { "-Scc" } else { "-Sc" };
                cmd.push(arg.to_owned());
                (cmd, true)
            }
            SubCmd::Pin { packages, unpin } => {
                cmd.push("-D".to_owned());
                let arg = match unpin {
                    true => "--asdeps",
                    false => "--asexplicit",
                };
                cmd.push(arg.to_owned());
                ([cmd, packages].concat(), true)
            }
            SubCmd::Search { queries } => {
                cmd.push("-Ss".to_owned());
                ([cmd, queries].concat(), false)
            }
            SubCmd::View {
                packages,
                remote,
                package_file,
                changelog,
                files,
                more,
            } => {
                incompatible((changelog, "changelog"), (files, "files"));
                incompatible((changelog, "changelog"), (more, "more"));
                incompatible((files, "files"), (more, "more"));

                incompatible((remote, "remote"), (package_file, "package-file"));
                incompatible((remote, "remote"), (changelog, "changelog"));

                let mut arg = match (remote, files) {
                    (true, false) => String::from("-S"),
                    (true, true) => String::from("-F"),
                    (false, _) => String::from("-Q"),
                };

                if package_file {
                    arg.push('p');
                }
                if changelog {
                    arg.push('c');
                } else if files {
                    arg.push('l');
                } else {
                    arg.push('i');
                }
                if more {
                    arg.push('i');
                }
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
                if global.debug {
                    cmd.push("--debug".to_owned());
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
                let mut cmd_arg = String::from("-");
                if ascii {
                    cmd_arg.push('a');
                }
                let color = global.color == Some(ColorChoice::Always)
                    || (global.color == Some(ColorChoice::Auto) || global.color.is_none())
                        && atty::is(atty::Stream::Stdout);
                if color {
                    cmd_arg.push('c');
                }
                if reverse {
                    cmd_arg.push('r');
                }
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
        }
    }
}

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
