# Quadratic Voting Grant on Injective

A quadratic funding implementation on Injective. This implementation distributes matching funds based on quadratic voting and includes [grant distribution algorithm](https://github.com/dorahacksglobal/qf-grant-contract/blob/bsc-long-term/grant-distribution-algorithm-en.md) (also called "quadratic progressive tax") to ensure fairer distribution.

For previous EVM implementations refer to this [repo](https://github.com/dorahacksglobal/qf-grant-contract/tree/bsc-long-term).

## Quick Start

[Setup Rust](https://rustup.rs/)

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-unknown-unknown
```

Run tests

```
cargo test
```

[Install Injectived](https://docs.injective.network/develop/tools/injectived/install)

## Scripts Entry

### initialize

Initialize the contract.

### start_round

Start a new round. The valut controlled by the program derrived address. If the init valut is not empty, the value will be treated as a fund in the round.

### set_fund

Set fund in a round.

### add_track

Register a new track.

### batch_upload_project

Register a project to the round.

### weighted_batch_vote

Vote to a project which you like.

### end_round

Only owenr of round can end a round.

### withdraw_all

After withdraw_grants be called, the administrator can withdraw the corresponding fee.

## Publish

build

```
cargo wasm
```

optimize

```
cargo run-script optimize
```

check

```
cargo run-script check
```

deploy

```sh
injectived tx wasm store /var/artifacts/quadratic_grant-aarch64.wasm \
--from=$(echo $INJ_ADDRESS) \
--chain-id="injective-888" \
--yes --fees=1005000000000000inj --gas=3000000 \
--node=https://k8s.testnet.tm.injective.network:443
```

```sh
export INJ_ADDRESS=inj1t68r9rqkrzdy2xdqmjj9mhxz3n7v480pmx52hz
export CONTRACT=inj1ns2vjmxe00guw75ctumc32k2q2e7qxqqwqsj73

START_ROUND='{"start_round":{"tax_adjustment_multiplier": 10, "donation_denom":"inj", "voting_unit": "10", "fund": "4000", "pubkey":[]}}'
yes 12345678 | injectived tx wasm execute $CONTRACT "$START_ROUND" \
--from=$(echo $INJ_ADDRESS) \
--chain-id="injective-888" \
--yes --fees=1000000000000000inj --gas=2000000 \
--node=https://k8s.testnet.tm.injective.network:443 \
--output json

ROUND='{"round":{"round_id": 2}}'
injectived query wasm contract-state smart $CONTRACT "$ROUND" \
--node=https://k8s.testnet.tm.injective.network:443 \
--output json
```

```sh
injectived tx gov submit-proposal wasm-store /var/artifacts/quadratic_grant-aarch64.wasm \
--title="Upload quadratic grant contract" \
--description="A quadratic funding implementation on Injective" \
--instantiate-everybody=true \
--deposit=1000000000000000000inj \
--run-as=inj1t68r9rqkrzdy2xdqmjj9mhxz3n7v480pmx52hz \
--gas=10000000 \
--chain-id=injective-1 \
--broadcast-mode=sync \
--yes \
--from inj1t68r9rqkrzdy2xdqmjj9mhxz3n7v480pmx52hz \
--gas-prices=500000000inj
```
