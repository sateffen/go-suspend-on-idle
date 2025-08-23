# Go Suspend on Idle

This is a small helper tool for my private homeserver. It checks if the system is idle and, if so, runs `systemctl suspend` to put the system into suspend mode.

## How it works

- Monitors system activity to detect idleness:
  - Checks for no active non-localhost TCP connections by parsing kernel network information from `/proc/net/tcp` and `/proc/net/tcp6`.
  - Checks for no active user sessions using `loginctl`.
- If the system is idle, it calls `systemctl suspend` to suspend the system.

## Building the project

There are two main ways to build this project:

### 1. Build with Go

You can build the binary by just using Go:

```sh
go build -o go-suspend-on-idle *.go
```

### 2. Build an Arch Linux package

If you want to build an Arch package, you can use the PKGBUILD in the archlinux/ folder:

```sh
cd archlinux
makepkg
```

Then you can install the resulting package with `pacman`.

## Available options

The following command-line options are available:

- `-verbose` (default: `false`):
  Enable verbose (debug) logging output.
- `-idletime` (default: `3`):
  Set the number of minutes the system must be idle (no active non-localhost TCP connections and no user sessions) before suspending.

If you want to use these options when running the program as a service, you may need to add them to the `ExecStart` line in your systemd unit file.

## Disclaimer

This project is just something I made for my own homeserver. You can use or fork it if you want, but don't expect me to add features for you. Use it at your own risk.
