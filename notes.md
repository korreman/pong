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
