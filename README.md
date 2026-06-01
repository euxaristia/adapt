# adapt

A [paru](https://github.com/morganamilo/paru)-like wrapper for `apt` on Ubuntu/Debian, written in Go.

Just like `paru` wraps `pacman` with a nicer interface, `adapt` wraps `sudo apt` with familiar AUR-helper-style commands.

## Features

- **Zero-args default**: Running `adapt` with no arguments runs `sudo apt update && sudo apt upgrade`
- **Pacman-style flags**: Use familiar flags like `-Syu`, `-S`, `-R`, `-Ss`, `-Si`, `-Q`, `-Qu`
- **Apt-style subcommands**: Also supports `install`, `remove`, `search`, `show`, `autoremove`, etc.
- **Read-only commands without sudo**: `search`, `show`, `list` run without elevating privileges

## Installation

### From source

```bash
git clone https://github.com/euxaristia/adapt.git
cd adapt
go build -o adapt .
sudo cp adapt /usr/local/bin/
```

## Usage

### Default behavior

```bash
adapt
# Runs: sudo apt update && sudo apt upgrade
```

### Pacman-style flags

| Flag | Description | Equivalent apt command |
|------|-------------|------------------------|
| `adapt -Syu` | Update and upgrade | `sudo apt update && sudo apt upgrade` |
| `adapt -Sy` | Update package lists | `sudo apt update` |
| `adapt -Su` | Upgrade packages | `sudo apt upgrade` |
| `adapt -S <pkg>` | Install package(s) | `sudo apt install <pkg>` |
| `adapt -R <pkg>` | Remove package(s) | `sudo apt remove <pkg>` |
| `adapt -Rs <pkg>` | Remove package(s) and unused deps | `sudo apt remove <pkg> && sudo apt autoremove` |
| `adapt -Ss <query>` | Search for packages | `apt search <query>` |
| `adapt -Si <pkg>` | Show package info | `apt show <pkg>` |
| `adapt -Q` | List installed packages | `dpkg-query -W` |
| `adapt -Qs <query>` | Search installed packages | `dpkg-query -W` (filtered) |
| `adapt -Qu` | List available upgrades | `apt list --upgradable` |
| `adapt -Sc` | Clean package cache | `sudo apt clean` |

### Apt-style subcommands

| Command | Description | Equivalent apt command |
|---------|-------------|------------------------|
| `adapt update` | Update package lists | `sudo apt update` |
| `adapt upgrade` | Upgrade packages | `sudo apt upgrade` |
| `adapt full-upgrade` | Full upgrade (dist-upgrade) | `sudo apt full-upgrade` |
| `adapt install <pkg>` | Install package(s) | `sudo apt install <pkg>` |
| `adapt remove <pkg>` | Remove package(s) | `sudo apt remove <pkg>` |
| `adapt purge <pkg>` | Purge package(s) | `sudo apt purge <pkg>` |
| `adapt search <query>` | Search for packages | `apt search <query>` |
| `adapt show <pkg>` | Show package info | `apt show <pkg>` |
| `adapt autoremove` | Remove unused dependencies | `sudo apt autoremove` |
| `adapt clean` | Clean package cache | `sudo apt clean` |
| `adapt list [filter]` | List installed packages | `dpkg-query -W` |
| `adapt edit-sources` | Edit sources.list | `sudo $EDITOR /etc/apt/sources.list` |

## Examples

```bash
# Update and upgrade everything
adapt

# Install a package
adapt -S htop
# or
adapt install htop

# Search for a package
adapt -Ss neovim
# or
adapt search neovim

# Remove a package
adapt -R htop
# or
adapt remove htop

# Show info about a package
adapt -Si htop
# or
adapt show htop

# Check for available upgrades
adapt -Qu
```

## Requirements

- Go 1.22+ (to build from source)
- `apt` package manager (Ubuntu/Debian)
- `sudo` privileges for write operations
