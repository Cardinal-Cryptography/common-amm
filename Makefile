.DEFAULT_GOAL := help

.PHONY: help
help: # Show help for each of the Makefile recipes.
	@grep -E '^[a-zA-Z0-9 -]+:.*#'  Makefile | sort | while read -r l; do printf "\033[1;32m$$(echo $$l | cut -f 1 -d':')\033[00m:$$(echo $$l | cut -f 2- -d'#')\n"; done

# Native arch
BUILDARCH := $(shell uname -m)

.PHONY: run-node
run-node: build-node ## Runs a network consisting of a single Aleph node.
	@echo "Starting aleph-network."
	@docker run --detach --rm --network host \
		--name aleph-network aleph-onenode-chain-${BUILDARCH}

.PHONY: stop-node
stop-node: ## Stops the local network.
	@echo "Stopping aleph-network."
	@docker stop aleph-network

.PHONY: kill-node
kill-node: ## Kills the local network.
	@echo "Killing aleph-network."
	@docker kill aleph-network

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
	@for d in $(CONTRACT_PATHS); do \
		echo "cargo contract build --quiet --manifest-path $$d/Cargo.toml --release" ; \
		cargo contract build --quiet --manifest-path $$d/Cargo.toml --release ; \
	done

.PHONY: build-all-for-e2e-tests
build-all-for-e2e-tests: ## Builds all contracts with features required for e2e tests.
	@for d in $(CONTRACT_PATHS); do \
		echo "cargo contract build --quiet --manifest-path $$d/Cargo.toml --release --features e2e-tests" ; \
		cargo contract build --quiet --manifest-path $$d/Cargo.toml --release --features e2e-tests; \
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
.PHONY: e2e-test
e2e-test:
	@echo "\nRunning test case: test::$(TEST)\n"
	@cd e2e-tests && cargo test test::$(TEST) -- --exact && cd ..

TEST_CASES = \
	factory::factory_contract_set_up_correctly \
	factory::set_fee \
	factory::set_fee_setter \
	pair::create_pair \
	pair::mint_pair \
	pair::swap_tokens \
	pair::burn_liquidity_provider_token

.PHONY: e2e-tests
e2e-tests:
	@for t in $(TEST_CASES); do \
		make TEST=$$t e2e-test ; \
  	done

.PHONY: build-and-wrap-all
build-and-wrap-all: build-all wrap-all

.PHONY: build-and-wrap-all-for-e2e-tests
build-and-wrap-all-for-e2e-tests: build-all-for-e2e-tests wrap-all

.PHONY: check-all-dockerized
check-all-dockerized: build-ink-dev
	@docker run --rm \
    	--network host \
    	--user "$(shell id -u):$(shell id -g)" \
    	--name ink-dev \
    	-v "$(shell pwd)":/code \
    	-v ~/.cargo/git:/usr/local/cargo/git \
    	-v ~/.cargo/registry:/usr/local/cargo/registry \
    	ink-dev \
    	make check-all

.PHONY: build-and-wrap-all-for-e2e-tests-dockerized
build-and-wrap-all-for-e2e-tests-dockerized: build-ink-dev
	@docker run --rm \
    	--network host \
    	--user "$(shell id -u):$(shell id -g)" \
    	--name ink-dev \
    	-v "$(shell pwd)":/code \
    	-v ~/.cargo/git:/usr/local/cargo/git \
    	-v ~/.cargo/registry:/usr/local/cargo/registry \
    	ink-dev \
    	make build-and-wrap-all-for-e2e-tests

.PHONY: e2e-tests-with-prelims
e2e-tests-with-prelims: build-and-wrap-all-for-e2e-tests-dockerized run-node e2e-tests stop-node
