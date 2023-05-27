How would the better pacman CLI look?

pong upgrade
pong search PACKAGE
pong install PACKAGE
pong remove PACKAGE
pong clean
pong tree
pong list PACKAGE
pong build PKGBUILD

-----
Okay, let's be systematic about this.

A package manager is a multitool that performs
an array of operations related to building, bundling, distributing, and installing software.
It's primary function is to allow users
to easily install a variety of software from a collection of remote repositories.

Software is bundled in "packages".
A package on its own is a collection of files and an installation script.
However, it should also be tagged with a list of dependencies (some optional),
other packages that need to be installed first in order for this one to be installed correctly.
Installing a package is performed by copying the files to the respective locations
and performing any additional steps laid out by the install script.

In a private storage, a package manager holds a local copy of the package registry.
This is the one that the user is querying when searching for packages to install.
Then, when told to install a package, it is looked up in the registry,
and all dependencies are installed before it.
When a package is installed,
it is downloaded and cached from a remote location specified in the registry.

To upgrade the registry, all packages must also be upgraded in turn.
This is all done by a single upgrade command.
So in short, the registry is replaced by an updated copy,
and all installed packages are reinstalled if necessary.
Since Arch is rolling release, all packages _must_ be updated.

-----
Subdivision of tasks

- Modify
    - Simulate the operation, without performing any changes.
    - Install a set of packages (and their dependencies).
    - Remove a set of packages (and any orphaned dependencies).
        - Also remove configuration files.
        - Remove all packages that depend on the removed packages.
    - Pin a set of packages.
    - Unpin a set of packages.
        - This should come with an "are you sure?" dialog when creating orphans.
    - Upgrade. Includes many bad operations:
        - Only synchronize the registry.
        - Only upgrade some dependencies.
    - Clean cache.
- Query
    - List packages matching...
        - Anything (all known packages).
        - Are installed.
        - Are in the registry.
        - Matching name.
        - Matching description.
        - Depends on X.
        - Is/isn't a dependency of any other package.
        - Is a dependency of X.
        - Provides file X (regex).
        - Is a dependency of X.
        - Is an orphan.
        - Is pinned/isn't pinned.
        - An update is available.
        - Belongs to a group.
    - List information about a package.
        - Name
        - Description
        - Version
        - Date
        - Files held
    - Show the dependency tree for one or more packages.
- Build

-----
What are some common operations that I have trouble performing?

- List the packages that provide binary X
  (but doesn't have it in the name or description).
- List the pinned packages that depend on a package.
- Search for the package that provides a file.
- Check whether a package is pinned or only a dependency.
- Check whether a package is installed from the registry or not.
