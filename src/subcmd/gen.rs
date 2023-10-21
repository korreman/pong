use crate::{subcmd::SubCmd, GlobalOpts};
use clap::ColorChoice;
use std::io::IsTerminal;

impl SubCmd {
    /// Generate the corresponding underlying command,
    /// and tell whether root user privileges are required to run it.
    pub(crate) fn generate_command(self, global: &GlobalOpts) -> (Vec<String>, bool) {
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
                cli.root = true;
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
                cli.root = true;
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
                cli.root = true;
                cli.arg("-S", true);
                cli.flag('q', global.quiet);
                cli.flag('w', download);
                cli.flag('y', !no_refresh);
                cli.flag('u', !refresh);
            }
            SubCmd::Clean { all } => {
                cli.root = true;
                cli.arg("-Sc", true);
                cli.flag('c', all);
            }
            SubCmd::Pin { packages, remove } => {
                cli.root = true;
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
                if cli.cmd.last().unwrap() == "-" {
                    cli.cmd.pop().unwrap();
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
        (cli.cmd, cli.root)
    }
}

struct Cli {
    cmd: Vec<String>,
    root: bool,
}

impl Cli {
    fn new(base: &str) -> Self {
        Self {
            cmd: vec![base.to_owned()],
            root: false,
        }
    }

    fn flag(&mut self, f: char, guard: bool) {
        if guard {
            self.cmd.last_mut().unwrap().push(f);
        }
    }

    fn arg(&mut self, a: &str, guard: bool) {
        if guard {
            self.cmd.push(a.to_owned());
        }
    }

    fn arg_opt<T: std::fmt::Display>(&mut self, a: &str, value: &Option<T>) {
        if let Some(value) = value {
            self.cmd.push(a.to_owned());
            self.cmd.push(format!("{value}"));
        }
    }

    fn args(&mut self, mut args: Vec<String>) {
        self.cmd.append(&mut args);
    }
}