# pong

A CLI wrapper that reorganizes `pacman` operations for more intuitive use.

`pacman` is a great and reliable package manager,
but it is no secret that many find it difficult to use and memorize.
`pong` attempts to solve this
by providing an intuitive flat set of commands for package management.

Commands perform only one type of action,
and subsequent flags do not change the general action being performed.
Additionally, variations of the same action are not spread across several commands.

`pong` emphasizes discoverability.
It self-documents with both brief and long descriptions for all commands.
Simply run `pong` with no subcommand to show the overview,
then run `pong [cmd] -h`/`pong [cmd] --help` to show the short/long help for a given command.

`pong` also supports AUR helpers through the `--aur-helper` parameter.
Upkeep operations will be dispatched to the helper,
while AUR search and installation are gated behind a flag.
This ensures that AUR packages are properly managed,
but users are aware of whether packages are from the AUR.

## Commands

The current commands are:

- `i`/`install`: Install packages
- `r`/`remove`: Remove packages
- `u`/`upgrade`: Refresh the sync database and upgrade packages
- `c`/`clean`: Clean the package caches
- `s`/`search`: Search for a package
- `l`/`list`: List installed packages
- `w`/`which`: Search for packages that own files
- `v`/`view`: Display various information about packages
- `t`/`tree`: Show the dependency tree of a package
- `t`/`pin`: Mark/unmark packages as explicitly installed
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
