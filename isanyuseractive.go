package main

import (
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
