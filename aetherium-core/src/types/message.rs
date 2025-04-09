use serde::Serialize;
use sha3::{digest::Update, Digest, Keccak256};
use std::fmt::{Debug, Display, Formatter};

use crate::utils::{fmt_address_for_domain, fmt_domain};
use crate::{Decode, Encode, AetheriumProtocolError, H256};

const AETHERIUM_MESSAGE_PREFIX_LEN: usize = 77;

/// A message ID that has been delivered to the destination
pub type Delivery = H256;

/// A Stamped message that has been committed at some nonce
pub type RawAetheriumMessage = Vec<u8>;

impl From<&AetheriumMessage> for RawAetheriumMessage {
    fn from(m: &AetheriumMessage) -> Self {
        let mut message_vec = vec![];
        m.write_to(&mut message_vec).expect("!write_to");
        message_vec
    }
}

/// A full Aetherium message between chains
#[derive(Clone, Eq, PartialEq, Hash, Serialize)]
pub struct AetheriumMessage {
    /// 1   Aetherium version number
    pub version: u8,
    /// 4   Message nonce
    pub nonce: u32,
    /// 4   Origin domain ID
    pub origin: u32,
    /// 32  Address in origin convention
    pub sender: H256,
    /// 4   Destination domain ID
    pub destination: u32,
    /// 32  Address in destination convention
    pub recipient: H256,
    /// 0+  Message contents
    pub body: Vec<u8>,
}

impl Default for AetheriumMessage {
    fn default() -> Self {
        Self {
            // Use version 3 now that Aetherium V3 is the default
            version: 3,
            nonce: 0,
            origin: 0,
            sender: H256::zero(),
            destination: 0,
            recipient: H256::zero(),
            body: vec![],
        }
    }
}

impl Debug for AetheriumMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "AetheriumMessage {{ id: {:?}, version: {}, nonce: {}, origin: {}, sender: {}, destination: {}, recipient: {}, body: 0x{} }}",
            self.id(),
            self.version,
            self.nonce,
            fmt_domain(self.origin),
            fmt_address_for_domain(self.origin, self.sender),
            fmt_domain(self.destination),
            fmt_address_for_domain(self.destination, self.recipient),
            hex::encode(&self.body)
        )
    }
}

impl Display for AetheriumMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

impl From<RawAetheriumMessage> for AetheriumMessage {
    fn from(m: RawAetheriumMessage) -> Self {
        AetheriumMessage::from(&m)
    }
}

impl From<&RawAetheriumMessage> for AetheriumMessage {
    fn from(m: &RawAetheriumMessage) -> Self {
        let version = m[0];
        let nonce: [u8; 4] = m[1..5].try_into().unwrap();
        let origin: [u8; 4] = m[5..9].try_into().unwrap();
        let sender: [u8; 32] = m[9..41].try_into().unwrap();
        let destination: [u8; 4] = m[41..45].try_into().unwrap();
        let recipient: [u8; 32] = m[45..77].try_into().unwrap();
        let body = m[77..].into();
        Self {
            version,
            nonce: u32::from_be_bytes(nonce),
            origin: u32::from_be_bytes(origin),
            sender: H256::from(sender),
            destination: u32::from_be_bytes(destination),
            recipient: H256::from(recipient),
            body,
        }
    }
}

impl Encode for AetheriumMessage {
    fn write_to<W>(&self, writer: &mut W) -> std::io::Result<usize>
    where
        W: std::io::Write,
    {
        writer.write_all(&self.version.to_be_bytes())?;
        writer.write_all(&self.nonce.to_be_bytes())?;
        writer.write_all(&self.origin.to_be_bytes())?;
        writer.write_all(self.sender.as_ref())?;
        writer.write_all(&self.destination.to_be_bytes())?;
        writer.write_all(self.recipient.as_ref())?;
        writer.write_all(&self.body)?;
        Ok(AETHERIUM_MESSAGE_PREFIX_LEN + self.body.len())
    }
}

impl Decode for AetheriumMessage {
    fn read_from<R>(reader: &mut R) -> Result<Self, AetheriumProtocolError>
    where
        R: std::io::Read,
    {
        let mut version = [0u8; 1];
        reader.read_exact(&mut version)?;

        let mut nonce = [0u8; 4];
        reader.read_exact(&mut nonce)?;

        let mut origin = [0u8; 4];
        reader.read_exact(&mut origin)?;

        let mut sender = H256::zero();
        reader.read_exact(sender.as_mut())?;

        let mut destination = [0u8; 4];
        reader.read_exact(&mut destination)?;

        let mut recipient = H256::zero();
        reader.read_exact(recipient.as_mut())?;

        let mut body = vec![];
        reader.read_to_end(&mut body)?;

        Ok(Self {
            version: u8::from_be_bytes(version),
            nonce: u32::from_be_bytes(nonce),
            origin: u32::from_be_bytes(origin),
            sender,
            destination: u32::from_be_bytes(destination),
            recipient,
            body,
        })
    }
}

impl AetheriumMessage {
    /// Convert the message to a message id
    pub fn id(&self) -> H256 {
        H256::from_slice(Keccak256::new().chain(self.to_vec()).finalize().as_slice())
    }
}
