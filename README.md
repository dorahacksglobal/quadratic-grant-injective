# Quadratic Voting Grant on Injetcive

A quadratic funding implementation on Injetcive. This implementation distributes matching funds based on quadratic voting and includes [grant distribution algorithm](https://github.com/dorahacksglobal/qf-grant-contract/blob/bsc-long-term/grant-distribution-algorithm-en.md) (also called "quadratic funding tax") to ensure fairer distribution.

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

```
injectived tx wasm store /var/artifacts/quadratic_grant-aarch64.wasm \
--from=$(echo $INJ_ADDRESS) \
--chain-id="injective-888" \
--yes --fees=1000000000000000inj --gas=2000000 \
--node=https://k8s.testnet.tm.injective.network:443
```
