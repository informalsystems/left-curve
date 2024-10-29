use {
    crate::account_factory::Username,
    grug::{Addr, Binary, ByteArray, Hash160, Message},
};

/// A public key that can be associated with a [`Username`](crate::auth::Username).
#[grug::derive(Serde, Borsh)]
#[derive(Copy)]
pub enum Key {
    /// An Secp256r1 public key in compressed form.
    Secp256r1(ByteArray<33>),
    /// An Secp256k1 public key in compressed form.
    Secp256k1(ByteArray<33>),
}

/// Data that the account expects for the transaction's [`credential`](grug::Tx::credential)
/// field.
#[grug::derive(Serde)]
pub enum Credential {
    /// An Secp256r1 signature signed by a Passkey, along with necessary metadata.
    Passkey(PasskeySignature),
    /// An Secp256k1 signature.
    Secp256k1(ByteArray<64>),
    /// An EIP712 signature signed by a compatible eth wallet.
    Eip712(Eip712Signature),
}

/// Data that a transaction's sender must sign with their private key.
///
/// This includes the messages to be included in the transaction, as well as
/// chain ID, sender and account sequence number for replay protection.
#[grug::derive(Serde)]
pub struct SignDoc {
    pub sender: Addr,
    pub messages: Vec<Message>,
    pub chain_id: String,
    pub sequence: u32,
}

/// Data that the account expects for the transaction's [`data`](grug::Tx::data)
/// field.
#[grug::derive(Serde)]
pub struct Metadata {
    /// Identifies the user who signed this transaction.
    pub username: Username,
    /// Identifies the key which the user used to sign this transaction.
    pub key_hash: Hash160,
    /// The sequence number this transaction was signed with.
    pub sequence: u32,
}

/// An Secp256r1 signature generated by a Passkey via Webauthn, along with
/// necessary metadata.
#[grug::derive(Serde)]
pub struct PasskeySignature {
    pub authenticator_data: Binary,
    pub client_data: Binary,
    pub sig: ByteArray<64>,
}

/// An EIP712 signature signed with a compatible eth wallet.
#[grug::derive(Serde)]
pub struct Eip712Signature {
    /// The EIP712 typed data object containing type information,
    /// domain and the message object.
    pub typed_data: Binary,
    pub sig: ByteArray<64>,
}

/// Passkey client data.
#[grug::derive(Serde)]
pub struct ClientData {
    // Should be "webauthn.get".
    #[serde(rename = "type")]
    pub ty: String,
    // Should be the `SignDoc` in base64 `URL_SAFE_NO_PAD` encoding.
    pub challenge: String,
    pub origin: String,
    #[serde(rename = "crossOrigin")]
    pub cross_origin: bool,
}
