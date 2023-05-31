use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::{Command, ExitCode};

use clap::{Args, ColorChoice, Parser, Subcommand};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Parser)]
#[command(author, version, about)]
struct Cmd {
    /// Print the corresponding command to stdout instead of running it.
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
    verbose: bool,
    /// Simulate a test run without performing any changes.
    #[arg(short, long)]
    simulate: bool,
    /// Specify when to colorize output.
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
        /// Packages to install/reinstall.
        #[arg(value_name = "PACKAGE")]
        packages: Vec<String>,
        /// Reinstall packages that are already installed.
        #[arg(long)]
        reinstall: bool,
        /// Retrieve packages from the server,
        /// but do not install/upgrade anything.
        #[arg(long)]
        download: bool,
    },

    /// Remove packages.
    ///
    /// Remove all specified packages and recursively remove all orphaned dependencies.
    /// Refuses to remove packages that are dependencies of others by default.
    #[command(alias = "r")]
    Remove {
        /// Packages to remove.
        #[arg(value_name = "PACKAGE")]
        packages: Vec<String>,
        /// Recursively remove all packages that depend on those specified for removal.
        #[arg(short, long)]
        uproot: bool,
        /// Do not remove orphaned dependencies.
        #[arg(short = 'o', long)]
        keep_orphans: bool,
        /// Remove configuration files as well.
        #[arg(short = 'c', long)]
        keep_configs: bool,
    },

    /// Update the package database and upgrade packages.
    #[command(alias = "u")]
    Upgrade {
        /// Do not update the package database.
        #[arg(long)]
        no_sync: bool,
        /// Do not upgrade any packages.
        #[arg(long)]
        only_sync: bool,
        /// Retrieve packages from the server,
        /// but do not install/upgrade anything.
        #[arg(long)]
        download: bool,
    },

    /// Clean the package caches.
    ///
    /// Remove packages that are no longer installed from the cache
    /// as well as unused sync databases.
    #[command(alias = "c")]
    Clean {
        /// Remove all packages from the cache,
        /// including ones that are currently installed.
        #[arg(long)]
        all: bool,
    },

    /// Mark packages as explicitly installed, avoiding indirect removal.
    ///
    /// Installed packages are marked with an install reason,
    /// that being either 'explicitly installed' or as 'installed as dependency'.
    /// Dependencies are generally removed along with the last package that depends on them.
    /// By changing the install reason to 'explicit',
    /// packages are pinned in place and avoid being removed indirectly.
    #[command(alias = "p")]
    Pin {
        /// Packages to mark.
        #[arg(value_name = "PACKAGE")]
        packages: Vec<String>,
        /// Mark the packages as dependencies instead, allowing implicit removal.
        #[arg(short, long)]
        unpin: bool,
    },

    /// Search for a package.
    #[command(alias = "s")]
    Search {
        /// Query strings to search for, regexes used for matching.
        #[arg(value_name = "QUERY")]
        queries: Vec<String>,
    },

    /// Print info on packages.
    #[command(alias = "d", visible_alias = "info")]
    Desc {
        /// Packages to display info on.
        #[arg(value_name = "PACKAGE")]
        packages: Vec<String>,
    },

    /// Print the dependency tree of a package.
    #[command(alias = "t")]
    Tree {
        /// The package to print a dependency tree for.
        package: String,
        /// Use ASCII characters for tree formatting.
        #[arg(short, long)]
        ascii: bool,
        /// Limit the depth of recursion.
        #[arg(short, long, value_name = "NUMBER")]
        depth: Option<u32>,
        /// Limit depth of optional.
        #[arg(long, value_name = "NUMBER")]
        depth_optional: Option<u32>,
        /// List package dependants instead of dependencies.
        #[arg(short, long)]
        reverse: bool,
    },
}

impl SubCmd {
    fn generate_command(self, global: &GlobalOpts) -> Vec<String> {
        let mut cmd = vec!["pacman".to_owned()];
        if global.simulate {
            cmd.push("--print".to_owned());
        };
        if global.verbose {
            cmd.push("--debug".to_owned());
        }
        if let Some(color) = global.color {
            cmd.push("--color".to_owned());
            cmd.push(color.to_string());
        }
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
                [cmd, packages].concat()
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
                [cmd, packages].concat()
            }
            SubCmd::Upgrade {
                download,
                no_sync,
                only_sync,
            } => {
                if no_sync && only_sync {
                    eprintln!("either the package database must be updated or the packages upgraded for this command to have any effect");
                    std::process::exit(-1);
                }
                let mut arg = String::from("-S");
                if download {
                    arg.push('w');
                }
                if !no_sync {
                    arg.push('y');
                }
                if !only_sync {
                    arg.push('u');
                }
                cmd.push(arg);
                cmd
            }
            SubCmd::Clean { all } => {
                let arg = if all { "-Scc" } else { "-Sc" };
                cmd.push(arg.to_owned());
                cmd
            }
            SubCmd::Pin { packages, unpin } => {
                cmd.push("-D".to_owned());
                let arg = match unpin {
                    true => "--asdeps",
                    false => "--asexplicit",
                };
                cmd.push(arg.to_owned());
                [cmd, packages].concat()
            }
            SubCmd::Search { queries } => {
                cmd.push("-Ss".to_owned());
                [cmd, queries].concat()
            }
            SubCmd::Desc { packages } => {
                cmd.push("-Si".to_owned());
                [cmd, packages].concat()
            }
            SubCmd::Tree {
                package,
                ascii,
                depth,
                depth_optional,
                reverse,
            } => {
                cmd = vec!["pactree".to_owned()];
                if global.verbose {
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
                cmd
            }
        }
    }
}

fn main() -> ExitCode {
    let args = Cmd::parse();
    let mut command = args.sub.generate_command(&args.opts);
    if args.generate_command {
        println!("{}", command.join(" "));
        ExitCode::SUCCESS
    } else {
        let mut process = Command::new(command.remove(0));
        for arg in &command {
            process.arg(arg);
        }
        process.exec();
        ExitCode::FAILURE
    }
}
