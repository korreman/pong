use clap::{Command, CommandFactory};
use clap_complete::{generate_to, shells::*};
use clap_complete_nushell::Nushell;
use std::env;
use std::io::Error;

include!("src/command.rs");

fn main() -> Result<(), Error> {
    let outdir = match env::var_os("OUT_DIR") {
        None => return Ok(()),
        Some(outdir) => outdir,
    };

    let mut cmd: Command = Cmd::command();
    generate_to(Bash, &mut cmd, "pong", outdir.clone()).unwrap();
    generate_to(Zsh, &mut cmd, "pong", outdir.clone()).unwrap();
    generate_to(Elvish, &mut cmd, "pong", outdir.clone()).unwrap();
    generate_to(Fish, &mut cmd, "pong", outdir.clone()).unwrap();
    generate_to(PowerShell, &mut cmd, "pong", outdir.clone()).unwrap();
    generate_to(Nushell, &mut cmd, "pong", outdir.clone()).unwrap();

    println!("cargo:warning=completions are generated in {outdir:?}");

    Ok(())
}
