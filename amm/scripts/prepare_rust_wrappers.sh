#!/bin/bash

# Runs ink-wrapper in order to generate Rust wrapper for selected contracts.
# Copies contracts' wasm files to the drink-tests resources directory.
# Requires that contracts had been built before running this script.

readonly SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

declare -a AMM_CONTRACTS=(
    "factory_contract" 
    "pair_contract" 
    "router_contract"
)

declare -a CONTRACTS=(
    "factory_contract" 
    "pair_contract" 
    "router_contract"
    "wrapped_azero"
    "psp22"
)

function copy_wasms() {
    for c in ${AMM_CONTRACTS[@]}; do
        echo "Copying $c"
        cp $SCRIPT_DIR/../../target/ink/$c/$c.wasm $SCRIPT_DIR/../../artifacts/$c.wasm;
        cp $SCRIPT_DIR/../../target/ink/$c/$c.json $SCRIPT_DIR/../../artifacts/$c.json;
        cp $SCRIPT_DIR/../../target/ink/$c/$c.contract $SCRIPT_DIR/../../artifacts/$c.contract;
    done
}

function wrap_contracts() {
    for c in ${CONTRACTS[@]}; do
        echo "Wrapping $c"
        ink-wrapper --metadata $SCRIPT_DIR/../../artifacts/$c.json \
		            --wasm-path $SCRIPT_DIR/../../artifacts/$c.wasm \
	 		| rustfmt --edition 2021 > $SCRIPT_DIR/../drink-tests/src/$c.rs ;
    done
}


function run() {
    copy_wasms
    wrap_contracts
}

run
