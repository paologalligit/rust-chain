# Consensus encoding v1

This document defines the bytes hashed and encoded by the in-memory prototype. It is a protocol definition for this repository, not a claim of network interoperability. The Rust implementation is in `src/core/codec.rs`; `tests/codec.rs` locks the examples below.

## Common rules

- All integer fields are fixed-width **big-endian**. There are no optional fields beyond the signature flag and no padding.
- Magic values are four ASCII bytes. Each transaction and block carries a **u16 version**, currently `1`. Unknown versions are rejected; a future version needs a new specification and migration rule.
- Strings are a u64 byte length followed by exactly that many UTF-8 bytes. No Unicode normalization, case folding, trimming, or null termination occurs. Distinct UTF-8 byte sequences are distinct identities.
- Hashes and transaction IDs are 32 raw bytes on the wire. The Rust API displays them as exactly 64 lowercase hexadecimal characters. Uppercase, short, and malformed hex are rejected at the API boundary.
- A decoder must consume the entire input, reject truncated or extra bytes, and reject lengths that exceed the available input or the platform's addressable memory.
- SHA-256 produces every 32-byte ID and hash. Domain tags and versions are included in hashed bytes.

## Transaction

The **unsigned body** is:

| Order | Field | Width / encoding |
| --- | --- | --- |
| 1 | Magic | ASCII `RCTX` (4 bytes) |
| 2 | Version | u16, value `1` |
| 3 | `from` | u64 byte length + UTF-8 bytes |
| 4 | `to` | u64 byte length + UTF-8 bytes |
| 5 | Amount | u64 |
| 6 | Fee | u64 |

The transaction ID (currently called `nonce` in the Rust model) is `SHA256(unsigned body)`. The **complete transaction** appends the 32-byte ID, then one signature flag byte: `00` means unsigned; `01` means exactly 64 more bytes of compact secp256k1 ECDSA signature (`r || s`). Other flags and high-S signatures are noncanonical. Signing and verification use `SHA256("RCTS" || unsigned body || transaction ID)` as the 32-byte message digest. Changing any unsigned field invalidates the ID and signature. Adding a signature leaves the ID unchanged but changes the complete transaction bytes.

Block and mempool admission check that the ID matches the fields. Sender ownership and mandatory signatures are **not** consensus rules yet; they are M2 work.

### Fixed unsigned transaction example

For `from="alice"`, `to="bob"`, amount `7`, fee `2`, no signature:

```text
unsigned body:
5243545800010000000000000005616c6963650000000000000003626f62
00000000000000070000000000000002
ID = SHA256(unsigned body):
00a32b36f156a60f5dd7bc2587bd2391ec249ad4b6a607aca5943242f986edd7
signing digest:
b6bc902cf3ac29da87cde4654708169882c17c7f57abeee3c444a93dd02ed4df
complete transaction = unsigned body || ID || 00
```

## Block

The **block preimage** is:

| Order | Field | Width / encoding |
| --- | --- | --- |
| 1 | Magic | ASCII `RCBL` (4 bytes) |
| 2 | Version | u16, value `1` |
| 3 | Height | u64 |
| 4 | Previous block hash | 32 raw bytes |
| 5 | Timestamp | signed i64, Unix seconds |
| 6 | Transaction count | u32 |
| 7 | Transactions | For each, u64 length + complete transaction bytes |
| 8 | Mining nonce | u64 |

The block hash is `SHA256(block preimage)`. The **complete block** appends its 32-byte hash. Including the complete transaction bytes commits the block to signatures as well as unsigned fields. Changing a signature therefore changes the block hash. Decoding checks the byte format and transaction IDs; `Block::verify` additionally checks the parent link, height, proof target, and recomputed hash.

The v1 genesis block has height, timestamp, nonce, and transaction count all zero, with a zero previous hash. Its preimage hash is `b731ce283a29a28b935716d1d7b947a1b7524d3eedc0afc89034b3de274c1268`. Genesis is a trusted anchor and is exempt from mining. Other blocks currently require eight leading zero bits.

## Compatibility and open rules

This is an intentional break from the earlier JSON-hashed prototype. Existing JSON-era hashes and chains cannot be mixed with v1 blocks. There is no persisted chain to migrate today. Transaction replay rules, address ownership, timestamp limits, and cumulative-work fork choice remain separate roadmap items. The fixed examples here are checked by the Rust tests; cross-implementation or independent-decoder vectors remain the next M1 item.
