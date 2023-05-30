__THIS TOOL IS IN DEVELOPMENT AND NOT READY FOR USE.__

# pong
A simplifying CLI-wrapper for Arch Linux package management.

The package manager `pacman` is great at what it does.
However, I cannot for the life of me remember how to tell it what to do.
I'm sure that the CLI makes sense from some perspective,
but I've decided to write a CLI wrapper that makes sense to me.

The goals for this wrapper are:

1. Memorability.
It should be easy to commit commands and options to memory,
even after using them once.
2. Discoverability.
It should be easy to find the functionality you are looking for,
and to discover new functionality that you haven't considered.
3. Good defaults.
The correct, preferred, or common ways of using the tool shouldn't require any configuration.
4. Safety.
It should be difficult to accidentally perform unsafe actions.
5. Anti-inconvenience.
Convenience and simplicity are vague goals
that often leads us to ignore latent and/or inherent complexity.
I prefer the weaker goals of not inconveniencing the user
and not introducing _unnecessary_ complexity.

## Dependencies

The only current dependencies are the packages `pacman` and `pacman-contrib`.
