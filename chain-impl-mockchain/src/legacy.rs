use crate::value::Value;

use cardano::address::{AddrType, ExtendedAddr, SpendingData};
use cardano::hdwallet::XPub;

pub use cardano::address::Addr as OldAddress;
use chain_core::mempack::{ReadBuf, ReadError, Readable};
use chain_core::property;
use chain_crypto::{Ed25519Bip32, PublicKey};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UtxoDeclaration {
    pub addrs: Vec<(OldAddressBytes, Value)>,
}

type OldAddressBytes = Vec<u8>;

pub fn oldaddress_from_xpub(address: &OldAddress, xpub: &PublicKey<Ed25519Bip32>) -> bool {
    match XPub::from_slice(xpub.as_ref()) {
        Err(_) => false,
        Ok(xpub_old) => {
            let a = address.deconstruct();
            let ea = ExtendedAddr::new(
                AddrType::ATPubKey,
                SpendingData::PubKeyASD(xpub_old),
                a.attributes.clone(),
            );
            ea == a
        }
    }
}

impl Readable for UtxoDeclaration {
    fn read<'a>(buf: &mut ReadBuf<'a>) -> Result<Self, ReadError> {
        let nb_entries = buf.get_u8()? as usize;
        if nb_entries >= 0xff {
            return Err(ReadError::StructureInvalid("nb entries".to_string()));
        }

        let mut addrs = Vec::with_capacity(nb_entries);
        for _ in 0..nb_entries {
            let value = Value::read(buf)?;
            let addr_size = buf.get_u16()? as usize;
            let mut addr_buf = vec![0u8; addr_size];
            addr_buf.copy_from_slice(buf.get_slice(addr_size)?);
            addrs.push((addr_buf, value))
        }

        Ok(UtxoDeclaration { addrs: addrs })
    }
}

impl property::Serialize for UtxoDeclaration {
    type Error = std::io::Error;
    fn serialize<W: std::io::Write>(&self, writer: W) -> Result<(), Self::Error> {
        use chain_core::packer::*;
        use std::io::Write;

        assert!(self.addrs.len() < 255);

        let mut codec = Codec::from(writer);
        codec.put_u8(self.addrs.len() as u8)?;
        for (b, v) in &self.addrs {
            v.serialize(&mut codec)?;
            codec.put_u16(b.len() as u16)?;
            codec.write_all(&b)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use quickcheck::{Arbitrary, Gen};

    impl Arbitrary for UtxoDeclaration {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let mut nb: usize = Arbitrary::arbitrary(g);
            nb = nb % 255;
            let mut addrs = Vec::with_capacity(nb);
            for _ in 0..nb {
                let value = Arbitrary::arbitrary(g);
                let addr = vec![Arbitrary::arbitrary(g), 1u8];
                addrs.push((addr, value))
            }

            UtxoDeclaration { addrs }
        }
    }
}
