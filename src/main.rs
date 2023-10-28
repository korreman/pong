use std::os::unix::process::CommandExt;
use std::process::{Command, ExitCode};

use aur::{Aur, AurPassthrough};
use clap::{Args, ColorChoice, CommandFactory, Parser};
use subcmd::SubCmd;

mod aur;
mod cli;
mod subcmd;

#[derive(Debug, Clone, Parser)]
#[command(
    author,
    version,
    about,
    max_term_width = 80,
    disable_version_flag = true
)]
struct Cmd {
    // TODO: Better name.
    /// Print the underlying command without executing it.
    #[arg(short, long)]
    generate_command: bool,

    #[command(flatten)]
    opts: GlobalOpts,

    #[command(subcommand)]
    sub: Option<SubCmd>,
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

    /// Specify an AUR helper to dispatch AUR commands to.
    #[arg(long, value_name = "CMD")]
    aur_helper: Option<String>,

    /// Print version
    #[arg(short = 'v', short_alias = 'V', long, action = clap::builder::ArgAction::Version)]
    version: (),
}

fn main() -> ExitCode {
    let args = Cmd::parse();
    let Some(sub) = &args.sub else {
        if Cmd::command().print_help().is_ok() {
            return ExitCode::SUCCESS;
        } else {
            return ExitCode::FAILURE;
        }
    };
    let mut cli = sub.clone().generate_command(&args.opts);

    if let (true, Some(helper)) = (cli.aur, args.opts.aur_helper) {
        AurPassthrough(helper.as_str()).transform(&mut cli);
    }

    if args.generate_command {
        println!("{}", cli.cmd.join(" "));
        return ExitCode::SUCCESS;
    }

    if cli.sudo && !cli.aur {
        match sudo::escalate_if_needed() {
            Ok(sudo::RunningAs::Root) | Ok(sudo::RunningAs::Suid) => (),
            _ => {
                eprintln!("failed to gain root privileges");
                return ExitCode::FAILURE;
            }
        }
    }

    let mut process = Command::new(cli.cmd.remove(0));
    for arg in &cli.cmd {
        process.arg(arg);
    }
    process.exec();
    ExitCode::FAILURE
}
