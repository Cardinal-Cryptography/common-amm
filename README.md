# DEX - UniswapV2
This folder contains the line by line implementation of [uniswap-v2 core](https://github.com/Uniswap/v2-core) and [uniswap-v2 periphery](https://github.com/Uniswap/v2-periphery) with its tests.

### Purpose
This is an unaudited full dex implementation ready to be used.

### Versions
[ink! 4.0.0](https://github.com/paritytech/ink/tree/v4.0.0)   
[openbrush 3.0.0](https://github.com/727-Ventures/openbrush-contracts/tree/3.0.0)

### License
Apache 2.0

## 🏗️ How to use - Contracts
##### 💫 Build
Use these [instructions](https://use.ink/getting-started/setup) to set up your ink!/Rust environment.
To build all contracts, run this command from the project root directory:

```sh
make build-all
```

If you want to build the contracts for E2E tests, use:

```sh
make build-all-for-e2e-tests
```

This will include the `e2e-tests` feature, which will allow you to run E2E tests in sequence on one node. **Do not use this for production!**

##### 💫 Wrap
Use these [instructions](https://github.com/Cardinal-Cryptography/ink-wrapper#installation) to set up your `ink-wrapper` environment.
Once you have built your contracts, you can wrap them by running this command from the project root directory:

```sh
make wrap-all
```

You can also build and wrap the contract in one step using:

```sh
make build-and-wrap-all
```

In case you want to build and wrap the contracts for E2E tests, you can use:

```sh
make build-and-wrap-all-for-e2e-tests
```

This will make the use of the `e2e-tests` feature. **Do not use for production!**

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
- Build and wrap your contracts for e2e tests (**not for production**).
- Run a single node.
- Sequentially run all the E2E test cases with setup and teardown.
- Stop the node.

##### 💫 Deploy
First start your local node. You can do that by running `make run-node` in the root directory of the project.

##### 💫 Help
You can see a list of available `make` recipes by running:

```sh
make help
```
