package main

import (
	"flag"
	"log/slog"
	"os"
	"os/exec"
	"os/signal"
	"syscall"
	"time"
)

func getLogLevel(isVerbose bool) slog.Level {
	if isVerbose {
		return slog.LevelDebug
	}

	return slog.LevelInfo
}

func main() {
	var isVerbose bool
	var idleTime int

	flag.BoolVar(&isVerbose, "verbose", true, "")
	flag.IntVar(&idleTime, "idletime", 3, "")
	flag.Parse()

	globalLogger := slog.New(slog.NewTextHandler(os.Stdout, &slog.HandlerOptions{
		Level: getLogLevel(isVerbose),
	}))
	slog.SetDefault(globalLogger)

	signalChan := make(chan os.Signal, 1)
	signal.Notify(signalChan, os.Interrupt, syscall.SIGINT, syscall.SIGTERM)

	checkActivityTicker := time.NewTicker(1 * time.Minute)

	slog.Debug("starting with configuration:", slog.Bool("verbose", isVerbose))
	slog.Info("started successfully")

	currentInactivityCounter := 0

	for {
		select {
		case receivedSignal := <-signalChan:
			slog.Info("Got closing signal, exiting...", slog.Any("receivedSignal", receivedSignal))
			return
		case <-checkActivityTicker.C:
			if isNetworkActive() || isAnyUserActive() {
				slog.Debug("system is in use, skip suspending...")
				currentInactivityCounter = 0
				continue
			}

			currentInactivityCounter += 1

			if currentInactivityCounter < idleTime {
				slog.Debug("system is inactive", slog.Int("counter", currentInactivityCounter))
				continue
			}

			slog.Debug("system is inactive for long peroid, start suspending...")
			err := exec.Command("systemctl", "suspend").Run()
			if err != nil {
				slog.Error("error executing 'systemctl suspend'", slog.Any("error", err))
				continue
			}

			slog.Debug("Suspend ended, welcome back")
			currentInactivityCounter = 0
		}
	}
}
