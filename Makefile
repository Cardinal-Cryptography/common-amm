.DEFAULT_GOAL := help

.PHONY: help
help: # Show help for each of the Makefile recipes.
	@grep -E '^[a-zA-Z0-9 -]+:.*#'  Makefile | sort | while read -r l; do printf "\033[1;32m$$(echo $$l | cut -f 1 -d':')\033[00m:$$(echo $$l | cut -f 2- -d'#')\n"; done

# Native arch
BUILDARCH := $(shell uname -m)

.PHONY: run-node
run-node: build-node ## Runs a network consisting of single Aleph node.
	@echo "Starting aleph-network."
	@docker run --detach --rm --network host \
	       	--name aleph-network aleph-onenode-chain-${BUILDARCH}

.PHONY: stop-node
stop-node: ## Stops the local network.
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

.PHONY: build-ink-dev
build-ink-dev: ## Builds ink-dev image for contract generation and wrapping.
	@docker build --tag ink-dev --file docker/ink_dev/Dockerfile \
		--build-arg UID=$(shell id -u) --build-arg GID=$(shell id -g) \
		docker/ink_dev

.PHONY: build-all
build-all: ## Builds all contracts.
	@for d in $(shell find $(CONTRACTS) -mindepth 1 -maxdepth 1 -type d); do \
		echo "cargo contract build --quiet --manifest-path $$d/Cargo.toml --release" ; \
		cargo contract build --quiet --manifest-path $$d/Cargo.toml --release ; \
	done

.PHONY: check-all
check-all: ## Runs cargo checks on all contracts.
	@cargo check --quiet --all-targets --all-features --all
	@cargo clippy --quiet --all-features -- --no-deps -D warnings
	@cargo fmt --quiet --all --check
	@for d in $(shell find $(CONTRACTS) -mindepth 1 -maxdepth 1 -type d); do \
		echo "Checking $$d" ; \
		cargo contract check --quiet --manifest-path $$d/Cargo.toml ; \
	done
	@cargo test --quiet --locked --frozen --workspace

.PHONY: format
format: ## Formats contract files.
	@cargo fmt --all

CONTRACT_DATA = ./target/ink
CONTRACT_DATA_PATHS := $(shell find $(CONTRACT_DATA) -mindepth 1 -maxdepth 1 -type d)

.PHONY: wrap-all
wrap-all: ## Generates code for contract interaction.
	@for c in $(notdir $(CONTRACT_DATA_PATHS)); do \
		echo "Wrapping $$c" ; \
	 	ink-wrapper -m ./target/ink/$$c/$$c.json --wasm-path ../../target/ink/$$c/$$c.wasm \
	 		| rustfmt --edition 2021 > ./e2e-tests/src/$$c.rs ; \
	done

# `TEST` needs to be passed into this rule.
.PHONY: e2e-test-case
e2e-test-case:
	@echo "\nRunning test case: test::$(TEST)\n"
	@cd e2e-tests && cargo test test::$(TEST) -- --exact && cd ..

# `TEST` needs to be passed into this rule.
.PHONY: e2e-test
e2e-test: run-node e2e-test-case stop-node

TEST_CASES = \
	fee::factory_contract_set_up_correctly \
	fee::set_fee \
	fee::set_fee_setter \
	pair::create_pair \
	pair::mint_pair \
	pair::swap_tokens \
	pair::burn_liquidity_provider_token

.PHONY: e2e-tests
e2e-tests:
	@for t in $(TEST_CASES); do \
		make TEST=$$t e2e-test ; \
  	done

.PHONY: all
all: check-all build-all wrap-all #e2e-tests

.PHONY: all-dockerized
all-dockerized: build-ink-dev
	@docker run --rm \
    	--network host \
    	--user "$(shell id -u):$(shell id -g)" \
    	--name ink-dev \
    	-v "$(shell pwd)":/code \
    	-v ~/.cargo/git:/usr/local/cargo/git \
    	-v ~/.cargo/registry:/usr/local/cargo/registry \
    	--workdir /code \
    	ink-dev \
    	make all
