## Stablepool deployment

1. Export wallet to a JSON format and save it as `scripts/amm/stablepool/deployerWallet.json`.
2. Create `scripts/secrets.json` and as shown in `scripts/amm/stablepool/secrets.example.json`.
3. Create `scripts/deploymentPoolsParams.json` and add deployment parameters as shown in `scripts/amm/stablepool/deploymentPoolsParams.example.json`.
4. Run deployment script

```
npm run deploy-stable
```

To run the script with `*.example.json` files, run

```
npm run deploy-stable example`
```
