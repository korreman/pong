# pong

A CLI wrapper that reorganizes `pacman` operations for more intuitive use.

`pacman` is a great and reliable package manager,
but it is no secret that many find it difficult to use and memorize.
Pages like the
[`pacman` Rosetta](https://wiki.archlinux.org/title/Pacman/Rosetta)
should be indication enough that people find it rather confusing.

`pong` attempts to solve this by providing an intuitive flat set of commands for package management.
Commands perform one type of action,
and subsequent flags do not change the general action being performed.
Similarly, the same type of action is not spread across several commands.

Alternatively, `pong` can be used as a lookup tool for `pacman`;
passing the `-g/--generate` flag will make pong print the command
that it was _going_ to run rather than run it.

`pong` also supports AUR helpers through the `--aur-helper` parameter.
Upkeep operations will be dispatched to the helper,
while search and installation are gated behind a flag.
This ensures that AUR packages are properly managed,
but AUR packages aren't installed unintentionally.

## Commands

The current commands are:

- `install`: Install packages
- `remove`: Remove packages
- `upgrade`: Refresh the sync database and upgrade packages
- `clean`: Clean the package caches
- `search`: Search for a package
- `list`: List installed packages
- `which`: Search for packages that own files
- `view`: Display various information about packages
- `tree`: Show the dependency tree of a package
- `pin`: Mark/unmark packages as explicitly installed
- `help`: Print this message or the help of the given subcommand(s)

## Dependencies

The only current dependencies are the packages `pacman` and `pacman-contrib`.

## Roadmap

- [x] Basic user functionality.
- [ ] AUR helper support.
    - [x] Installation
    - [x] Removal
    - [x] Upgrade
    - [x] Cleaning
    - [x] Searching
    - [ ] AUR-specific operations
    - [ ] Listing
    - [ ] Viewing
- [ ] Completion
    - [ ] Command completion.
    - [ ] Content completion.
- [ ] ...
