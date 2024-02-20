#!/usr/bin/env bash

# This script copies the contract artifacts from the build directory to the
# artifacts directory expected by the typechain-compiler binary.

set -euo pipefail

script_path="${BASH_SOURCE[0]}"
script_dir=$(dirname "${script_path}")

function cp_files() {
    cp "$script_dir"/../target/ink/"$1"/"$1".json "$script_dir"/../artifacts/ && \
    cp "$script_dir"/../target/ink/"$1"/"$1".contract "$script_dir"/../artifacts/ && \
    echo "Copied $1 artifacts" || echo "Failed to copy $1 artifacts"
}

mkdir -p "$script_dir"/../artifacts

cp_files "factory_contract"
cp_files "pair_contract"
cp_files "router_contract"
cp_files "psp22"
cp_files "wrapped_azero"
cp_files "farm_contract"
