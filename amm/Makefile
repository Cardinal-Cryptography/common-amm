.DEFAULT_GOAL := help

.PHONY: help
help: # Show help for each of the Makefile recipes.
	@grep -E '^[a-zA-Z0-9 -]+:.*#'  Makefile | sort | while read -r l; do printf "\033[1;32m$$(echo $$l | cut -f 1 -d':')\033[00m:$$(echo $$l | cut -f 2- -d'#')\n"; done

AMM_CONTRACTS = ./contracts
AMM_CONTRACTS_PATHS := $(shell find $(AMM_CONTRACTS) -mindepth 1 -maxdepth 1 -type d)

TEST_CONTRACTS = ../test-contracts
TEST_PATHS := $(shell find $(TEST_CONTRACTS) -mindepth 1 -maxdepth 1 -type d)

.PHONY: build-amm
build-amm: ## Builds AMM contracts.
	@for d in $(AMM_CONTRACTS_PATHS); do \
	 	echo "Building $$d contract" ; \
	 	cargo contract build --quiet --manifest-path $$d/Cargo.toml --release ; \
	done

.PHONY: check-amm
check-amm: check-drink-tests ## Runs cargo (contract) check on AMM contracts.
	@for d in $(AMM_CONTRACTS_PATHS); do \
		echo "Checking $$d" ; \
		cargo check --quiet --all-targets --all-features --manifest-path $$d/Cargo.toml ; \
		cargo clippy --quiet --all-features --manifest-path $$d/Cargo.toml -- --no-deps -D warnings ; \
		cargo contract check --quiet --manifest-path $$d/Cargo.toml ; \
	done

.PHONY: build-test-contracts
build-test-contracts: ## Builds contracts used in e2e-tests
	@for d in $(TEST_PATHS); do \
		echo "Building $$d contract" ; \
		if [ "$$d" = "$(TEST_CONTRACTS)/psp22" ]; then \
			cargo contract build --quiet --manifest-path $$d/Cargo.toml --release --features "contract"; \
		else \
			cargo contract build --quiet --manifest-path $$d/Cargo.toml --release; \
		fi \
	done

.PHONY: build-all
build-all: build-amm build-test-contracts ## Builds all contracts.

CONTRACTS := factory_contract pair_contract router_contract wrapped_azero

.PHONY: wrap-all
wrap-all: ## Generates Rust wrappers for interacting with AMM contracts.
	@for c in $(CONTRACTS); do \
		echo "Wrapping $$c contract" ; \
		ink-wrapper --metadata ../target/ink/$$c/$$c.json \
		            --wasm-path ../../../target/ink/$$c/$$c.wasm \
	 		| rustfmt --edition 2021 > ./drink-tests/src/$$c.rs ; \
	done
	@echo "Wrapping psp22 contract" ; \
	ink-wrapper --metadata ../target/ink/psp22/psp22.json \
	            --wasm-path ../../../target/ink/psp22/psp22.wasm \
		| rustfmt --edition 2021 > ./drink-tests/src/psp22_contract.rs ; \

.PHONY: setup-tests
setup-tests: build-test-contracts build-amm wrap-all  ## Builds contracts and generates wrappers.

.PHONY: drink-tests
drink-tests: ## Runs tests for drink contract.
	@echo "Running drink tests" ; \
	cargo test --manifest-path ./drink-tests/Cargo.toml -- --nocapture ; \

.PHONY: check-drink-tests
check-drink-tests: ## Runs cargo checks on drink tests.
	@echo "Checking drink tests" ; \
	cargo check --tests --manifest-path ./drink-tests/Cargo.toml ; \

.PHONY:
all-drink: setup-tests drink-tests ## Runs all tests.
