# rust-chain

[![CI](https://github.com/paologalligit/rust-chain/actions/workflows/ci.yml/badge.svg)](https://github.com/paologalligit/rust-chain/actions/workflows/ci.yml)

A small Rust proof of work blockchain prototype for exploring chain validation, transaction identity, mempool ordering, and secp256k1 signatures. It is a learning project with explicit invariants and an executable mining demo. It is **not** a cryptocurrency node or a system for real funds.

## Run it

Prerequisites: a recent stable Rust toolchain with Cargo, rustfmt, and Clippy.

```sh
cargo run --locked
```

The binary mines four empty blocks at a fixed 8-bit demonstration difficulty and prints each height and nonce. It keeps state in memory; every run starts from the same deterministic genesis block.

## What is implemented

| Component | Current behavior |
| --- | --- |
| Blocks and mining | SHA-256 over a JSON block preimage; nonce search; fixed leading-zero-bit difficulty; parent, height, hash, transaction identity, and proof checks before append. |
| History | In-memory chain beginning at a deterministic genesis block; validates a candidate chain before applying a naive longest-chain choice. |
| Transactions and wallet | Content-derived transaction ID, ECDSA signing and verification with `secp256k1`, in-memory key pairs. |
| Mempool | Bounded priority queue ordered by fee, amount, then transaction ID; duplicate IDs replace without evicting another entry. |

The implementation uses `serde_json` for a readable block preimage and `secp256k1` for the signature primitive. Those dependencies serve specific boundaries; no network or persistence framework is included until the protocol and storage contracts are defined.

## Quality gates

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
cargo doc --no-deps --locked
cargo build --release --locked
```

CI runs these checks on pull requests and pushes, enforces an 85% LLVM line-coverage floor, and scans dependencies against RustSec advisories. The coverage report is uploaded as a build artifact. A `v*` tag packages the Linux demo binary and SHA-256 checksum after all gates pass. The measured baseline for this revision is 93.49%, including the demo binary, which tests do not execute. For a local coverage report, install `cargo-llvm-cov` and the `llvm-tools-preview` Rust component, then run:

```sh
cargo llvm-cov --locked --workspace --all-targets --lcov --output-path lcov.info
```

`lcov.info` is ignored by Git. See [CONTRIBUTING.md](CONTRIBUTING.md) for the contribution workflow.

## Design and limits

The [architecture notes](docs/ARCHITECTURE.md) explain the current data flow and invariants. [Engineering challenges](docs/CHALLENGES.md) records tradeoffs and failure modes. The [roadmap](docs/ROADMAP.md) separates completed work from the next useful milestones, and the [release checklist](docs/RELEASE_CHECKLIST.md) makes the gap to a real node explicit.

This prototype has no peer-to-peer layer, persistent ledger, balances, transaction execution, difficulty adjustment, or account ownership rule linking a signature to `from`. The JSON hash format is illustrative rather than a versioned consensus serialization. A longer chain is chosen by block count, not cumulative work. Do not use it to secure assets.
