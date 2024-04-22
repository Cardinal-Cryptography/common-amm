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
)

declare -a TEST_CONTRACTS=(
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
    echo "Wrapping wrapped_azero" &&
    ink-wrapper --metadata $SCRIPT_DIR/../../test-contracts/wrapped-azero/target/ink/wrapped_azero.json \
                --wasm-path ../resources/wrapped_azero.wasm \
        | rustfmt --edition 2021 > $SCRIPT_DIR/../drink-tests/src/wrapped_azero.rs ;
    echo "Wrapping psp22" &&
    ink-wrapper --metadata $SCRIPT_DIR/../../test-contracts/psp22/target/ink/psp22.json \
                --wasm-path ../resources/psp22.wasm \
        | rustfmt --edition 2021 > $SCRIPT_DIR/../drink-tests/src/psp22.rs ;
}

function copy_wasms() {
    for c in ${CONTRACTS[@]}; do
        echo "Copying $c.wasm"
        cp $SCRIPT_DIR/../../target/ink/$c/$c.wasm $SCRIPT_DIR/../drink-tests/resources/$c.wasm;
    done
    echo "Copying wrapped_zero.wasm" &&
    cp $SCRIPT_DIR/../../test-contracts/wrapped-azero/target/ink/wrapped_azero.wasm $SCRIPT_DIR/../drink-tests/resources/wrapped_azero.wasm ;
    echo "Copying psp22.wasm" &&
    cp $SCRIPT_DIR/../../test-contracts/psp22/target/ink/psp22.wasm $SCRIPT_DIR/../drink-tests/resources/psp22.wasm ;
}

function run() {
    wrap_contracts
    copy_wasms
}

run
