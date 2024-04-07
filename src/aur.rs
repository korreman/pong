use crate::builder::Cli;

pub trait Aur {
    fn transform(&self, cli: &mut Cli);
}

pub struct AurPassthrough<'a>(pub &'a str);

impl<'a> Aur for AurPassthrough<'a> {
    fn transform(&self, cli: &mut Cli) {
        cli.cmd[0] = self.0.to_owned();
    }
}
