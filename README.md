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
even after using them only once.
2. Discoverability.
It should be easy to find the functionality you are looking for
and to discover new functionality that you didn't know about.
3. Good defaults.
The correct, preferred, or common ways of using the tool shouldn't require any configuration.
4. Safety.
It should be difficult to accidentally perform unsafe actions.
5. Anti-inconvenience.
The tool should feel convenient,
but not at the cost of other goals.
Hence, it's more about it not being inconvenient to use.


## Dependencies

The only current dependencies are the packages `pacman` and `pacman-contrib`.
