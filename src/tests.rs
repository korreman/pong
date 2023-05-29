// Most tests are essentially gonna be the same.
// Compare the invocation of the command with a corresponding operation.
// The two problems I can think of:
// 1. How do we create separated arguments in a concise manner?
// 2. The output arguments can be arranged in several orders that are equivalent.
//    How should we test this correctly?

use std::ffi::OsStr;

use crate::*;

fn parse(args: &[&OsStr]) -> String {
    let options = Cmd::parse_from(args);
    options.sub.generate_command(&options.opts).join(" ")
}

macro_rules! generates {
    ( $( $arg:expr ),+ ; $expected:expr ) => {
        let actual = parse(&[OsStr::new("pong"), $( OsStr::new($arg) ),+ ]);
        assert_eq!(actual, $expected);
    };
}

#[test]
fn upgrade() {
    generates![ "upgrade"
              ; "pacman --color auto -Syu" ];

    generates![ "--color", "never", "upgrade"
              ; "pacman --color never -Syu" ];

    generates![ "--color", "auto", "upgrade"
              ; "pacman --color auto -Syu" ];

    generates![ "-c", "auto", "upgrade"
              ; "pacman --color auto -Syu" ];

    generates![ "--color", "always", "upgrade"
              ; "pacman --color always -Syu" ];

    generates![ "--simulate", "--color", "always", "upgrade"
              ; "pacman --print --color always -Syu" ];
}

#[test]
fn tree() {
    generates![ "tree", "abc"
              ; "pactree -c abc" ];

    generates![ "--color", "never", "tree", "abc"
              ; "pactree abc" ];
}
