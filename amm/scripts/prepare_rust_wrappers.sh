#!/bin/bash

# Runs ink-wrapper in order to generate Rust wrapper for selected contracts.
# Copies contracts' wasm files to the drink-tests resources directory.
# Requires that contracts had been built before running this script.

readonly SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

mkdir -p $SCRIPT_DIR/../drink-tests/resources

declare -a CONTRACTS=(
    "factory_contract" 
    "pair_contract" 
    "router_contract"
    "wrapped_azero"
    "psp22"
)

function wrap_contracts() {
    for c in ${CONTRACTS[@]}; do
        echo "Wrapping $c"
        ink-wrapper --metadata $SCRIPT_DIR/../../target/ink/$c/$c.json \
		            --wasm-path ../resources/$c.wasm \
	 		| rustfmt --edition 2021 > $SCRIPT_DIR/../drink-tests/src/$c.rs ;
    done
}

function copy_wasms() {
    for c in ${CONTRACTS[@]}; do
        echo "Copying $c.wasm"
        cp $SCRIPT_DIR/../../target/ink/$c/$c.wasm $SCRIPT_DIR/../drink-tests/resources/$c.wasm;
    done
}

function run() {
    wrap_contracts
    copy_wasms
}

run