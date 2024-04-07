use std::os::unix::process::CommandExt;
use std::process::{Command, ExitCode};

mod aur;
mod builder;
mod command;
mod generate;

use aur::{Aur, AurPassthrough};
use clap::{CommandFactory, Parser};
use command::Cmd;
use generate::*;

fn main() -> ExitCode {
    let args = Cmd::parse();
    let Some(sub) = &args.sub else {
        if Cmd::command().print_help().is_ok() {
            return ExitCode::SUCCESS;
        } else {
            return ExitCode::FAILURE;
        }
    };
    let mut cli = generate_command(sub.clone(), &args.opts);

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
