# ![Common logo](common_logo.svg "Common logo")


This repository contains implementations of AMM DEXes written for Common product.

There will be multiple AMM models implemented, each being the most suitable model for a certain token pair (stablecoin pairs being created in a CFM based on Curve StableSwap, PSP22 token pair on UniswapV2, etc.)

## Purpose

This is an unaudited full dex implementation ready to be used.

## Versions

[ink! 4.3.0](https://github.com/paritytech/ink/tree/v4.3.0)
`cargo-contract` in version `3.2.0`

## License

Apache 2.0

### 🏗️ How to use - Contracts

##### Cloning

Please note that this project contains two git submodules so the proper way to clone it will be `git clone --recurse-submodules git@github.com:Cardinal-Cryptography/common-amm.git`.

If you've already cloned the repo previously and want to just update, run `git submodule update --init --recursive`.

##### 💫 Build

Use these [instructions](https://use.ink/getting-started/setup) to set up your ink!/Rust environment.
To build all contracts, run this command from the project root directory:

```sh
make build-all
```

##### 💫 Build verifiably

Given a deployed set of contracts with some code hashes, it's possible to check that the contract has been produced from a certain version of the source code in this repo (say a given commit). To do so:

1. `git checkout $COMMIT`
2. `make build-dockerized`.

You can also run `make build-dockerized` and ensure that the generated code hashes are the same as the ones found on chian.

The contracts will be deployed using the same docker image as the one used for this procedure, which smooths out indeterminism in ink! contract compilation.

The reason to build contracts with this command is to allow for _reproducible builds_ (ink! contracts' builds are not deterministic).

##### How to verify

Check out the repository at commit `TODO` (after deployment) and in the root of the project run the command above. This will output contracts' builds to `/target/ink` directory. 

For every contract there's a separate folder in which you will find `<contract>.json` containing contract's metadata. One of the keys is `source.hash`. Compare that to the code hash of the on-chain contract.

##### 💫 Wrap

Use these [instructions](https://github.com/Cardinal-Cryptography/ink-wrapper#installation) to set up your `ink-wrapper` environment.
Once you have built your contracts, you can wrap them by running this command from the project root directory:

```sh
make wrap-all
```

You can also build and wrap the contracts in one step using:

```sh
make build-and-wrap-all
```

##### 💫 Run checks

Rust code checks and unit tests can be run from the root directory of the project:

```sh
make check-all
```

##### 💫 Run unit test

To manually run unit tests, use:

```sh
cargo test
```

##### 💫 Run E2E tests

To run the E2E test suite, execute the following command from the root directory of the project.

```sh
make e2e-tests-with-setup-and-teardown
```

This will:

- Build and wrap your contracts.
- Run a single node.
- Sequentially run all the E2E test cases with setup.
- Stop the node.

##### 💫 Run drink!-based integration tests

To run drink!-based integration test suite, currently implemented only for `Farm` contract, go to `./farm` directory and run `make all-drink` - this will set up the tests and run them.

For more information about the framework see [here](./farm/tests/README.md).

##### 💫 Deploy

First start your local node. You can do that by running `make start-node` in the root directory of the project.

To deploy contracts, execute `npm run deploy-local` in the root directory.

To create sample tokens, execute `npm run create-tokens-local`.
To register them as pairs in the DEX, run `npm run add-liquidity-local`.

Note that this requires rebuilding TypeScript wrappers first: `npm run compile`.

##### 💫 Help

You can see a list of available `make` recipes by running:

```sh
make help
```

## Acknowledgement

The contracts here implement a constant-product AMM based on the classical Uniswap V2 model. The initial version of the code has been developed by @HyunggyuJang, @PierreOssun and @shunsukew from Astar and was based on the Solidity implementation of UniswapV2. Since then, a multitude of fixes, enhancements, and alterations have been implemented to refine its functionality and address evolving needs. Through collaborative efforts and continuous improvement, the repository has undergone significant changes, paving the way for a more robust and efficient system that aligns with our current objectives.