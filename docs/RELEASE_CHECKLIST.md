# Release readiness checklist

This is a gate for any future claim that the software is fit to run as a networked node. It is deliberately incomplete today.

## Current repository gate

- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --all-targets --locked -- -D warnings`
- [x] `cargo test --locked` with negative-path regression tests
- [x] `cargo build --release --locked`
- [x] CI coverage report as an artifact
- [x] Automated RustSec advisory scan and dependency update proposals
- [x] Gated Linux binary artifact for version tags
- [x] Coverage threshold calibrated from a 93.49% local line-coverage baseline (CI floor: 85%)

## Protocol and operations gate

- [ ] Versioned canonical encoding and independent test vectors
- [ ] Sender authorization and deterministic state transitions
- [ ] Replay, double-spend, overflow, and malformed-input tests
- [ ] Cumulative-work fork choice and time/difficulty rules
- [ ] Durable state, crash recovery, backup, and migrations
- [ ] Peer security limits, synchronization, and multi-node tests
- [ ] Threat model and independent security review
- [ ] Documented incident and rollback procedures

Passing the repository gate alone is not a security or production-readiness claim.
