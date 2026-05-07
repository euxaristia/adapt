# adapter

A [paru](https://github.com/morganamilo/paru)-like wrapper for `apt` on Ubuntu/Debian, written in Rust.

Just like `paru` wraps `pacman` with a nicer interface, `adapter` wraps `sudo apt` with familiar AUR-helper-style commands.

## Features

- **Zero-args default**: Running `adapter` with no arguments runs `sudo apt update && sudo apt upgrade`
- **Pacman-style flags**: Use familiar flags like `-Syu`, `-S`, `-R`, `-Ss`, `-Si`, `-Q`, `-Qu`
- **Apt-style subcommands**: Also supports `install`, `remove`, `search`, `show`, `autoremove`, etc.
- **Read-only commands without sudo**: `search`, `show`, `list` run without elevating privileges

## Installation

### From source

```bash
git clone https://github.com/euxaristia/adapter.git
cd adapter
cargo build --release
sudo cp target/release/adapter /usr/local/bin/
```

## Usage

### Default behavior

```bash
adapter
# Runs: sudo apt update && sudo apt upgrade
```

### Pacman-style flags

| Flag | Description | Equivalent apt command |
|------|-------------|------------------------|
| `adapter -Syu` | Update and upgrade | `sudo apt update && sudo apt upgrade` |
| `adapter -Sy` | Update package lists | `sudo apt update` |
| `adapter -Su` | Upgrade packages | `sudo apt upgrade` |
| `adapter -S <pkg>` | Install package(s) | `sudo apt install <pkg>` |
| `adapter -R <pkg>` | Remove package(s) | `sudo apt remove <pkg>` |
| `adapter -R <pkg> --recursive` | Remove package(s) and unused deps | `sudo apt remove <pkg> && sudo apt autoremove` |
| `adapter -Ss <query>` | Search for packages | `apt search <query>` |
| `adapter -Si <pkg>` | Show package info | `apt show <pkg>` |
| `adapter -Q` | List installed packages | `dpkg-query -W` |
| `adapter -Qs <query>` | Search installed packages | `dpkg-query -W` (filtered) |
| `adapter -Qu` | List available upgrades | `apt list --upgradable` |
| `adapter -Sc` | Clean package cache | `sudo apt clean` |

### Apt-style subcommands

| Command | Description | Equivalent apt command |
|---------|-------------|------------------------|
| `adapter update` | Update package lists | `sudo apt update` |
| `adapter upgrade` | Upgrade packages | `sudo apt upgrade` |
| `adapter full-upgrade` | Full upgrade (dist-upgrade) | `sudo apt full-upgrade` |
| `adapter install <pkg>` | Install package(s) | `sudo apt install <pkg>` |
| `adapter remove <pkg>` | Remove package(s) | `sudo apt remove <pkg>` |
| `adapter purge <pkg>` | Purge package(s) | `sudo apt purge <pkg>` |
| `adapter search <query>` | Search for packages | `apt search <query>` |
| `adapter show <pkg>` | Show package info | `apt show <pkg>` |
| `adapter autoremove` | Remove unused dependencies | `sudo apt autoremove` |
| `adapter clean` | Clean package cache | `sudo apt clean` |
| `adapter list [filter]` | List installed packages | `dpkg-query -W` |
| `adapter edit-sources` | Edit sources.list | `sudo $EDITOR /etc/apt/sources.list` |

## Examples

```bash
# Update and upgrade everything
adapter

# Install a package
adapter -S htop
# or
adapter install htop

# Search for a package
adapter -Ss neovim
# or
adapter search neovim

# Remove a package
adapter -R htop
# or
adapter remove htop

# Show info about a package
adapter -Si htop
# or
adapter show htop

# Check for available upgrades
adapter -Qu
```

## Requirements

- Rust 1.70+ (to build from source)
- `apt` package manager (Ubuntu/Debian)
- `sudo` privileges for write operations
