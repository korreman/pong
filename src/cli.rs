pub struct Cli {
    pub cmd: Vec<String>,
    pub sudo: bool,
}

impl Cli {
    pub fn new(base: &str) -> Self {
        Self {
            cmd: vec![base.to_owned()],
            sudo: false,
        }
    }

    pub fn flag(&mut self, f: char, guard: bool) {
        if guard {
            self.cmd.last_mut().unwrap().push(f);
        }
    }

    pub fn arg(&mut self, a: &str, guard: bool) {
        if guard {
            self.cmd.push(a.to_owned());
        }
    }

    pub fn arg_opt<T: std::fmt::Display>(&mut self, a: &str, value: &Option<T>) {
        if let Some(value) = value {
            self.cmd.push(a.to_owned());
            self.cmd.push(format!("{value}"));
        }
    }

    pub fn args(&mut self, mut args: Vec<String>) {
        self.cmd.append(&mut args);
    }
}
