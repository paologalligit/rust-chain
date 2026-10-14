//! Canonical v1 consensus bytes. See docs/CONSENSUS_ENCODING.md.

use std::{error::Error, fmt};

use secp256k1::ecdsa::Signature;

use super::{hashing::hash_bytes, Block, Transaction};

pub const TRANSACTION_VERSION: u16 = 1;
pub const BLOCK_VERSION: u16 = 1;

const TX_MAGIC: &[u8; 4] = b"RCTX";
const SIGN_MAGIC: &[u8; 4] = b"RCTS";
const BLOCK_MAGIC: &[u8; 4] = b"RCBL";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodecError {
    UnexpectedEof,
    TrailingBytes,
    InvalidMagic,
    UnsupportedVersion(u16),
    InvalidUtf8,
    InvalidHash,
    InvalidTransactionId,
    InvalidSignature,
    LengthOverflow,
}

impl fmt::Display for CodecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl Error for CodecError {}

pub(crate) fn parse_hash(value: &str) -> Result<[u8; 32], CodecError> {
    let decoded = hex::decode(value).map_err(|_| CodecError::InvalidHash)?;
    let bytes: [u8; 32] = decoded.try_into().map_err(|_| CodecError::InvalidHash)?;
    if hex::encode(bytes) != value {
        return Err(CodecError::InvalidHash);
    }
    Ok(bytes)
}

fn put_string(out: &mut Vec<u8>, value: &str) -> Result<(), CodecError> {
    let length = u64::try_from(value.len()).map_err(|_| CodecError::LengthOverflow)?;
    out.extend_from_slice(&length.to_be_bytes());
    out.extend_from_slice(value.as_bytes());
    Ok(())
}

pub(crate) fn transaction_body(
    version: u16,
    from: &str,
    to: &str,
    amount: u64,
    fee: u64,
) -> Result<Vec<u8>, CodecError> {
    if version != TRANSACTION_VERSION {
        return Err(CodecError::UnsupportedVersion(version));
    }
    let mut out = Vec::new();
    out.extend_from_slice(TX_MAGIC);
    out.extend_from_slice(&version.to_be_bytes());
    put_string(&mut out, from)?;
    put_string(&mut out, to)?;
    out.extend_from_slice(&amount.to_be_bytes());
    out.extend_from_slice(&fee.to_be_bytes());
    Ok(out)
}

pub(crate) fn transaction_id(tx: &Transaction) -> Result<[u8; 32], CodecError> {
    Ok(hash_bytes(&transaction_body(
        tx.version,
        tx.from.as_str(),
        tx.to.as_str(),
        tx.amount,
        tx.fee,
    )?))
}

pub(crate) fn signing_digest(tx: &Transaction) -> Result<[u8; 32], CodecError> {
    let mut bytes = Vec::from(SIGN_MAGIC.as_slice());
    bytes.extend_from_slice(&transaction_body(
        tx.version,
        tx.from.as_str(),
        tx.to.as_str(),
        tx.amount,
        tx.fee,
    )?);
    bytes.extend_from_slice(&parse_hash(&tx.nonce)?);
    Ok(hash_bytes(&bytes))
}

fn canonical_signature(signature: &Signature) -> bool {
    let mut normalized = *signature;
    normalized.normalize_s();
    normalized == *signature
}

impl Transaction {
    /// Encodes the complete transaction, including its ID and optional signature.
    pub fn to_bytes(&self) -> Result<Vec<u8>, CodecError> {
        let mut bytes = transaction_body(
            self.version,
            self.from.as_str(),
            self.to.as_str(),
            self.amount,
            self.fee,
        )?;
        let id = parse_hash(&self.nonce).map_err(|_| CodecError::InvalidTransactionId)?;
        if id != transaction_id(self)? {
            return Err(CodecError::InvalidTransactionId);
        }
        bytes.extend_from_slice(&id);
        match &self.signature {
            None => bytes.push(0),
            Some(signature) if canonical_signature(signature) => {
                bytes.push(1);
                bytes.extend_from_slice(&signature.serialize_compact());
            }
            Some(_) => return Err(CodecError::InvalidSignature),
        }
        Ok(bytes)
    }

    /// Parses one complete canonical v1 transaction.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CodecError> {
        let mut decoder = Decoder::new(bytes);
        decoder.magic(TX_MAGIC)?;
        let version = decoder.u16()?;
        if version != TRANSACTION_VERSION {
            return Err(CodecError::UnsupportedVersion(version));
        }
        let from = decoder.string()?;
        let to = decoder.string()?;
        let amount = decoder.u64()?;
        let fee = decoder.u64()?;
        let id = decoder.array::<32>()?;
        let signature = match decoder.u8()? {
            0 => None,
            1 => {
                let signature = Signature::from_compact(&decoder.array::<64>()?)
                    .map_err(|_| CodecError::InvalidSignature)?;
                if !canonical_signature(&signature) {
                    return Err(CodecError::InvalidSignature);
                }
                Some(signature)
            }
            _ => return Err(CodecError::InvalidSignature),
        };
        decoder.finish()?;
        let tx = Transaction {
            version,
            nonce: hex::encode(id),
            from,
            to,
            amount,
            fee,
            signature,
        };
        if transaction_id(&tx)? != id {
            return Err(CodecError::InvalidTransactionId);
        }
        Ok(tx)
    }
}

pub(crate) fn block_preimage(
    version: u16,
    height: u64,
    previous_hash: &str,
    timestamp: i64,
    txs: &[Transaction],
    nonce: u64,
) -> Result<Vec<u8>, CodecError> {
    let mut out = block_prefix(version, height, previous_hash, timestamp, txs)?;
    out.extend_from_slice(&nonce.to_be_bytes());
    Ok(out)
}

pub(crate) fn block_prefix(
    version: u16,
    height: u64,
    previous_hash: &str,
    timestamp: i64,
    txs: &[Transaction],
) -> Result<Vec<u8>, CodecError> {
    if version != BLOCK_VERSION {
        return Err(CodecError::UnsupportedVersion(version));
    }
    let count = u32::try_from(txs.len()).map_err(|_| CodecError::LengthOverflow)?;
    let mut out = Vec::new();
    out.extend_from_slice(BLOCK_MAGIC);
    out.extend_from_slice(&version.to_be_bytes());
    out.extend_from_slice(&height.to_be_bytes());
    out.extend_from_slice(&parse_hash(previous_hash)?);
    out.extend_from_slice(&timestamp.to_be_bytes());
    out.extend_from_slice(&count.to_be_bytes());
    for tx in txs {
        let bytes = tx.to_bytes()?;
        let length = u64::try_from(bytes.len()).map_err(|_| CodecError::LengthOverflow)?;
        out.extend_from_slice(&length.to_be_bytes());
        out.extend_from_slice(&bytes);
    }
    Ok(out)
}

impl Block {
    /// Returns the bytes committed by this block's hash.
    pub fn preimage_bytes(&self) -> Result<Vec<u8>, CodecError> {
        block_preimage(
            self.version,
            self.height,
            &self.previous_hash,
            self.timestamp,
            &self.txs,
            self.nonce,
        )
    }

    /// Encodes the complete block, appending its declared hash to the preimage.
    pub fn to_bytes(&self) -> Result<Vec<u8>, CodecError> {
        let mut out = self.preimage_bytes()?;
        out.extend_from_slice(&parse_hash(&self.hash)?);
        Ok(out)
    }

    /// Parses one complete canonical v1 block. Use `verify` for chain rules.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CodecError> {
        let mut decoder = Decoder::new(bytes);
        decoder.magic(BLOCK_MAGIC)?;
        let version = decoder.u16()?;
        if version != BLOCK_VERSION {
            return Err(CodecError::UnsupportedVersion(version));
        }
        let height = decoder.u64()?;
        let previous_hash = hex::encode(decoder.array::<32>()?);
        let timestamp = decoder.i64()?;
        let tx_count = decoder.u32()?;
        let mut txs = Vec::new();
        for _ in 0..tx_count {
            let bytes = decoder.length_prefixed()?;
            txs.push(Transaction::from_bytes(bytes)?);
        }
        let nonce = decoder.u64()?;
        let hash = hex::encode(decoder.array::<32>()?);
        decoder.finish()?;
        Ok(Block {
            version,
            height,
            hash,
            previous_hash,
            timestamp,
            txs,
            nonce,
        })
    }
}

struct Decoder<'a> {
    bytes: &'a [u8],
    cursor: usize,
}

impl<'a> Decoder<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, cursor: 0 }
    }

    fn take(&mut self, length: usize) -> Result<&'a [u8], CodecError> {
        let end = self
            .cursor
            .checked_add(length)
            .ok_or(CodecError::LengthOverflow)?;
        let result = self
            .bytes
            .get(self.cursor..end)
            .ok_or(CodecError::UnexpectedEof)?;
        self.cursor = end;
        Ok(result)
    }

    fn array<const N: usize>(&mut self) -> Result<[u8; N], CodecError> {
        self.take(N)?
            .try_into()
            .map_err(|_| CodecError::UnexpectedEof)
    }

    fn magic(&mut self, expected: &[u8; 4]) -> Result<(), CodecError> {
        if self.take(4)? != expected {
            return Err(CodecError::InvalidMagic);
        }
        Ok(())
    }

    fn u8(&mut self) -> Result<u8, CodecError> {
        Ok(self.array::<1>()?[0])
    }

    fn u16(&mut self) -> Result<u16, CodecError> {
        Ok(u16::from_be_bytes(self.array()?))
    }

    fn u32(&mut self) -> Result<u32, CodecError> {
        Ok(u32::from_be_bytes(self.array()?))
    }

    fn u64(&mut self) -> Result<u64, CodecError> {
        Ok(u64::from_be_bytes(self.array()?))
    }

    fn i64(&mut self) -> Result<i64, CodecError> {
        Ok(i64::from_be_bytes(self.array()?))
    }

    fn length_prefixed(&mut self) -> Result<&'a [u8], CodecError> {
        let length = usize::try_from(self.u64()?).map_err(|_| CodecError::LengthOverflow)?;
        self.take(length)
    }

    fn string(&mut self) -> Result<String, CodecError> {
        let bytes = self.length_prefixed()?;
        std::str::from_utf8(bytes)
            .map(str::to_owned)
            .map_err(|_| CodecError::InvalidUtf8)
    }

    fn finish(&self) -> Result<(), CodecError> {
        if self.cursor != self.bytes.len() {
            return Err(CodecError::TrailingBytes);
        }
        Ok(())
    }
}
