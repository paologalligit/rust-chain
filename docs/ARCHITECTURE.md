# Architecture and invariants

## Data flow

```text
Transaction::new -> validate -> MemPool (fee order)
                         |             |
                         |             +-- take transactions for candidate
                         v
                   mine_new_block -> Block::verify -> History::try_to_append
                                                   -> History::choose_chain
```

`main.rs` is a four-block demo. The reusable code lives in `src/core`: `models` holds block and transaction data, `hashing` and `mining` implement the proof, `history` owns the chain, `memory_pool` owns pending transactions, and `wallet` holds in-memory key pairs. There is no networking or database boundary yet.

## Rules enforced today

1. Genesis is deterministic: height 0, timestamp 0, nonce 0, and a hash of its own preimage. It is trusted as the chain anchor; it does not need a mined proof.
2. A child block must point to the current tip, increment height by one, contain transactions whose content-derived IDs match their fields, and carry a 32-byte SHA-256 hash satisfying the fixed 8-bit target. Its hash must match the recalculated preimage.
3. An invalid append returns an error without changing the chain. Candidate reorganization chains must start with the same genesis and pass every adjacent-block check before selection.
4. The mempool indexes transactions by ID and by a total priority order: higher fee, then higher amount, then lexicographic ID. At capacity, an equal or worse candidate is rejected; a better candidate evicts the worst. Replacing an existing ID does not evict.
5. Signatures can be created and verified against an explicitly supplied public key. The ledger does not yet prove that this public key owns the `from` address.

`History::get_height()` currently returns the **number of blocks**, including genesis; the tip block's `height` is therefore one less. This API is kept for compatibility with the existing example, but a future API should distinguish `len()` from tip height.

## Deliberate tradeoffs

- The JSON preimage is easy to inspect, but it is not a stable consensus encoding. Changes to field names, ordering, or serialization libraries can change hashes. A versioned byte encoding is needed before interoperability.
- The difficulty is fixed at 8 leading zero bits so tests and the demo finish quickly. It gives no meaningful attack resistance.
- The chain-selection policy uses length because every non-genesis block currently has the same difficulty. Cumulative-work fork choice is required before variable difficulty.
- All state is in memory. Persistence needs atomic writes, recovery checks, and a migration strategy.
