package main

import (
	"bufio"
	"encoding/hex"
	"fmt"
	"log/slog"
	"net"
	"os"
	"strconv"
	"strings"
)

func isNetworkActive(networkInterfaceName string) bool {
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

	interfaceIPs := extractIPAddresses(networkInterfaceAddrs)
	if len(interfaceIPs) == 0 {
		return false
	}

	if hasEstablishedConnectionsOnInterface("/proc/net/tcp", interfaceIPs) {
		return true
	}

	return hasEstablishedConnectionsOnInterface("/proc/net/tcp6", interfaceIPs)
}

func extractIPAddresses(addrs []net.Addr) []net.IP {
	var ips []net.IP
	for _, addr := range addrs {
		if ipNet, ok := addr.(*net.IPNet); ok {
			ips = append(ips, ipNet.IP)
		}
	}
	return ips
}

func hasEstablishedConnectionsOnInterface(procFile string, interfaceIPs []net.IP) bool {
	file, err := os.Open(procFile)
	if err != nil {
		slog.Error("Could not open TCP connections file", slog.String("file", procFile), slog.Any("error", err))
		return true
	}
	defer file.Close()

	scanner := bufio.NewScanner(file)

	scanner.Scan()

	for scanner.Scan() {
		line := strings.TrimSpace(scanner.Text())
		if line == "" {
			continue
		}

		conn, err := parseTCPLine(line)
		if err != nil {
			slog.Debug("Failed to parse TCP line", slog.String("line", line), slog.Any("error", err))
			continue
		}

		if conn.state == tcpStateEstablished {
			for _, ifaceIP := range interfaceIPs {
				if conn.localAddr.Equal(ifaceIP) {
					return true
				}
			}
		}
	}

	return false
}

type tcpConnection struct {
	localAddr  net.IP
	localPort  uint16
	remoteAddr net.IP
	remotePort uint16
	state      tcpState
}

type tcpState int

const (
	tcpStateEstablished tcpState = 1
)

func parseTCPLine(line string) (tcpConnection, error) {
	fields := strings.Fields(line)
	if len(fields) < 4 {
		return tcpConnection{}, fmt.Errorf("invalid TCP line format")
	}

	localAddr, localPort, err := parseAddressPort(fields[1])
	if err != nil {
		return tcpConnection{}, fmt.Errorf("invalid local address: %w", err)
	}

	remoteAddr, remotePort, err := parseAddressPort(fields[2])
	if err != nil {
		return tcpConnection{}, fmt.Errorf("invalid remote address: %w", err)
	}

	state, err := parseState(fields[3])
	if err != nil {
		return tcpConnection{}, fmt.Errorf("invalid state: %w", err)
	}

	return tcpConnection{
		localAddr:  localAddr,
		localPort:  localPort,
		remoteAddr: remoteAddr,
		remotePort: remotePort,
		state:      state,
	}, nil
}

func parseAddressPort(addrPort string) (net.IP, uint16, error) {
	parts := strings.Split(addrPort, ":")
	if len(parts) != 2 {
		return nil, 0, fmt.Errorf("invalid address:port format")
	}

	var addr net.IP
	var err error

	if len(parts[0]) == 8 {
		addr, err = parseHexIPv4(parts[0])
	} else if len(parts[0]) == 32 {
		addr, err = parseHexIPv6(parts[0])
	} else {
		return nil, 0, fmt.Errorf("invalid address hex length: %d", len(parts[0]))
	}

	if err != nil {
		return nil, 0, fmt.Errorf("invalid address: %w", err)
	}

	port, err := strconv.ParseUint(parts[1], 16, 16)
	if err != nil {
		return nil, 0, fmt.Errorf("invalid port: %w", err)
	}

	return addr, uint16(port), nil
}

func parseHexIPv4(hexAddr string) (net.IP, error) {
	if len(hexAddr) != 8 {
		return nil, fmt.Errorf("invalid IPv4 hex length: %d", len(hexAddr))
	}

	bytes, err := hex.DecodeString(hexAddr)
	if err != nil {
		return nil, err
	}

	return net.IPv4(bytes[3], bytes[2], bytes[1], bytes[0]), nil
}

func parseHexIPv6(hexAddr string) (net.IP, error) {
	if len(hexAddr) != 32 {
		return nil, fmt.Errorf("invalid IPv6 hex length: %d", len(hexAddr))
	}

	bytes, err := hex.DecodeString(hexAddr)
	if err != nil {
		return nil, err
	}

	ipv6 := make(net.IP, 16)
	for i := 0; i < 16; i += 4 {
		ipv6[i] = bytes[i+3]
		ipv6[i+1] = bytes[i+2]
		ipv6[i+2] = bytes[i+1]
		ipv6[i+3] = bytes[i]
	}

	return ipv6, nil
}

func parseState(stateHex string) (tcpState, error) {
	state, err := strconv.ParseInt(stateHex, 16, 32)
	if err != nil {
		return 0, err
	}

	return tcpState(state), nil
}
