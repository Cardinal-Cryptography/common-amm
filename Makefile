.DEFAULT_GOAL := help

.PHONY: help
help: # Show help for each of the Makefile recipes.
	@grep -E '^[a-zA-Z0-9 -]+:.*#'  Makefile | sort | while read -r l; do printf "\033[1;32m$$(echo $$l | cut -f 1 -d':')\033[00m:$$(echo $$l | cut -f 2- -d'#')\n"; done

# Native arch
BUILDARCH := $(shell uname -m)

.PHONY: up
up: build-node ## Starts up the local network consisting of single Aleph node.
	@echo "Starting aleph-network."
	@docker run --detach --rm --network host \
	       	--name aleph-network aleph-onenode-chain-${BUILDARCH}

.PHONY: down
down: ## Stops the local network.
	@echo "Stopping aleph-network."
	@docker stop aleph-network

.PHONY: test
test: ## Runs the e2e tests.
	@npm run test:typechain

.PHONY: build-node
# Build multi-CPU architecture images and publish them. rust alpine images support the linux/amd64 and linux/arm64/v8 architectures.
build-node: build-node-${BUILDARCH} ## Detects local arch and builds docker image
	@docker build --tag aleph-onenode-chain --file docker/aleph_node/Dockerfile docker/aleph_node

.PHONY: build-node-arm64
build-node-arm64:
	@docker buildx build --pull --platform linux/arm64/v8  -t aleph-onenode-chain-arm64 --load docker/aleph_node

.PHONY: build-node-x86_64
build-node-x86_64:
	@docker buildx build --pull --platform linux/amd64 -t aleph-onenode-chain-x86_64 --load docker/aleph_node

CONTRACTS = ./uniswap-v2/contracts
CONTRACT_PATHS := $(shell find $(CONTRACTS) -mindepth 1 -maxdepth 1 -type d)
# Read, write, execute access added for interaction outside the container.
BUILD_CONTRACTS_CMD := "echo 'Building contracts!'$(foreach d,$(CONTRACT_PATHS),\
	&& cargo contract build --quiet --manifest-path $d/Cargo.toml --release) && chmod -R 777 target"

.PHONY: build-ink-dev
build-ink-dev: ## Builds ink-dev image for contract generation and wrapping.
	@docker build --tag ink-dev --file docker/ink_dev/Dockerfile docker/ink_dev

.PHONY: up-ink-dev
up-ink-dev: ## Runs an `ink-dev` container in the background.
	@echo "Starting ink-dev."
	@docker run --detach --tty --rm \
    	--network host \
    	--name ink-dev \
    	-v "$(shell pwd)":/code \
    	-v ~/.cargo/git:/usr/local/cargo/git \
    	-v ~/.cargo/registry:/usr/local/cargo/registry \
    	ink-dev

.PHONY: down-ink-dev
down-ink-dev: ## Stops the `ink-dev` container.
	@echo "Stopping ink-dev."
	@docker stop ink-dev

.PHONY: build-all
build-all: ## Builds all contracts.
	@docker exec ink-dev /bin/sh -c $(BUILD_CONTRACTS_CMD)

.PHONY: check-all
check-all: ## Runs cargo checks on all contracts.
	@docker exec ink-dev cargo check --quiet --all-targets --all-features --all
	@docker exec ink-dev cargo clippy --quiet --all-features -- --no-deps -D warnings
	@docker exec ink-dev cargo fmt --quiet --all --check
	@for d in $(CONTRACT_PATHS); do \
		docker exec ink-dev /bin/sh -c "echo 'Checking $$d' && cargo contract check --quiet --manifest-path $$d/Cargo.toml" ; \
	done
	@docker exec ink-dev cargo test --quiet --locked --frozen --workspace

.PHONY: format
format: ## Formats contract files.
	@docker exec ink-dev cargo fmt --all

CONTRACT_DATA = ./target/ink
CONTRACT_DATA_PATHS := $(shell find $(CONTRACT_DATA) -mindepth 1 -maxdepth 1 -type d)
CONTRACT_DATA_NAMES := $(notdir $(CONTRACT_DATA_PATHS))

INK_WRAPPER_CMD := "echo 'Wrapping contracts!' $(foreach c,$(CONTRACT_DATA_NAMES),&&\
	ink-wrapper -m ./target/ink/$c/$c.json --wasm-path ../../target/ink/$c/$c.wasm > ./e2e-tests/src/$c.rs)"

.PHONY: wrap-all
wrap-all: ## Generates code for contract interaction.
	@docker exec ink-dev /bin/sh -c $(INK_WRAPPER_CMD)

.PHONY: format-wrapped
format-wrapped: ## Formats code for contract interaction.
	@docker exec ink-dev /bin/sh -c "cd e2e-tests && cargo fmt"

.PHONY: build-and-wrap-all
build-and-wrap-all: up-ink-dev build-all check-all wrap-all format-wrapped down-ink-dev

# `TEST` needs to be passed into this rule.
.PHONY: e2e-test-case
e2e-test-case:
	@cd e2e-tests; cargo test test::$(TEST) -- --exact; cd ..

# `TEST` needs to be passed into this rule.
.PHONY: e2e-test
e2e-test: up e2e-test-case down
