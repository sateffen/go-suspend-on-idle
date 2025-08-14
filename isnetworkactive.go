package main

import (
	"bufio"
	"log/slog"
	"os"
	"strings"
)

func isNetworkActive() bool {
	// check ipv4 connections
	if hasNonLocalhostConnections("/proc/net/tcp") {
		return true
	}

	// check ipv6 connections
	return hasNonLocalhostConnections("/proc/net/tcp6")
}

func hasNonLocalhostConnections(procFile string) bool {
	file, err := os.Open(procFile)
	if err != nil {
		slog.Error("Could not open TCP connections file", slog.String("file", procFile), slog.Any("error", err))
		return true
	}
	defer file.Close()

	scanner := bufio.NewScanner(file)

	scanner.Scan()

	for scanner.Scan() {
		fields := strings.Fields(scanner.Text())
		if len(fields) < 4 {
			continue
		}

		// Quick state check first - avoid expensive parsing if not established
		if !isStateEstablished(fields[3]) {
			continue
		}

		if !isLocalhostHex(fields[1]) {
			return true
		}
	}

	return false
}

func isStateEstablished(stateHex string) bool {
	// Quick check: established state is "01" in hex
	return stateHex == "01"
}

func isLocalhostHex(addrPort string) bool {
	colonIndex := strings.Index(addrPort, ":")
	if colonIndex == -1 {
		return false
	}

	hexAddr := addrPort[:colonIndex]

	// IPv4: 127.x.x.x range (127.0.0.0/8)
	if len(hexAddr) == 8 {
		return hexAddr[:2] == "7F"
	}

	// IPv6: Only ::1 exactly
	if len(hexAddr) == 32 {
		return hexAddr == "00000000000000000000000000000001"
	}

	return false
}
