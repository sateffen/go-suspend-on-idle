# Global variables
PROJECTNAME=go-suspend-on-idle
# Go related variables.
GOBASE=$(shell pwd)
GOPATH="$(GOBASE)/vendor:$(GOBASE)"
GOBIN=$(GOBASE)/bin
GOFILES=$(wildcard *.go)

## clean-build: Clean up, install dependencies and build the project
clean-build: clean install-dependencies build

## clean: Clean the projects build cache
clean:
	@echo " > Cleaning Build Cache..."
	@GOPATH=$(GOPATH) GOBIN=$(GOBIN) go clean
	@rm -r bin/

## clean-arch-build: Clean artifacts created by arch-build
clean-arch-package:
	@echo " > Cleaning Arch-Package-Build Cache..."
	@rm -r pkg/ src/ go-suspend-on-idle-*.pkg.tar.zst

## run: Run the project using the local config.toml
run:
	@echo " > Running Code..."
	@GOPATH=$(GOPATH) GOBIN=$(GOBIN) go run $(GOFILES) config.toml

## build: Build the project
build:
	@echo " > Building Binary..."
	@GOPATH=$(GOPATH) GOBIN=$(GOBIN) go build -ldflags="-w -s" -o $(GOBIN)/$(PROJECTNAME) $(GOFILES)

## arch-package: Build an arch-package
arch-package:
	@echo " > Building Arch Package..."
	@makepkg

## install-dependencies: Install all necessary dependencies for this project
install-dependencies:
	@echo " > Checking if there is any missing dependencies..."
	@GOPATH=$(GOPATH) GOBIN=$(GOBIN) go get $(get)

.PHONY: help
help: Makefile
	@echo
	@echo "Choose a command run in "$(PROJECTNAME)":"
	@echo
	@sed -n 's/^##//p' $< | column -t -s ':' |  sed -e 's/^/ /'
	@echo
