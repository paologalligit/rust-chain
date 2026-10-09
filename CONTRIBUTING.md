# Contributing

Small changes that strengthen one invariant or advance one roadmap item are easiest to review. Open an issue or pull request with the behavior being changed, the expected result, and the evidence from tests.

## Local workflow

1. Install a recent stable Rust toolchain with rustfmt and Clippy.
2. Run `cargo fmt --all`, then `cargo clippy --all-targets --locked -- -D warnings` and `cargo test --locked`.
3. For consensus or mempool changes, add a regression test that would fail before the change, including a rejection path.
4. Run `cargo doc --no-deps --locked` and `cargo build --release --locked` before requesting review.
5. Explain any changes to block or transaction hashing in `docs/ARCHITECTURE.md` and add fixed test vectors when an encoding is introduced.

Coverage can be generated with `cargo llvm-cov --locked --workspace --all-targets --lcov --output-path lcov.info` after installing `cargo-llvm-cov` and `llvm-tools-preview`. CI uploads this report for inspection.

## Review checklist

- [ ] Tests demonstrate the intended behavior and failure mode
- [ ] Public API and design documents match the implementation
- [ ] No new consensus assumptions are implicit
- [ ] Local quality commands pass
- [ ] Scope and remaining risks are stated in the pull request
