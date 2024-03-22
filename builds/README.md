# Verifiable contracts build
This repository contains metadata of contracts built using the following instruction:
```sh
docker run \
	--name subscan-builder \
	-v "$(pwd)":/builds/contract \
	-v "$(pwd)"/target:/target \
	quay.io/subscan-explorer/wasm-compile-build:amd64-stable-v3.2.0  \
	make build-all
```

The reason to build contracts with this command is to allow for _reproducible builds_ (ink! contracts' builds are not deterministic).


## How to verify

Check out the repository at commit `TODO` and in the root of the project run the command above. This will output contracts' builds to `/target/ink` directory. 

For every contract there's a separate folder in which you will find `<contract>.json` containing contract's metadata. One of the keys is `source.hash`. Compare that to the code hash of the on-chain contract.