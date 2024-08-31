use serde::{Deserialize, Deserializer, Serialize};
use serde::de::{SeqAccess, Visitor};

pub mod event_handler;

type Sec256k1PublicKey = [u8; 32];
type DisruptedDataRecord = Vec<u8>;
type Sec256k1Signature = [u8; 64];

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) enum Group {
    Nostr,
    Git,
}

