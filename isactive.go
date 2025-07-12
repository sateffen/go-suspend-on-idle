package main

import (
	"log/slog"
	"net"
	"os/exec"
	"strings"
)

func isAnyUserActive() bool {
	cmdOutput, err := exec.Command("loginctl", "list-sessions", "--no-legend").Output()
	if err != nil {
		return true
	}

	return strings.Contains(string(cmdOutput), "active")
}

func isNetworkActive(networkInterfaceName string) bool {
	cmdOutput, err := exec.Command("ss", "-tn", "-o", "state", "established", "--no-header").Output()
	if err != nil {
		slog.Error("Could not read open TCP connections", slog.Any("error", err))
		return true
	}

	networkInterface, err := net.InterfaceByName(networkInterfaceName)
	if err != nil {
		slog.Error("Could not read network interface", slog.String("interface", networkInterfaceName), slog.Any("error", err))
		return true
	}

	networkInterfaceAddrs, err := networkInterface.Addrs()
	if err != nil {
		slog.Error("Could not read network interface addresses", slog.String("interface", networkInterfaceName), slog.Any("error", err))
		return true
	}

	cmdOutputAsString := string(cmdOutput)

	for _, interfaceAddr := range networkInterfaceAddrs {
		ipAddr := interfaceAddr.String()

		if strings.Contains(cmdOutputAsString, ipAddr[:len(ipAddr)-3]) {
			return true
		}
	}

	return false
}
