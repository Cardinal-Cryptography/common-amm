.DEFAULT_GOAL := help

.PHONY: help
help: # Show help for each of the Makefile recipes.
	@grep -E '^[a-zA-Z0-9 -]+:.*#'  Makefile | sort | while read -r l; do printf "\033[1;32m$$(echo $$l | cut -f 1 -d':')\033[00m:$$(echo $$l | cut -f 2- -d'#')\n"; done

# Native arch
BUILDARCH := $(shell uname -m)

.PHONY: up
up: build-node ## Starts up the local network consisting of single Aleph node.
	@docker run --detach --rm --network host \
	       	--name aleph-network aleph-onenode-chain-${BUILDARCH}

.PHONY: down
down: ## Stops the local network.
	@docker stop aleph-network

.PHONY: test
test: ## Runs the e2e tests
	@npm run test:typechain

.PHONY: build-node
# Build multi-CPU architecture images and publish them. rust alpine images support the linux/amd64 and linux/arm64/v8 architectures.
build-node: build-node-${BUILDARCH} ## Detects local arch and builds docker image
	@docker build --tag aleph-onenode-chain --file docker/Dockerfile docker

.PHONY: build-node-arm64
build-node-arm64:
	@docker buildx build --pull --platform linux/arm64/v8  -t aleph-onenode-chain-arm64 --load docker

.PHONY: build-node-x86_64
build-node-x86_64:
	@docker buildx build --pull --platform linux/amd64 -t aleph-onenode-chain-x86_64 --load docker

CONTRACTS = ./uniswap-v2/contracts
LOGIC = ./uniswap-v2/logics

.PHONY: check-all
check-all: # Runs cargo checks on all contracts
	@cargo check --all-targets --all-features --all
	@cargo clippy --all-features -- --no-deps -D warnings
	@for d in $(shell find $(CONTRACTS) -mindepth 1 -maxdepth 1 -type d); do \
		cargo contract check --manifest-path $$d/Cargo.toml ; \
	done

.PHONY: build-all
build-all: # Builds all contracts
	@for d in $(shell find $(CONTRACTS) -mindepth 1 -maxdepth 1 -type d); do \
		cargo contract build --manifest-path $$d/Cargo.toml --release ; \
	done

.PHONY: format
format: # Formats contract files
	@cargo fmt --all