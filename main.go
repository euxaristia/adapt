package main

import (
	"bytes"
	"errors"
	"fmt"
	"io"
	"os"
	"os/exec"
	"strings"
)

const version = "0.1.0"

var errExitOK = errors.New("exit requested")

type pacmanFlags struct {
	sync      bool
	remove    bool
	query     bool
	search    bool
	info      bool
	refresh   bool
	upgrade   bool
	clean     bool
	recursive bool
	targets   []string
}

func main() {
	if err := run(os.Args[1:]); err != nil {
		if errors.Is(err, errExitOK) {
			return
		}
		fmt.Fprintf(os.Stderr, "adapt: %v\n", err)
		os.Exit(1)
	}
}

func run(args []string) error {
	args, noconfirm, err := parseGlobalOptions(args)
	if err != nil {
		return err
	}

	if len(args) == 0 {
		return runDefault(noconfirm)
	}

	if isSubcommand(args[0]) {
		return runSubcommand(args[0], args[1:], noconfirm)
	}

	flags, err := parsePacmanFlags(args)
	if err != nil {
		return err
	}
	return runPacmanFlags(flags, noconfirm)
}

func parseGlobalOptions(args []string) ([]string, bool, error) {
	var kept []string
	noconfirm := false

	for i := 0; i < len(args); i++ {
		switch args[i] {
		case "--":
			kept = append(kept, args[i:]...)
			return kept, noconfirm, nil
		case "--noconfirm":
			noconfirm = true
		case "-h", "--help":
			printHelp(os.Stdout)
			return nil, noconfirm, errExitOK
		case "-V", "--version":
			fmt.Fprintf(os.Stdout, "adapt %s\n", version)
			return nil, noconfirm, errExitOK
		default:
			kept = append(kept, args[i])
		}
	}

	return kept, noconfirm, nil
}

func isSubcommand(name string) bool {
	switch name {
	case "update", "upgrade", "full-upgrade", "install", "remove", "purge",
		"search", "show", "autoremove", "clean", "list", "edit-sources":
		return true
	default:
		return false
	}
}

func runSubcommand(name string, args []string, noconfirm bool) error {
	args = stripLeadingSeparator(args)

	switch name {
	case "update":
		if err := requireNoArgs(name, args); err != nil {
			return err
		}
		return updatePackages(noconfirm)
	case "upgrade":
		full, err := parseUpgradeOptions(args)
		if err != nil {
			return err
		}
		if full {
			return fullUpgradePackagesWithExtras(noconfirm)
		}
		return upgradePackagesWithExtras(noconfirm)
	case "full-upgrade":
		if err := requireNoArgs(name, args); err != nil {
			return err
		}
		return fullUpgradePackagesWithExtras(noconfirm)
	case "install":
		if len(args) == 0 {
			return errors.New("no packages specified for installation")
		}
		return installPackages(args, noconfirm)
	case "remove":
		if len(args) == 0 {
			return errors.New("no packages specified for removal")
		}
		return removePackages(args, false, noconfirm)
	case "purge":
		if len(args) == 0 {
			return errors.New("no packages specified for purge")
		}
		return removePackages(args, true, noconfirm)
	case "search":
		if len(args) == 0 {
			return errors.New("no search query specified")
		}
		return searchPackages(strings.Join(args, " "))
	case "show":
		if len(args) != 1 {
			return errors.New("show requires exactly one package")
		}
		return showPackage(args[0])
	case "autoremove":
		if err := requireNoArgs(name, args); err != nil {
			return err
		}
		return autoremovePackages(noconfirm)
	case "clean":
		if err := requireNoArgs(name, args); err != nil {
			return err
		}
		return cleanPackageCache()
	case "list":
		filter := ""
		if len(args) > 0 {
			filter = strings.Join(args, " ")
		}
		return listInstalledPackages(filter)
	case "edit-sources":
		if err := requireNoArgs(name, args); err != nil {
			return err
		}
		return editSources()
	default:
		return fmt.Errorf("unknown command: %s", name)
	}
}

func parseUpgradeOptions(args []string) (bool, error) {
	full := false

	for _, arg := range args {
		switch arg {
		case "-f", "--full":
			full = true
		default:
			return false, fmt.Errorf("unknown upgrade option: %s", arg)
		}
	}

	return full, nil
}

func requireNoArgs(command string, args []string) error {
	if len(args) == 0 {
		return nil
	}
	return fmt.Errorf("%s does not accept arguments: %s", command, strings.Join(args, " "))
}

func stripLeadingSeparator(args []string) []string {
	if len(args) > 0 && args[0] == "--" {
		return args[1:]
	}
	return args
}

func parsePacmanFlags(args []string) (pacmanFlags, error) {
	var flags pacmanFlags

	for i := 0; i < len(args); i++ {
		arg := args[i]

		if arg == "--" {
			flags.targets = append(flags.targets, args[i+1:]...)
			break
		}

		if strings.HasPrefix(arg, "--") {
			if err := setLongFlag(&flags, arg); err != nil {
				return flags, err
			}
			continue
		}

		if strings.HasPrefix(arg, "-") && arg != "-" {
			if err := setShortFlags(&flags, strings.TrimPrefix(arg, "-")); err != nil {
				return flags, err
			}
			continue
		}

		flags.targets = append(flags.targets, arg)
	}

	if flags.remove && flags.search {
		// Pacman uses -Rs for recursive removal. Lowercase s remains search elsewhere.
		flags.search = false
		flags.recursive = true
	}

	if countOperations(flags) > 1 {
		return flags, errors.New("choose only one operation: -S/--sync, -R/--remove, or -Q/--query")
	}

	return flags, nil
}

func setLongFlag(flags *pacmanFlags, arg string) error {
	switch arg {
	case "--sync":
		flags.sync = true
	case "--remove":
		flags.remove = true
	case "--query":
		flags.query = true
	case "--search":
		flags.search = true
	case "--info":
		flags.info = true
	case "--refresh":
		flags.refresh = true
	case "--upgrade":
		flags.upgrade = true
	case "--clean":
		flags.clean = true
	case "--recursive":
		flags.recursive = true
	default:
		return fmt.Errorf("unknown option: %s", arg)
	}
	return nil
}

func setShortFlags(flags *pacmanFlags, letters string) error {
	for _, letter := range letters {
		switch letter {
		case 'S':
			flags.sync = true
		case 'R':
			flags.remove = true
		case 'Q':
			flags.query = true
		case 's':
			flags.search = true
		case 'i':
			flags.info = true
		case 'y':
			flags.refresh = true
		case 'u':
			flags.upgrade = true
		case 'c':
			flags.clean = true
		default:
			return fmt.Errorf("unknown option: -%c", letter)
		}
	}
	return nil
}

func countOperations(flags pacmanFlags) int {
	count := 0
	if flags.sync {
		count++
	}
	if flags.remove {
		count++
	}
	if flags.query {
		count++
	}
	return count
}

func runPacmanFlags(flags pacmanFlags, noconfirm bool) error {
	if noOperation(flags) && len(flags.targets) == 0 {
		return runDefault(noconfirm)
	}

	if flags.sync {
		return runSync(flags, noconfirm)
	}
	if flags.remove {
		return runRemove(flags, noconfirm)
	}
	if flags.query {
		return runQuery(flags)
	}
	if flags.search {
		if len(flags.targets) == 0 {
			return errors.New("no search query specified. Usage: adapt -Ss <query>")
		}
		return searchPackages(strings.Join(flags.targets, " "))
	}
	if flags.info {
		if len(flags.targets) == 0 {
			return errors.New("no package specified. Usage: adapt -Si <package>")
		}
		return showPackage(flags.targets[0])
	}
	if flags.clean {
		return cleanPackageCache()
	}

	fmt.Println(":: adapt - paru, but for apt")
	fmt.Println(":: Try 'adapt --help' for usage information.")
	return nil
}

func noOperation(flags pacmanFlags) bool {
	return !flags.sync &&
		!flags.remove &&
		!flags.query &&
		!flags.search &&
		!flags.info &&
		!flags.clean &&
		!flags.refresh &&
		!flags.upgrade
}

func runSync(flags pacmanFlags, noconfirm bool) error {
	hasTargets := len(flags.targets) > 0

	if flags.search && hasTargets {
		return searchPackages(strings.Join(flags.targets, " "))
	}
	if flags.info && hasTargets {
		return showPackage(flags.targets[0])
	}
	if flags.clean {
		return cleanPackageCache()
	}
	if flags.refresh && flags.upgrade && !hasTargets {
		return runUpdateUpgradeAndExtras(noconfirm)
	}
	if flags.refresh && !hasTargets {
		return updatePackages(noconfirm)
	}
	if flags.upgrade && !hasTargets {
		return upgradePackagesWithExtras(noconfirm)
	}
	if hasTargets {
		if flags.refresh {
			if err := updatePackages(noconfirm); err != nil {
				return err
			}
		}
		if flags.upgrade {
			if err := upgradePackagesWithExtras(noconfirm); err != nil {
				return err
			}
		}
		return installPackages(flags.targets, noconfirm)
	}
	if flags.refresh {
		return updatePackages(noconfirm)
	}
	if flags.upgrade {
		return upgradePackagesWithExtras(noconfirm)
	}

	fmt.Println(":: Sync operation. Use -Syu to update & upgrade, -S <pkg> to install, -Ss <query> to search.")
	return nil
}

func runRemove(flags pacmanFlags, noconfirm bool) error {
	if len(flags.targets) == 0 {
		return errors.New("no packages specified for removal. Usage: adapt -R <package>")
	}

	if err := removePackages(flags.targets, false, noconfirm); err != nil {
		return err
	}
	if flags.recursive {
		return autoremovePackages(noconfirm)
	}
	return nil
}

func runQuery(flags pacmanFlags) error {
	hasTargets := len(flags.targets) > 0

	if flags.info && hasTargets {
		return showPackage(flags.targets[0])
	}
	if flags.search && hasTargets {
		return listInstalledPackages(strings.Join(flags.targets, " "))
	}
	if flags.upgrade {
		return listUpgrades()
	}
	if hasTargets {
		return listInstalledPackages(strings.Join(flags.targets, " "))
	}
	return listInstalledPackages("")
}

func runDefault(noconfirm bool) error {
	fmt.Println(":: adapt - paru, but for apt")
	fmt.Println(":: No operation specified, running update && upgrade...")
	fmt.Println()
	return runUpdateUpgradeAndExtras(noconfirm)
}

func runUpdateUpgradeAndExtras(noconfirm bool) error {
	if err := updatePackages(noconfirm); err != nil {
		return err
	}
	if err := upgradePackages(noconfirm); err != nil {
		return err
	}
	return updateSecondaryPackages(noconfirm)
}

func upgradePackagesWithExtras(noconfirm bool) error {
	if err := upgradePackages(noconfirm); err != nil {
		return err
	}
	return updateSecondaryPackages(noconfirm)
}

func fullUpgradePackagesWithExtras(noconfirm bool) error {
	if err := fullUpgradePackages(noconfirm); err != nil {
		return err
	}
	return updateSecondaryPackages(noconfirm)
}

func updatePackages(noconfirm bool) error {
	fmt.Println(":: Updating package lists...")
	return runApt([]string{"update"}, true, noconfirm)
}

func upgradePackages(noconfirm bool) error {
	fmt.Println(":: Upgrading packages...")
	return runApt([]string{"upgrade"}, true, noconfirm)
}

func fullUpgradePackages(noconfirm bool) error {
	fmt.Println(":: Performing full upgrade...")
	return runApt([]string{"full-upgrade"}, true, noconfirm)
}

func installPackages(packages []string, noconfirm bool) error {
	if len(packages) == 0 {
		return errors.New("no packages specified for installation")
	}

	fmt.Printf(":: Installing package(s): %s\n", strings.Join(packages, " "))
	args := append([]string{"install", "--"}, packages...)
	return runApt(args, true, noconfirm)
}

func removePackages(packages []string, purge bool, noconfirm bool) error {
	if len(packages) == 0 {
		return errors.New("no packages specified for removal")
	}

	action := "remove"
	label := "Removing"
	if purge {
		action = "purge"
		label = "Purging"
	}

	fmt.Printf(":: %s packages: %s\n", label, strings.Join(packages, " "))
	args := append([]string{action, "--"}, packages...)
	return runApt(args, true, noconfirm)
}

func autoremovePackages(noconfirm bool) error {
	fmt.Println(":: Removing unused dependencies...")
	return runApt([]string{"autoremove"}, true, noconfirm)
}

func searchPackages(query string) error {
	return runApt([]string{"search", query}, false, false)
}

func showPackage(pkg string) error {
	return runApt([]string{"show", pkg}, false, false)
}

func cleanPackageCache() error {
	fmt.Println(":: Cleaning package cache...")
	return runApt([]string{"clean"}, true, false)
}

func listUpgrades() error {
	fmt.Println(":: Checking for available upgrades...")
	return runApt([]string{"list", "--upgradable"}, false, false)
}

func listInstalledPackages(filter string) error {
	cmd := exec.Command("dpkg-query", "-W", "-f=${Package}\t${Version}\n")
	var stderr bytes.Buffer
	cmd.Stderr = &stderr

	output, err := cmd.Output()
	if err != nil {
		message := strings.TrimSpace(stderr.String())
		if message == "" {
			message = err.Error()
		}
		return fmt.Errorf("dpkg-query failed: %s", message)
	}

	filter = strings.ToLower(filter)
	for _, line := range strings.Split(strings.TrimSuffix(string(output), "\n"), "\n") {
		if line == "" {
			continue
		}
		if filter == "" || strings.Contains(strings.ToLower(line), filter) {
			fmt.Println(line)
		}
	}
	return nil
}

func editSources() error {
	editor := os.Getenv("EDITOR")
	if editor == "" {
		editor = "nano"
	}

	shellCommand := editor + " /etc/apt/sources.list"
	if isRoot() {
		return runCommand("editor", "sh", []string{"-c", shellCommand})
	}
	return runCommand("editor", "sudo", []string{"sh", "-c", shellCommand})
}

func updateSecondaryPackages(noconfirm bool) error {
	if err := updateFlatpaks(noconfirm); err != nil {
		return err
	}
	return updateSnaps()
}

func updateFlatpaks(noconfirm bool) error {
	if !hasFlatpak() {
		return nil
	}

	fmt.Println(":: Updating flatpak packages...")
	args := []string{"update"}
	if noconfirm {
		args = append(args, "-y")
	}
	return runCommand("flatpak update", "flatpak", args)
}

func updateSnaps() error {
	if !hasSnap() {
		return nil
	}

	fmt.Println(":: Updating snap packages...")
	if isRoot() {
		return runCommand("snap refresh", "snap", []string{"refresh"})
	}
	return runCommand("snap refresh", "sudo", []string{"snap", "refresh"})
}

func hasFlatpak() bool {
	cmd := exec.Command("flatpak", "--version")
	cmd.Stdout = io.Discard
	cmd.Stderr = io.Discard
	return cmd.Run() == nil
}

func hasSnap() bool {
	cmd := exec.Command("snap", "--version")
	cmd.Stdout = io.Discard
	cmd.Stderr = io.Discard
	return cmd.Run() == nil
}

func runApt(args []string, useSudo bool, noconfirm bool) error {
	aptArgs := args
	if noconfirm {
		aptArgs = append([]string{"-y"}, aptArgs...)
	}

	if useSudo && !isRoot() {
		return runCommand("apt command", "sudo", append([]string{"apt"}, aptArgs...))
	}
	return runCommand("apt command", "apt", aptArgs)
}

func runCommand(label string, name string, args []string) error {
	cmd := exec.Command(name, args...)
	cmd.Stdin = os.Stdin
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr

	if err := cmd.Run(); err != nil {
		var exitErr *exec.ExitError
		if errors.As(err, &exitErr) {
			return fmt.Errorf("%s failed with status: %s", label, exitErr.ProcessState)
		}
		return fmt.Errorf("failed to execute %s: %w", label, err)
	}
	return nil
}

func isRoot() bool {
	return os.Geteuid() == 0
}

func printHelp(out io.Writer) {
	fmt.Fprintf(out, `adapt %s
paru-like wrapper for apt

Usage:
  adapt [--noconfirm]
  adapt [--noconfirm] <command> [args]
  adapt [--noconfirm] <pacman-style flags> [targets]

Commands:
  update                update package lists
  upgrade [-f|--full]   upgrade packages, or full-upgrade with --full
  full-upgrade          perform apt full-upgrade
  install <pkg...>      install packages
  remove <pkg...>       remove packages
  purge <pkg...>        purge packages
  search <query...>     search available packages
  show <pkg>            show package details
  autoremove            remove unused dependencies
  clean                 clean package cache
  list [filter...]      list installed packages
  edit-sources          edit /etc/apt/sources.list with $EDITOR

Pacman-style flags:
  -Syu                  update, upgrade, then update flatpaks/snaps if available
  -Sy                   update package lists
  -Su                   upgrade packages, then update flatpaks/snaps if available
  -S <pkg...>           install packages
  -R <pkg...>           remove packages
  -Rs <pkg...>          remove packages, then autoremove unused dependencies
  -Ss <query...>        search available packages
  -Si <pkg>             show package details
  -Q                    list installed packages
  -Qs <query...>        search installed packages
  -Qu                   list available upgrades
  -Sc                   clean package cache

Global options:
  --noconfirm           pass -y to apt/flatpak write operations
  -h, --help            show this help
  -V, --version         show version
`, version)
}
