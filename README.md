# dev-suite

dev-suite is a set of tools designed to redistribute distributed work. Our code
has been locked into centralized services like GitHub and Gitlab. They provide
nice things like an issue tracker and PRs as an integrated service. If we can
put that into the git repo itself we'd be free to host our code wherever and
have our code and it's project management share a common history rather than
being locked into a service and divorcing the context of the important things
that shape what code gets written.

Currently dev-suite has two tools:
- Hooked, a git hooks manager to allow cross platform git hooks that can help
  enable things like trunk based development and allow CI in a local
  environment
- Ticket, an on disk ticket manager and viewer to allow issues to live inside of
  the repo and travel with it

## Installation

### Dependencies

#### OSX
- git

#### Linux
- git

#### Windows
- git for Windows installed to the default path for hooked to work

#### Optional deps
- Ruby
- Python
- Bash (included with git for Windows)

Make sure you have these on your path somewhere for the git hooks to work
properly in hooked

### Recommended install method
1. Grab a binary for the ds tool for your platform and place it somewhere on
   your PATH
  - [Windows](https://dev-suite-spaces.nyc3.digitaloceanspaces.com/windows/ds.exe)
  - [Linux](https://dev-suite-spaces.nyc3.digitaloceanspaces.com/linux/ds)
  - [OSX](https://dev-suite-spaces.nyc3.digitaloceanspaces.com/osx/ds)
2. Run `ds install` which will install `ticket` and `hooked` to:
  - Windows: `C:\\Users\YourUser\AppData\Local\dev-suite`
  - Linux: `$XDG_BIN_HOME` or `$XDG_DATA_HOME/../bin` or `$HOME/.local/bin`
  - OSX: `/usr/local/bin`
  On Windows `ds install` will add the install path to your PATH with the `setx`
  command. You might need to log out or restart your computer to see the
  desired effect of not needing to type the path to the executable to run it.
3. Run `ds config self init` to initialize a user config on the system. Failing
   to do so will likely cause unexpected errors.

### Manually compile
In the event the above doesn't work for some reason or because of a bug (please
file an issue and see CONTRIBUTING.md on how to do so) you can install these
tools manually.

1. Make sure you have the stable Rust compiler and Cargo installed we recommend
   doing so via [rustup](https://rustup.rs/)
2. Clone this repo
3. `cd` into the repo
4. `cargo install --path .`
5. `cargo install --path hooked`
6. `cargo install --path ticket`
7. Run `ds config self init` to initialize a user config on the system. Failing
   to do so will likely cause unexpected errors.

## Usage

### Initializing a repo to use dev-suite
While each tool has it's own init command we recommend running `ds init` inside
of a repo that you want to use these tools. You can choose the ones that you
want to use from the command prompt and it will also initialize the repo with
it's own repo config so this is probably the best way to do so.

### ds

`ds` is the main orchestration tool for setting things up with dev-suite. As
this is not the main driver beyond setup it only has a few commands:

```bash
# Initialize a repo to use dev-suite and it's tools
ds init

# Install dev-suite's tools onto your computer
ds install

# Config commands for the user and repo

## Create a dev-suite repo config in a repo
ds config repo init

## Add yourself as a maintainer to the repo config
ds config repo add me

## Pretty print the repo config to the terminal
ds config repo show

## Create a dev-suite user config for the system
ds config user init "Display Name"

## Pretty print the user config to the terminal
ds config user show
```

## Hooked
`hooked` is a dev-suite tool used to create git hooks for your repo to travel
with it and to link them to `.git/hooks` on a fresh clone of it.

```bash
# Initialize a repo to use hooked if it was not initialized with it when using
# `ds init`

## Initialize the repo to use bash for git hooks
hooked init bash

## Initialize the repo to use ruby for git hooks
hooked init ruby

## Initialize the repo to use python for git hooks
hooked init python

# Link pre-existing dev-suite git hooks
hooked link
```

## Ticket

`ticket` is a dev-suite tool used to create, update, view, and manage
tickets for your code base.

```bash
# Initialize a repo to use ticket if it was not initialized with it when using
# `ds init`
ticket init

# Open up a new ticket
ticket new

# Close a ticket
ticket close <TICKET-UUID>

# Comment on a ticket
ticket comment <TICKET-UUID> <MESSAGE>

# Show a ticket on the commandline
ticket show <TICKET-UUID>

# Assign a ticket to yourself
ticket assign <TICKET-UUID> to me

# Assign a ticket to someone else
ticket assign <TICKET-UUID> to them <USER-UUID> <NAME>

# Migrate old versions of tickets to the newer versions this does nothing for now
# unless you checkout the codebase from a pre v0.1 release
ticket migrate

# Open up the tui to look at tickets and comment on them
ticket

```

## Contributing
See CONTRIBUTING.md for more details

## Changelog
See CHANGELOG.md for more details

## Code of Conduct
The Code of Conduct is strictly enforced. See CODE_OF_CONDUCT.md for more
details.

## Opening PRs and Issues on public mirrors
GitHub, Gitlab, and Bitbucket are mirrored repos and all PRs and Issues will be
closed. The point of dev-suite is to not depend on the value add of these
services. These mirror exist only to provide a public way to clone the source
code.

## Blog Posts
- [Redistributing Distributed Work](https://blog.mgattozzi.dev/redistributing-distributed-work/)

## License
All code and contributions are licensed under the GNU Public License v3.0
See LICENSE.md for more details. While this code does use the GPL we don't
condone the actions of Richard Stallman or the FSF in it's protection of him.
