use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::{Command, ExitCode};

use clap::{Args, ColorChoice, Parser, Subcommand};

#[derive(Debug, Clone, Parser)]
#[command(author, version, about)]
struct Cmd {
    /// Print the corresponding command to stdout instead of running it.
    #[arg(long)]
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

    /// Mark packages as directly installed. (TODO: better description)
    Pin {
        /// Packages to pin.
        #[arg(value_name = "PACKAGE")]
        packages: Vec<String>,
    },

    /// Unmark packages as directly installed. (TODO: better description)
    Unpin {
        /// Packages to unpin.
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
                let mut cmd_arg = String::from("-R");
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
                depth,
                depth_optional,
                reverse,
            } => {
                cmd = vec!["pactree".to_owned()];
                let mut cmd_arg = String::from("-");
                if ascii {
                    cmd_arg.push('a');
                }
                let color = global.color == ColorChoice::Always
                    || global.color == ColorChoice::Auto && atty::is(atty::Stream::Stdout);
                if color {
                    cmd_arg.push('c');
                }
                // TODO: More global configuration options.
                if reverse {
                    cmd_arg.push('r');
                }
                if let Some(d) = depth {
                    cmd.push("-d".to_owned());
                    cmd.push(format!("{d}"));
                }
                if let Some(dopt) = depth_optional {
                    cmd.push(format!("--optional={dopt}"));
                }
                if global.verbose {
                    cmd.push("--debug".to_owned());
                }
                if cmd_arg != "-" {
                    cmd.push(cmd_arg);
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
