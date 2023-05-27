use std::os::unix::process::CommandExt;
use std::process::{Command, ExitCode};

use clap::{Parser, Subcommand};

#[derive(Debug, Clone, Parser)]
#[command(author, version, about)]
struct Cmd {
    /// Display debug messages.
    #[arg(short, long)]
    verbose: bool,

    /// Print the generated pacman command to stdout instead of running it.
    #[arg(long)]
    generate_command: bool,

    /// Simulate a test run without performing any changes.
    #[arg(long)]
    simulate: bool,

    #[command(subcommand)]
    sub: SubCmd,
}

#[derive(Debug, Clone, Subcommand)]
enum SubCmd {
    /// Install or reinstall packages.
    Install {
        /// Packages to install/reinstall.
        packages: Vec<String>,
    },

    /// Remove packages.
    Remove {
        /// Packages to remove.
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

    /// Search for a package.
    Search {
        /// Query strings to search for, regexes used for matching.
        queries: Vec<String>,
    },

    /// Print info on packages.
    Info {
        /// Packages to display info on.
        packages: Vec<String>,
    },

    /// Mark packages as directly installed. (TODO: better description)
    Pin {
        /// Packages to pin.
        packages: Vec<String>,
    },

    /// Unmark packages as directly installed. (TODO: better description)
    Unpin {
        /// Packages to unpin.
        packages: Vec<String>,
    },

    /// Print the dependency tree of a package.
    Tree {
        /// The package to print a dependency tree for.
        package: String,
        /// Use ASCII characters for tree formatting.
        #[arg(short, long)]
        ascii: bool,
        /// Colorize output.
        #[arg(short, long)]
        color: bool,
        /// Limit the depth of recursion.
        #[arg(short, long, value_name = "NUMBER")]
        depth: Option<u32>,
        // TODO: Rest of arguments.
    },
}

impl SubCmd {
    fn generate_command(self, simulate: bool) -> Vec<String> {
        let mut cmd = if !simulate {
            vec!["pacman".to_owned()]
        } else {
            vec!["pacman".to_owned(), "--print".to_owned()]
        };
        match self {
            SubCmd::Install { packages } => {
                cmd.push("-S".to_owned());
                [cmd, packages].concat()
            }
            SubCmd::Remove {
                packages,
                keep_configs,
                keep_orphans,
                uproot,
            } => {
                let mut cmd_arg = String::from("R");
                if !keep_orphans {
                    cmd_arg.push('s');
                }
                if !keep_configs {
                    cmd_arg.push('n');
                }
                if uproot {
                    cmd_arg.push('c');
                }
                cmd.push(cmd_arg);
                [cmd, packages].concat()
            }
            SubCmd::Upgrade => {
                cmd.push("-Syu".to_owned());
                cmd
            }
            SubCmd::Search { queries } => {
                cmd.push("-Ss".to_owned());
                [cmd, queries].concat()
            }
            SubCmd::Info { packages } => {
                cmd.push("-Si".to_owned());
                [cmd, packages].concat()
            }
            SubCmd::Pin { packages } => {
                cmd.push("-D".to_owned());
                cmd.push("--asexplicit".to_owned());
                [cmd, packages].concat()
            }
            SubCmd::Unpin { packages } => {
                cmd.push("-S".to_owned());
                cmd.push("--asdeps".to_owned());
                [cmd, packages].concat()
            }
            SubCmd::Tree {
                package,
                ascii,
                color,
                depth,
            } => {
                cmd = vec!["pactree".to_owned()];
                let mut cmd_arg = String::from("-");
                if ascii {
                    cmd_arg.push('a');
                }
                if color {
                    cmd_arg.push('c');
                }
                if let Some(depth) = depth {
                    cmd_arg.push('d');
                    cmd.push(format!("{depth}"));
                }
                cmd.push(package);
                cmd
            }
        }
    }
}

fn main() -> ExitCode {
    let args = Cmd::parse();
    let mut command = args.sub.generate_command(args.simulate);
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
