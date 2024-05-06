.DEFAULT_GOAL := help

.PHONY: help
help: # Show help for each of the Makefile recipes.
	@grep -E '^[a-zA-Z0-9 -]+:.*#'  Makefile | sort | while read -r l; do printf "\033[1;32m$$(echo $$l | cut -f 1 -d':')\033[00m:$$(echo $$l | cut -f 2- -d'#')\n"; done

.PHONY: build-farm
build-farm: ## Builds farm contracts.
	@echo "Building farm contract" ; \
	cargo contract build --quiet --manifest-path contract/Cargo.toml --release ; \
	cp ../target/ink/farm_contract/farm_contract.wasm ../artifacts/farm_contract.wasm ; \
	cp ../target/ink/farm_contract/farm_contract.json ../artifacts/farm_contract.json ; \
	cp ../target/ink/farm_contract/farm_contract.contract ../artifacts/farm_contract.contract ; 

.PHONY: check-farm
check-farm: ## Runs cargo checks on farm contracts.
	@echo "Checking farm contract" ; \
	cargo check --quiet --all-targets --all-features --manifest-path contract/Cargo.toml ; \
	cargo clippy --quiet --all-features --manifest-path contract/Cargo.toml -- --no-deps -D warnings ; \
	cargo contract check --quiet --manifest-path contract/Cargo.toml ; \

.PHONY: generate-farm-wrapper
generate-farm-wrapper: build-farm ## Generates Rust wrappers for interacting with farm contract.
	@echo "Wrapping farm contract" ; \
	ink-wrapper --metadata ../artifacts/farm_contract.json \
				--wasm-path ../../../../artifacts/farm_contract.wasm \
		| rustfmt --edition 2021 > ./tests/src/farm/farm_contract.rs ; \

.PHONY: setup-tests
setup-tests: build-farm generate-farm-wrapper ## Builds contracts and generates wrappers.

.PHONY: drink-tests
drink-tests: ## Runs tests for drink contract.
	@echo "Running drink tests" ; \
	cargo test --quiet --manifest-path ./tests/Cargo.toml -- --nocapture ; \

.PHONY:
all-drink: setup-tests drink-tests ## Runs all tests.
