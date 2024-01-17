.DEFAULT_GOAL := help

.PHONY: help
help: # Show help for each of the Makefile recipes.
	@grep -E '^[a-zA-Z0-9 -]+:.*#'  Makefile | sort | while read -r l; do printf "\033[1;32m$$(echo $$l | cut -f 1 -d':')\033[00m:$$(echo $$l | cut -f 2- -d'#')\n"; done

.PHONY: build-farm
build-farm: ## Builds farm contracts.
	@echo "Building farm contract" ; \
	cargo contract build --quiet --manifest-path contract/Cargo.toml --release ; \

.PHONY: check-farm
check-farm: ## Runs cargo checks on farm contracts.
	@echo "Checking farm contract" ; \
	cargo check --quiet --all-targets --all-features --manifest-path contract/Cargo.toml ; \
	cargo clippy --quiet --all-features --manifest-path contract/Cargo.toml -- --no-deps -D warnings ; \
	cargo contract check --quiet --manifest-path contract/Cargo.toml ; \

.PHONY: generate-farm-wrapper
generate-farm-wrapper: check-farm build-farm ## Generates Rust wrappers for interacting with farm contract.
	@echo "Wrapping farm contract" ; \
	ink-wrapper --metadata ../target/ink/farm_contract/farm_contract.json \
				--wasm-path ../../../target/ink/farm_contract/farm_contract.wasm \
		| rustfmt --edition 2021 > ./tests/src/farm.rs ; \

.PHONY: generate-psp22-wrapper
generate-psp22-wrapper: ## Generates Rust wrappers for interacting with PSP22 contracts.
	@echo "Building PSP22 contract" ; \
	cargo contract build --quiet --manifest-path ../test-contracts/psp22/Cargo.toml --release --features "contract" ;
	@echo "Wrapping PSP22 contract" ; \
	ink-wrapper --metadata ../target/ink/psp22/psp22.json \
				--wasm-path ../../../target/ink/psp22/psp22.wasm \
		| rustfmt --edition 2021 > ./tests/src/psp22.rs ; \

.PHONY: generate-wrappers
generate-wrappers: generate-psp22-wrapper generate-farm-wrapper ## Generates Rust wrappers for interacting with contracts.

.PHONY: setup-tests
setup-tests: generate-wrappers ## Builds contracts and generates wrappers.

.PHONY: drink-tests
drink-tests: ## Runs tests for drink contract.
	@echo "Running drink tests" ; \
	cargo test --quiet --manifest-path ./tests/Cargo.toml -- --nocapture ; \
