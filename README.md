[![Crates.io](https://img.shields.io/crates/v/git-interactive-rebase-tool.svg)][crates-io]
[![Packaging status](https://repology.org/badge/tiny-repos/git-interactive-rebase-tool.svg)](https://repology.org/project/git-interactive-rebase-tool/versions)
[![GitHub license](https://img.shields.io/badge/license-GPL-blue.svg)][license]
[![Coverage Status](https://coveralls.io/repos/github/MitMaro/git-interactive-rebase-tool/badge.svg?branch=master)](https://coveralls.io/github/MitMaro/git-interactive-rebase-tool?branch=master)

# Git Interactive Rebase Tool

Native cross-platform full-featured terminal-based [sequence editor][git-sequence-editor] for interactive rebase in Git 1.7.8+.

[![Git Interactive Rebase Tool](/docs/assets/images/girt-demo.gif?raw=true)](https://youtu.be/q3tzb-gQC0w)

## Table of Contents

* [Features](#features)
* [Install](./readme/install.md)
* [Setup](#setup)
* [Usage](#direct-usage)
* [Customization](./readme/customization.md)
* [Development](#development)
* [Related Projects](#related-projects)
* [License](#license)

## Features

### Cross-platform

Built and works on Linux, macOS, Windows, and many others.

### Set action

Easily set the action to `pick`, `squash`, `fixup`, `edit`, `reword` and `drop`.

![Basic operations](/docs/assets/images/girt-set-actions.gif?raw=true)

### Reorder rebase list

Reorder the action list with a single key press.

![Reorder items](/docs/assets/images/girt-reorder.gif?raw=true)

### Multiline modification

Change action and reorder multiple lines at once with visual mode.

![Visual mode](/docs/assets/images/girt-visual-mode.gif?raw=true)

### Toggle `break`s

![Toggle breaks](/docs/assets/images/girt-break.gif?raw=true)

### View commit details and diff

View the commit overview, and a full commit diff with a press of a key.

![Commit overview](/docs/assets/images/girt-commit-overview.gif?raw=true)

![Commit diff](/docs/assets/images/girt-commit-diff.gif?raw=true)

### Unicode and Emoji support

![Unicode support](/docs/assets/images/girt-unicode.png?raw=true)

![Emoji support](/docs/assets/images/girt-emoji.png?raw=true)

### Edit `exec` command

Easily edit the command that is run by an `exec` command.

![exec action command edit](/docs/assets/images/girt-edit.gif?raw=true)

### Edit in external editor

Need to do something in your Git editor? Quickly shell out to your editor, make a change and return to the tool.

![Shell out to editor](/docs/assets/images/girt-external-editor.gif?raw=true)

### Advanced Features

#### Modified line exec command

This optional feature allows for the injection of an `exec` action after modified lines, where modified is determined as a changed action, command, or reference. This can be used to amend commits to update references in the commit message or run a test suite only on modified commits.

To enable this option, set the `interactive-rebase-tool.postModifiedLineExecCommand` option, providing an executable or script.

```shell
git config --global interactive-rebase-tool.postModifiedLineExecCommand "/path/to/global/script"
```

Or using repository-specific configuration, for targeted scripts.

```shell
git config --global interactive-rebase-tool.postModifiedLineExecCommand "/path/to/repo/script"
```

The first argument provided to the script will always be the action performed. Then, depending on the action, the script will be provided a different set of arguments.

For `drop`, `fixup`, `edit`, `pick`, `reword` and `squash` actions, the script will additionally receive the original commit hash, for `exec` the original and new commands are provided, and for `label`, `reset`, `merge`, and `update-ref` the original label/reference and new label/reference are provided.

Full example of a resulting rebase todo file, assuming that `interactive-rebase-tool.postModifiedLineExecCommand` was set to `script.sh`.

```
# original line: label onto
label new-onto
exec script.sh "label" "onto" "new-onto"

# original line: reset onto
reset new-onto
exec script.sh "reset" "onto" "new-onto"

pick   a12345 My feature
# original line: pick b12345 My change
squash b12345 My change
exec script.sh "squash" "b12345"

# original line: label branch
label branch
exec script.sh "label" "branch" "new-branch"

# original line: exec command
exec new-command
exec script.sh "exec" "command" "new-command"
```

## Setup

### Most systems

```shell
git config --global sequence.editor interactive-rebase-tool
```

### Windows

#### Standard Command Prompt and Windows Terminal

```shell
git config --global sequence.editor "'C:/path/to/interactive-rebase-tool.exe'"
```

#### Git Bash

Git Bash requires the use of `winpty` in order to work correctly, so to set the editor use:

```shell
git config --global sequence.editor "winpty /c/path/to/interactive-rebase-tool.exe"
```

#### Notes

Windows before version 10 has [serious rendering issues with saturated darker colors](https://devblogs.microsoft.com/commandline/updating-the-windows-console-colors/), such as the blue color that is entirely illegible on modern displays. While it is possible to avoid using saturated colors, a better option is to update the theme using Microsoft's [ColorTool](https://github.com/Microsoft/Terminal/tree/master/src/tools/ColorTool).

### Temporary Override

You can temporarily use a different sequence editor by using the `GIT_SEQUENCE_EDITOR` environment variable:

```shell
GIT_SEQUENCE_EDITOR=emacs git rebase -i [<upstream> [<branch>]]
```

## Direct Usage

```shell
interactive-rebase-tool <rebase-todo-filepath>
interactive-rebase-tool --help
interactive-rebase-tool --version
```

### Getting Help

The tool has built-in help that can be accessed by using the `?` key.

### Common Default Key Bindings

Key bindings can be customized, see [configuration](readme/customization.md#key-bindings) for all key bindings and information on configuring.

| Key         | Mode        | Description                                |
|-------------|-------------|--------------------------------------------|
| `?`         | All         | Show help                                  |
| Up          | Normal/Diff | Move selection up                          |
| Down        | Normal/Diff | Move selection down                        |
| Page Up     | Normal/Diff | Move selection up five lines               |
| Page Down   | Normal/Diff | Move selection down five lines             |
| Home        | Normal/Diff | Move selection to start of list            |
| End         | Normal/Diff | Move selection to home of list             |
| `q`         | Normal/Diff | Abort interactive rebase                   |
| `Q`         | Normal/Diff | Immediately abort interactive rebase       |
| `w`         | Normal/Diff | Write interactive rebase file              |
| `W`         | Normal/Diff | Immediately write interactive rebase file  |
| `j`         | Normal/Diff | Move selected commit(s) down               |
| `k`         | Normal/Diff | Move selected commit(s) up                 |
| `b`         | Normal      | Toggle break action                        |
| `p`         | Normal/Diff | Set selected commit(s) to be picked        |
| `r`         | Normal/Diff | Set selected commit(s) to be reworded      |
| `e`         | Normal/Diff | Set selected commit(s) to be edited        |
| `s`         | Normal/Diff | Set selected commit(s) to be squashed      |
| `f`         | Normal/Diff | Set selected commit(s) to be fixed-up      |
| `d`         | Normal      | Set selected commit(s) to be dropped       |
| `d`         | Diff        | Show full commit diff                      |
| `E`         | Normal      | Edit the command of an editable action     |
| `v`         | Normal/Diff | Enter and exit visual mode (for selection) |
| `I`         | Normal      | Insert a new line                          |
| `Control+d` | Normal      | Duplicate the selected line                |
| `Delete`    | Normal/Diff | Remove selected lines                      |
| `!`         | Normal/Diff | Open todo file in external editor          |
| `Control+z` | Normal/Diff | Undo the previous change                   |
| `Control+y` | Normal/Diff | Redo the previously undone change          |
| `c`         | Normal/Diff | Show commit information                    |
| Down        | Diff        | Scroll view down                           |
| Up          | Diff        | Scroll view up                             |
| Left        | Diff        | Scroll view left                           |
| Right       | Diff        | Scroll view right                          |
| Home        | Diff        | Scroll view to the top                     |
| End         | Diff        | Scroll view to the end                     |
| PageUp      | Diff        | Scroll view a step up                      |
| PageDown    | Diff        | Scroll view a step down                    |

## Supported Platforms

### Linux

Supported on all Linux based distributions. The project is tested on Debian and Ubuntu, but should work on any standard Linux distribution. If the project is not working on your platform, please [open an issue](https://github.com/MitMaro/git-interactive-rebase-tool/issues/new).

The tool is tested in [Tilix](https://gnunn1.github.io/tilix-web/) and [Gnome Terminal](https://help.gnome.org/users/gnome-terminal/stable/).

### macOS

Supported on the latest version of macOS, though previous versions should work.

The tool is tested in [iTerm2](https://www.iterm2.com/) and [Terminal](https://support.apple.com/en-ca/guide/terminal/welcome/mac).

### Windows

Supported on the latest versions of Windows 10 and Windows 11.

The tool is tested on Windows 11 in [PowerShell](https://docs.microsoft.com/powershell/) and Command Prompt inside [Windows Console](https://docs.microsoft.com/windows/console/) and [Windows Terminal](https://docs.microsoft.com/windows/terminal/). The latest version of Git Bash provided from [Git for Windows](https://gitforwindows.org/) is also supported.

Windows Console and Git Bash are only minimally supported, and some features or graphical glitches may occur. Windows Console running in [legacy mode](https://docs.microsoft.com/windows/console/legacymode) is not supported.

[Cygwin](https://www.cygwin.com/) is not officially supported.

### Others

Other platforms are not officially supported. Some platforms have community support, and if you are having trouble getting the project working on your platform, please [open an issue](https://github.com/MitMaro/git-interactive-rebase-tool/issues/new).

## Development

### Install Rust

To start developing the project, you will need to [install Rust][install-rust], which can generally be done using [rustup].

### Setup

#### Cargo Make

This project uses [cargo-make](https://github.com/sagiegurari/cargo-make) as a task runner. To install:

```shell
cargo install --force cargo-make
```

#### Debian and derivatives

If you plan to build a release package you will need `pkg-config` and `liblzma-dev`. They can be installed using `apt`:

```shell
sudo apt install pkg-config liblzma-dev
```

### Build and run

To build or run the project, from the project root run:

```shell
# only build
cargo make build --release
# build and run
cargo run -- <path-to-git-rebase-todo-file>
```

Sample rebase todo files can be found in ./test/fixtures.

### Tests

Automated tests are available for all features and can be run with:

```shell
cargo make test
```

### Docs

API docs for the project are generated using Rust Doc:

```shell
cargo make docs
```

### Linting

An addition to the lints provided by `rustc`, this project uses `Clippy` to provide additional linting, run with:

```shell
cargo make lint
```

This will run lints using stable and nightly. The nightly lints may show errors, but will not result in a failure.

### Format

This project uses `rust-fmt` to provide a consistent format. A helpful script will ensure that all files are formatted correctly:

```shell
cargo make format
```

### Coverage

The project use [Tarpaulin](https://github.com/xd009642/tarpaulin) to generate coverage reports. Coverage reports are used to find gaps in tests. To generate the coverage report:

```shell
cargo make coverage
```

An addition to the report printed to the CLI, an HTML report can be found in the `coverage` directory.

### Release

#### Debian Packaging Building

```shell
cargo make build-deb
```

A deb file will be written to `target/debian/git-interactive-rebase-tool_*.deb`.

#### RPM Building

```shell
cargo make build-rpm
```

A rpm file will be written to `target/generate-rpm/git-interactive-rebase-tool-*.deb`.

#### Reproducible Builds

Providing a [`SOURCE_DATE_EPOCH`](https://reproducible-builds.org/specs/source-date-epoch/#idm55) environment variable with a valid Unix timestamp, defined in seconds, will ensure a reproducible build.

## Related Projects

* [rebase-editor](https://github.com/sjurba/rebase-editor) is a very similar project written in Node.js.
* [cj-git-patchtool](https://github.com/pflanze/cj-git-patchtool) is another tool for doing `git rebase` en masse.
* [andrewshadura/git-crecord](https://github.com/andrewshadura/git-crecord) provides interactive selection of changes to a commit.

## License

Copyright © 2017-2024 Tim Oram and Contributors

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

A copy of the GNU General Public License can be found in the file
[COPYING](COPYING).

See [Third Party Licenses](https://gitrebasetool.mitmaro.ca/licenses.html) for licenses of the third-party libraries used by this project.

[appveyor-build]:https://ci.appveyor.com/project/MitMaro/git-interactive-rebase-tool/branch/master
[cargo]:https://github.com/rust-lang/cargo
[crates-io]:https://crates.io/crates/git-interactive-rebase-tool
[git-sequence-editor]:https://git-scm.com/docs/git-config#Documentation/git-config.txt-sequenceeditor
[install-rust]:https://doc.rust-lang.org/book/getting-started.html
[license]:https://raw.githubusercontent.com/MitMaro/git-interactive-rebase-tool/master/LICENSE
[releases]:https://github.com/MitMaro/git-interactive-rebase-tool/releases
[rustup]:https://www.rustup.rs/
[travis-build]:https://travis-ci.org/MitMaro/git-interactive-rebase-tool
