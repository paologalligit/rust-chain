# Engineering challenges

These are the problems that shaped this iteration, plus the boundaries still open.

## Correctness issues addressed

- **Validation result discarded:** `Block::verify` could return `Ok(false)`, while history appended the block anyway. History now checks that result and tests confirm invalid link, height, hash, and transaction rejection without mutation.
- **Ambiguous proof bits:** formatting each byte as unpadded binary made the prefix length depend on byte values. Mining and verification now count actual leading zero bits in bytes.
- **Mempool ordering contract:** different transactions with the same fee and amount compared equal in `Ord` but unequal in `Eq`, so `BTreeSet` silently lost entries. The transaction ID now breaks ties, and a regression test covers the case.
- **Capacity edge cases:** a duplicate could evict another transaction; capacity zero could panic. Admission checks duplicates first and rejects candidates that do not outrank the current worst entry.
- **Genesis drift:** the old genesis timestamp changed on every construction, while its hash stayed constant. Genesis is now deterministic and hash-consistent.

## Open design work

| Challenge | Why it matters | Direction |
| --- | --- | --- |
| Canonical encoding | JSON is not an interoperable consensus format. | Specify versioned bytes, field ordering, integer widths, and test vectors. |
| Identity and authorization | `from` is an arbitrary string and block validation does not verify signatures. | Define address derivation, sender key binding, and signature requirements. |
| State transition | Validly hashed transactions do not move balances or prevent overspend. | Add a deterministic account or UTXO model with replay and overflow checks. |
| Fork choice | Length alone is insufficient when difficulty changes. | Track cumulative work and define tie behavior. |
| Persistence and networking | The chain disappears on exit and cannot synchronize with peers. | Design storage and peer protocols after consensus rules are stable. |
| Error detail | Current public errors are intentionally small and generic. | Introduce typed validation causes as APIs mature. |

See [ROADMAP.md](ROADMAP.md) for the proposed order. These gaps are why the project is labeled a prototype rather than a production blockchain.
