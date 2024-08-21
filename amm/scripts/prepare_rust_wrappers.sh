#!/bin/bash

# Runs ink-wrapper in order to generate Rust wrapper for selected contracts.
# Copies contracts' wasm files to the drink-tests resources directory.
# Requires that contracts had been built before running this script.

readonly SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

declare -a CONTRACTS=(
    "factory_contract" 
    "pair_contract" 
    "stable_pool_contract" 
    "router_contract"
)

function wrap_contracts() {
    for c in ${CONTRACTS[@]}; do
        echo "Wrapping $c"
        ink-wrapper --metadata $SCRIPT_DIR/../../artifacts/$c.json \
		            --wasm-path $SCRIPT_DIR/../../artifacts/$c.wasm \
	 		| rustfmt --edition 2021 > $SCRIPT_DIR/../drink-tests/src/$c.rs ;
    done
}

wrap_contracts
