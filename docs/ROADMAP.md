# Roadmap

A prioritized plan, not a promise of release dates. Each milestone has a concrete exit condition.

## M0 — trustworthy prototype (current)

- [x] Deterministic genesis and checked block append
- [x] Correct leading-zero-bit proof test
- [x] Mempool ordering and capacity invariants
- [x] Unit and integration regression tests
- [x] Formatting, lint, test, release-build, coverage, advisory, and tagged-package CI jobs
- [x] Architecture, challenges, contribution, and readiness documentation

## M1 — consensus specification

- [ ] Define a canonical binary transaction and block encoding with version fields
- [ ] Publish fixed hash and signature test vectors across two implementations or independent decoders
- [ ] Define transaction uniqueness, replay rules, and timestamp limits
- [ ] Replace generic validation errors with typed, testable failure causes
- [ ] Add property-based tests for encoding and state invariants

Exit condition: the same bytes produce the same hashes and validity result across implementations.

## M2 — authenticated state

- [ ] Bind `from` to a public key or address derivation rule and require signatures in block validation
- [ ] Add deterministic balance or UTXO transitions with overflow and double-spend protection
- [ ] Make mempool admission reuse consensus transaction checks
- [ ] Add state-transition and adversarial transaction tests

Exit condition: replaying a chain from genesis yields the same state and rejects unauthorized spending.

## M3 — durable single node

- [ ] Add versioned storage with atomic commit and crash recovery
- [ ] Add cumulative-work fork choice and configurable difficulty policy
- [ ] Add observability for mining, validation, and storage failures
- [ ] Define migration, backup, and recovery tests

Exit condition: a node restarts, verifies persisted data, and reaches the same tip and state.

## M4 — networked experiment

- [ ] Specify peer messages, validation budgets, and synchronization protocol
- [ ] Add integration tests with multiple isolated nodes and hostile input
- [ ] Complete the [release checklist](RELEASE_CHECKLIST.md) before any public network claim

Exit condition: nodes converge in repeatable tests under forks and malformed messages.
