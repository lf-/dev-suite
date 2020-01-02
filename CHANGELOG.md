# v0.1

The change log was generated from the git commit titles and grouped by type of
commit or task completed with the commit. They are not in any particular order.
If you'd like to see the contents of the commits please take a look at the git
log! The major highlights of the first release include:

- Initial release of `ds`
- Initial release of `ticket`
- Initial release of `hooked`

Have fun taking the initial release out for a spin!

## Added Features

Make hooked and dev-suite git hooks cross platform
Add the ability to install dev-suite to ds tool
Add the ability to assign users to tickets
Switch from termion to crossterm for tui
Upgrade tui to allow commenting from it
Add the ability to add comments to tickets
Extend ds with config subcommand
Create configamajig to handle dev-suite configs
Upgrade ticket format from V0 to V1 to use UUIDs
Add a tui for ticket
Add logging output to ticket
Add logging output to hooked
Create dev-suite tool to orchestrate tooling
Add 'hooked init' test
Add hooked and empty inited hooks from the tool
Add ticket functionality to dev-suite (#3)

## Bug Fixes

Change ticket tui to fix thread panic on Windows
Fix init logic for `ds init`

## Chores and refactors

Add CHANGELOG and CONTRIBUTING and update README
Cleanup ds install and format hooked properly
Update reqwest and create a release profile
Make the toolchain use the latest stable rustc
Add licenses for dependencies to the project
Add a repo config and set self as maintainer
Refactor ticket to use common methods
Change pre-commit hook so that it works in fish
Upgrade Rust from 1.39 to 1.40
Add README and CODE_OF_CONDUCT
License all code under GPL-3.0
Make the pre-commit script pedantic and fix errors
Add commit message linting hook to the repo
Remove GitHub actions now that git hooks exist
Fix formatting and add checks to pre-commit hook
Setup CI for dev-suite (#1)
Initialize the dev-suite repo
Change ticket/Cargo.toml to use non * versions
Bump anyhow from 1.0.19 to 1.0.22
Move find_root function into the new shared crate
Setup the website skeleton with the kube theme

## Issues

Create ticket 'Create a tui for ticket'
Close 'Create a tui for ticket'
Create a ticket for a find_root function
Add tickets to the repo
