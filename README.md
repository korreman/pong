__THIS TOOL IS IN DEVELOPMENT AND NOT READY FOR USE.__

# pong

A CLI wrapper that reorganizes `pacman` operations for more intuitive use.

`pacman` is a great and reliable package manager,
but it is no secret that many find it difficult to use and memorize.
Pages like the
[`pacman` Rosetta](https://wiki.archlinux.org/title/Pacman/Rosetta)
should be indication enough that it can be rather confusing.

`pong` attempts to solve this by providing an intuitive flat set of commands for package management.
Commands perform one type of action,
and subsequent flags do not change the general action being performed.
The same type of action is not spread across several commands.
As an example, installation is done with `pong install` (or `pong i`),
while searching is done with `pong search` (`pong s`).
Searching through installed packages is done with `pong search --installed` (`pong s -i`).

Alternatively, `pong` can be used as a lookup tool for `pacman`;
passing the `-g/--generate` flag will make pong print the command
that it was _going_ to run rather than run it.

`pong` also supports AUR helpers.
The default behavior is to dispatch to the AUR helper for most operations,
but gate searching and installation behind a flag.
This ensures that AUR packages are properly managed,
while making the user aware when they install from the AUR.

## Dependencies

The only current dependencies are the packages `pacman` and `pacman-contrib`.

## Roadmap

[*] Basic user functionality.
[ ] AUR helper support.
    [*] Installation
    [*] Removal
    [*] Upgrade
    [*] Cleaning
    [*] Searching
    [ ] AUR-specific operations
    [ ] Listing
    [ ] Viewing
[ ] Completion
    [ ] Command completion.
    [ ] Content completion.
