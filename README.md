# Toolbox
These are the personal command-line tools I've put together over the years.
Most are wrappers or aliases for commands I use often. A few, like `rmem`,
`rchmod`, and `wrap`, do a little more of their own work.

It's very much my own toolbox. The names, defaults, and behavior follow my
habits and the way I set up my machines. Some of these might be useful to you
too, but expect a few choices that make more sense on my computer than yours.

## Building
With Rust and Cargo installed, run this from the repository root:

```sh
cargo run -p toolbox-build --
```

That builds every command supported on your current OS and architecture and
puts the executables in `dist/OS-ARCH/bin/`, such as `dist/linux-x86_64/bin/`.
Add that folder to your `PATH`, or copy the executables wherever you keep your
tools. Use `--output PATH` to choose a different base directory:

```sh
cargo run -p toolbox-build -- --output ./bin
```

## Commands
| Command | What it does | Platforms |
| --- | --- | --- |
| `7z` | Runs `7zz`. | MacOS |
| `docker` | Runs `podman`. | All |
| `firewallctl` | Runs `firewall-cmd`. | Linux |
| `grean` | Runs `grean` from `lib/grean-2.0.10/bin/` beside the wrapper. I update that path when needed. | All |
| `hide` | Marks paths hidden on Windows; adds a leading dot to their names on Unix. | All |
| `open` | Opens paths with their default application. Defaults to the current directory. | Linux, Windows |
| `py`, `python` | Runs `python3`. | All |
| `pyclean` | Removes Python cache directories and `.pyc` and `.pyo` files beneath the supplied roots. | All |
| `rchmod` | Changes file and directory modes recursively, with separate `--file` and `--dir` modes. | All |
| `renet` | Runs `ipconfig /flushdns`, `/release`, and `/renew`, even if an earlier step fails. | Windows |
| `reviso` | Runs `reviso` from `lib/reviso-2.2.10/bin/` beside the wrapper. I update that path when needed. | All |
| `rmem` | Removes empty directories recursively, preserving each supplied root. | All |
| `rmmd` | Removes metadata created by native file explorers. | All |
| `sqlite` | Runs `sqlite3`. | All |
| `unhide` | Clears the hidden attribute on Windows; removes one leading dot from names on Unix. | All |
| `wrap` | Wraps standard input at word boundaries. Defaults to 120 columns. | All |

The wrappers forward arguments, help, version, standard streams, and exit codes
to their underlying commands. The other tools have `--help` and `--version` of
their own. `rchmod`, `rmem`, and `rmmd` also have dry-run and verbose options.

You'll need the programs that the wrappers call installed separately. `pyclean`
needs `fd` and `rm`; `rchmod` needs `chmod`. I usually have MSYS on Windows for
those Unix tools. `wrap` leaves long words intact, so some lines can exceed the
requested width.
