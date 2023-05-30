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
    #[arg(long)]
    simulate: bool,
    /// Specify when to colorize output.
    #[arg(short, long, value_enum, default_value_t = ColorChoice::Auto)]
    color: ColorChoice,
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
    /// Install or reinstall packages.
    Install {
        /// Packages to install/reinstall.
        #[arg(value_name = "PACKAGE")]
        packages: Vec<String>,
        /// Don't ignore packages that are already installed.
        #[arg(long)]
        reinstall: bool,
        /// Retrieve packages from the server,
        /// but do not install/upgrade anything.
        #[arg(long)]
        download: bool,
    },

    /// Remove packages.
    Remove {
        /// Packages to remove.
        #[arg(value_name = "PACKAGE")]
        packages: Vec<String>,
        /// Do not remove config files.
        #[arg(long)]
        keep_configs: bool,
        /// Do not remove orphaned dependencies.
        #[arg(long)]
        keep_orphans: bool,
        /// Recursively remove all packages that depend on those specified for removal.
        #[arg(short, long)]
        uproot: bool,
    },

    /// Update the remote registry and upgrade packages.
    Upgrade,

    /// Clean the package caches.
    ///
    /// Remove packages that are no longer installed from the cache
    /// as well as unused sync databases.
    Clean {
        /// Remove all packages from the cache,
        /// including ones that are currently installed.
        #[arg(long)]
        all: bool,
    },

    /// Mark packages as explicitly installed, avoiding implicit removal.
    ///
    /// Installed packages are marked with an install reason,
    /// that being either 'explicitly installed' or as 'installed as dependency'.
    /// Dependencies are generally removed along with the last package that depends on them.
    /// By changing the install reason to 'explicit',
    /// packages are pinned in place and avoid being removed indirectly.
    Pin {
        /// Packages to mark.
        #[arg(value_name = "PACKAGE")]
        packages: Vec<String>,
        /// Mark the packages as dependencies instead, allowing implicit removal.
        #[arg(short, long)]
        unpin: bool,
    },

    /// Search for a package.
    Search {
        /// Query strings to search for, regexes used for matching.
        #[arg(value_name = "QUERY")]
        queries: Vec<String>,
    },

    /// Print info on packages.
    Info {
        /// Packages to display info on.
        #[arg(value_name = "PACKAGE")]
        packages: Vec<String>,
    },

    /// Print the dependency tree of a package.
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
        cmd.push("--color".to_owned());
        cmd.push(global.color.to_string());
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
            SubCmd::Upgrade => {
                cmd.push("-Syu".to_owned());
                cmd
            }
            SubCmd::Clean { all } => {
                let arg = if all { "-Scc" } else { "-Sc" };
                cmd.push(arg.to_owned());
                cmd
            }
            SubCmd::Pin {
                packages,
                unpin: dependency,
            } => {
                cmd.push("-D".to_owned());
                let arg = match dependency {
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
            SubCmd::Info { packages } => {
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
                let color = global.color == ColorChoice::Always
                    || global.color == ColorChoice::Auto && atty::is(atty::Stream::Stdout);
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
