.DEFAULT_GOAL := help

.PHONY: help
help: # Show help for each of the Makefile recipes.
	@grep -E '^[a-zA-Z0-9 -]+:.*#'  Makefile | sort | while read -r l; do printf "\033[1;32m$$(echo $$l | cut -f 1 -d':')\033[00m:$$(echo $$l | cut -f 2- -d'#')\n"; done

# Native arch
BUILDARCH := $(shell uname -m)

.PHONY: start-node
start-node: ## Runs a network consisting of a single Aleph node.
	@echo "Starting aleph-network."
	@docker run --detach --rm --network host \
		--name aleph-network aleph-onenode-chain-${BUILDARCH}

.PHONY: stop-node
stop-node: ## Stops the local network.
	@echo "Stopping aleph-network."
	@docker stop aleph-network

.PHONY: restart-node
restart-node: stop-node start-node ## Restarts the local network.

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

AMM_CONTRACTS = ./amm/contracts
AMM_CONTRACTS_PATHS := $(shell find $(AMM_CONTRACTS) -mindepth 1 -maxdepth 1 -type d)

FARM_CONTRACTS = ./farm
FARM_PATHS := $(shell find $(FARM_CONTRACTS) -mindepth 1 -maxdepth 1 -type d)

TEST_CONTRACTS = ./test-contracts
TEST_PATHS := $(shell find $(TEST_CONTRACTS) -mindepth 1 -maxdepth 1 -type d)

.PHONY: build-all
build-all: ## Builds all production contracts.
	@for d in $(AMM_CONTRACTS_PATHS); do \
		echo "Building $$d contract" ; \
		cargo contract build --quiet --manifest-path $$d/Cargo.toml --release ; \
	done
	@for d in $(FARM_PATHS); do \
		echo "Building $$d contract" ; \
		cargo contract build --quiet --manifest-path $$d/Cargo.toml --release ; \
	done

.PHONY: build-test-contracts
build-test-contracts: ## Builds contracts used in e2e-tests
	@for d in $(TEST_PATHS); do \
		echo "Building $$d contract" ; \
		if [[ "$$d" = $(TEST_CONTRACTS)/psp22 ]]; then \
			cargo contract build --quiet --manifest-path $$d/Cargo.toml --release --features "contract"; \
		else \
			cargo contract build --quiet --manifest-path $$d/Cargo.toml --release; \
		fi \
	done

.PHONY: check-all
check-all: ## Runs cargo checks and unit tests on all contracts.
	@cargo check --quiet --all-targets --all-features --all
	@cargo clippy --quiet --all-features -- --no-deps -D warnings
	@cargo fmt --all --check
	@for d in $(AMM_CONTRACTS_PATHS); do \
		echo "Checking $$d" ; \
		cargo contract check --quiet --manifest-path $$d/Cargo.toml ; \
	done
	@for d in $(FARM_PATHS); do \
		echo "Checking $$d" ; \
		cargo contract check --quiet --manifest-path $$d/Cargo.toml ; \
	done
	@cargo test --quiet --locked --frozen --workspace
	@echo "Checking AMM e2e tests"
	@cd ./amm/e2e-tests && cargo check --quiet

.PHONY: format
format: ## Formats contract files.
	@cargo fmt --all

CONTRACT_DATA = ./target/ink

.PHONY: wrap-all
wrap-all: ## Generates code for contract interaction.
	@for c in $(notdir $(shell find $(CONTRACT_DATA) -mindepth 1 -maxdepth 1 -type d)); do \
		echo "Wrapping $$c" ; \
	 	ink-wrapper -m ./target/ink/$$c/$$c.json --wasm-path ../../../target/ink/$$c/$$c.wasm \
	 		| rustfmt --edition 2021 > ./amm/e2e-tests/src/$$c.rs ; \
	done

.PHONY: e2e-tests
e2e-tests: ## Runs all the e2e tests in sequence.
	@cd amm/e2e-tests && cargo test -- --test-threads 1 && cd ..

.PHONY: build-and-wrap-all
build-and-wrap-all: build-all build-test-contracts wrap-all ## Builds all contracts and generates code for contract interaction.

INK_DEV_IMAGE = public.ecr.aws/p6e8q1z1/ink-dev:1.7.0

.PHONY: check-all-dockerized
check-all-dockerized: ## Runs cargo checks and unit tests on all contracts in a container.
	@docker run --rm \
    	--name ink-dev \
    	-v "$(shell pwd)":/code \
    	$(INK_DEV_IMAGE) \
    	make check-all

.PHONY: build-and-wrap-all-dockerized
build-and-wrap-all-dockerized: ## Builds all contracts and generates code for contract interaction. Run in a container.
	@docker run --rm \
    	--name ink-dev \
    	-v "$(shell pwd)":/code \
    	$(INK_DEV_IMAGE) \
    	make build-and-wrap-all

.PHONY: e2e-tests-with-setup-and-teardown
e2e-tests-with-setup-and-teardown: build-and-wrap-all-dockerized build-node start-node e2e-tests stop-node ## Runs the E2E test suite.
