# Go Suspend on Idle

This is a small helper tool for my private homeserver. It checks if the system is idle and, if so, runs `systemctl suspend` to put the system into suspend mode.

## How it works

- Monitors system activity to detect idleness:
  - Checks for no active TCP connections on a specified ethernet interface.
  - Checks for no active user sessions on the system.
- If the system is idle, it calls `systemctl suspend` to suspend the system.

## Building the project

There are two main ways to build this project:

### 1. Build with Make

You can build the binary using the provided Makefile:

```sh
make build
```

The compiled binary will be placed in the `bin/` directory.

### 2. Build an Arch Linux package

If you want to build an Arch package, use:

```sh
make arch-package
```

This will use `makepkg` to create a package file for Arch Linux, which you can then install with `pacman`.

## Available options

The following command-line options are available:

- `-interface` (default: `eth0`):
  Specify the ethernet interface to monitor for active TCP connections.
- `-verbose`:
  Enable verbose (debug) logging output.
- `-idletime` (default: `3`):
  Set the number of minutes the system must be idle (no active TCP connections and no user sessions) before suspending.

If you want to use these options when running the program as a service, you may need to add them to the `ExecStart` line in your systemd unit file.

## Disclaimer

This project is just something I made for my own homeserver. You can use or fork it if you want, but don't expect me to add features for you. Use it at your own risk.
