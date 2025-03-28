use crate::Address;
use crate::Digest;

use blake2::Digest as DigestTrait;

type Blake2b256 = blake2::Blake2b<blake2::digest::consts::U32>;

#[derive(Debug, Default)]
pub struct Hasher(Blake2b256);

impl Hasher {
    pub fn new() -> Self {
        Self(Blake2b256::new())
    }

    pub fn update<T: AsRef<[u8]>>(&mut self, data: T) {
        self.0.update(data)
    }

    /// Retrieve result and consume hasher instance.
    pub fn finalize(self) -> Digest {
        let mut buf = [0; Digest::LENGTH];
        let result = self.0.finalize();

        buf.copy_from_slice(result.as_slice());

        Digest::new(buf)
    }

    pub fn digest<T: AsRef<[u8]>>(data: T) -> Digest {
        let mut hasher = Self::new();
        hasher.update(data);
        hasher.finalize()
    }
}

impl std::io::Write for Hasher {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.0.flush()
    }
}

impl crate::Ed25519PublicKey {
    pub fn to_address(&self) -> Address {
        let mut hasher = Hasher::new();
        self.write_into_hasher(&mut hasher);
        let digest = hasher.finalize();
        Address::new(digest.into_inner())
    }

    fn write_into_hasher(&self, hasher: &mut Hasher) {
        hasher.update([self.scheme().to_u8()]);
        hasher.update(self.inner());
    }
}

impl crate::Secp256k1PublicKey {
    pub fn to_address(&self) -> Address {
        let mut hasher = Hasher::new();
        self.write_into_hasher(&mut hasher);
        let digest = hasher.finalize();
        Address::new(digest.into_inner())
    }

    fn write_into_hasher(&self, hasher: &mut Hasher) {
        hasher.update([self.scheme().to_u8()]);
        hasher.update(self.inner());
    }
}

impl crate::Secp256r1PublicKey {
    pub fn to_address(&self) -> Address {
        let mut hasher = Hasher::new();
        self.write_into_hasher(&mut hasher);
        let digest = hasher.finalize();
        Address::new(digest.into_inner())
    }

    fn write_into_hasher(&self, hasher: &mut Hasher) {
        hasher.update([self.scheme().to_u8()]);
        hasher.update(self.inner());
    }
}

impl crate::ZkLoginPublicIdentifier {
    /// Define as iss_bytes_len || iss_bytes || padded_32_byte_address_seed.
    pub fn to_address_padded(&self) -> Address {
        let mut hasher = Hasher::new();
        self.write_into_hasher_padded(&mut hasher);
        let digest = hasher.finalize();
        Address::new(digest.into_inner())
    }

    fn write_into_hasher_padded(&self, hasher: &mut Hasher) {
        hasher.update([self.scheme().to_u8()]);
        hasher.update([self.iss().len() as u8]); // TODO enforce iss is less than 255 bytes
        hasher.update(self.iss());
        hasher.update(self.address_seed().padded());
    }

    /// Define as iss_bytes_len || iss_bytes || unpadded_32_byte_address_seed.
    pub fn to_address_unpadded(&self) -> Address {
        let mut hasher = Hasher::new();
        hasher.update([self.scheme().to_u8()]);
        hasher.update([self.iss().len() as u8]); // TODO enforce iss is less than 255 bytes
        hasher.update(self.iss());
        hasher.update(self.address_seed().unpadded());
        let digest = hasher.finalize();
        Address::new(digest.into_inner())
    }
}

impl crate::PasskeyPublicKey {
    pub fn to_address(&self) -> Address {
        let mut hasher = Hasher::new();
        self.write_into_hasher(&mut hasher);
        let digest = hasher.finalize();
        Address::new(digest.into_inner())
    }

    fn write_into_hasher(&self, hasher: &mut Hasher) {
        hasher.update([self.scheme().to_u8()]);
        hasher.update(self.inner().inner());
    }
}

impl crate::MultisigCommittee {
    /// Derive an Address from a MultisigCommittee. A MultiSig address
    /// is defined as the 32-byte Blake2b hash of serializing the flag, the
    /// threshold, concatenation of all n flag, public keys and
    /// its weight. `flag_MultiSig || threshold || flag_1 || pk_1 || weight_1
    /// || ... || flag_n || pk_n || weight_n`.
    ///
    /// When flag_i is ZkLogin, pk_i refers to [struct ZkLoginPublicIdentifier]
    /// derived from padded address seed in bytes and iss.
    pub fn to_address(&self) -> Address {
        use crate::MultisigMemberPublicKey::*;

        let mut hasher = Hasher::new();
        hasher.update([self.scheme().to_u8()]);
        hasher.update(self.threshold().to_le_bytes());

        for member in self.members() {
            match member.public_key() {
                Ed25519(p) => p.write_into_hasher(&mut hasher),
                Secp256k1(p) => p.write_into_hasher(&mut hasher),
                Secp256r1(p) => p.write_into_hasher(&mut hasher),
                ZkLogin(p) => p.write_into_hasher_padded(&mut hasher),
            }

            hasher.update(member.weight().to_le_bytes());
        }

        let digest = hasher.finalize();
        Address::new(digest.into_inner())
    }
}

#[cfg(feature = "serde")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "serde")))]
mod type_digest {
    use super::Hasher;
    use crate::CheckpointContents;
    use crate::CheckpointContentsDigest;
    use crate::CheckpointDigest;
    use crate::CheckpointSummary;
    use crate::Digest;
    use crate::Object;
    use crate::ObjectDigest;
    use crate::Transaction;
    use crate::TransactionDigest;
    use crate::TransactionEffects;
    use crate::TransactionEffectsDigest;
    use crate::TransactionEvents;
    use crate::TransactionEventsDigest;

    impl Object {
        pub fn digest(&self) -> ObjectDigest {
            const SALT: &str = "Object::";
            let digest = type_digest(SALT, self);
            ObjectDigest::new(digest.into_inner())
        }
    }

    impl CheckpointSummary {
        pub fn digest(&self) -> CheckpointDigest {
            const SALT: &str = "CheckpointSummary::";
            let digest = type_digest(SALT, self);
            CheckpointDigest::new(digest.into_inner())
        }
    }

    impl CheckpointContents {
        pub fn digest(&self) -> CheckpointContentsDigest {
            const SALT: &str = "CheckpointContents::";
            let digest = type_digest(SALT, self);
            CheckpointContentsDigest::new(digest.into_inner())
        }
    }

    impl Transaction {
        pub fn digest(&self) -> TransactionDigest {
            const SALT: &str = "TransactionData::";
            let digest = type_digest(SALT, self);
            TransactionDigest::new(digest.into_inner())
        }
    }

    impl TransactionEffects {
        pub fn digest(&self) -> TransactionEffectsDigest {
            const SALT: &str = "TransactionEffects::";
            let digest = type_digest(SALT, self);
            TransactionEffectsDigest::new(digest.into_inner())
        }
    }

    impl TransactionEvents {
        pub fn digest(&self) -> TransactionEventsDigest {
            const SALT: &str = "TransactionEvents::";
            let digest = type_digest(SALT, self);
            TransactionEventsDigest::new(digest.into_inner())
        }
    }

    fn type_digest<T: serde::Serialize>(salt: &str, ty: &T) -> Digest {
        let mut hasher = Hasher::new();
        hasher.update(salt);
        bcs::serialize_into(&mut hasher, ty).unwrap();
        hasher.finalize()
    }
}

#[cfg(feature = "serde")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "serde")))]
mod signing_message {
    use crate::hash::Hasher;
    use crate::Digest;
    use crate::Intent;
    use crate::IntentAppId;
    use crate::IntentScope;
    use crate::IntentVersion;
    use crate::PersonalMessage;
    use crate::SigningDigest;
    use crate::Transaction;

    impl Transaction {
        pub fn signing_digest(&self) -> SigningDigest {
            const INTENT: Intent = Intent {
                scope: IntentScope::TransactionData,
                version: IntentVersion::V0,
                app_id: IntentAppId::MySocial,
            };
            let digest = signing_digest(INTENT, self);
            digest.into_inner()
        }
    }

    fn signing_digest<T: serde::Serialize + ?Sized>(intent: Intent, ty: &T) -> Digest {
        let mut hasher = Hasher::new();
        hasher.update(intent.to_bytes());
        bcs::serialize_into(&mut hasher, ty).unwrap();
        hasher.finalize()
    }

    impl PersonalMessage<'_> {
        pub fn signing_digest(&self) -> SigningDigest {
            const INTENT: Intent = Intent {
                scope: IntentScope::PersonalMessage,
                version: IntentVersion::V0,
                app_id: IntentAppId::MySocial,
            };
            let digest = signing_digest(INTENT, &self.0);
            digest.into_inner()
        }
    }
}

/// A 1-byte domain separator for hashing Object ID in MySocial. It is starting from 0xf0
/// to ensure no hashing collision for any ObjectId vs Address which is derived
/// as the hash of `flag || pubkey`.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
#[cfg_attr(feature = "proptest", derive(test_strategy::Arbitrary))]
#[repr(u8)]
enum HashingIntent {
    #[cfg(feature = "serde")]
    ChildObjectId = 0xf0,
    RegularObjectId = 0xf1,
}

impl crate::ObjectId {
    /// Create an ObjectId from `TransactionDigest` and `count`.
    ///
    /// `count` is the number of objects that have been created during a transactions.
    pub fn derive_id(digest: crate::TransactionDigest, count: u64) -> Self {
        let mut hasher = Hasher::new();
        hasher.update([HashingIntent::RegularObjectId as u8]);
        hasher.update(digest);
        hasher.update(count.to_le_bytes());
        let digest = hasher.finalize();
        Self::new(digest.into_inner())
    }

    /// Derive an ObjectId for a Dynamic Child Object.
    ///
    /// hash(parent || len(key) || key || key_type_tag)
    #[cfg(feature = "serde")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "serde")))]
    pub fn derive_dynamic_child_id(&self, key_type_tag: &crate::TypeTag, key_bytes: &[u8]) -> Self {
        let mut hasher = Hasher::new();
        hasher.update([HashingIntent::ChildObjectId as u8]);
        hasher.update(self);
        hasher.update(
            u64::try_from(key_bytes.len())
                .expect("key_bytes must fit into a u64")
                .to_le_bytes(),
        );
        hasher.update(key_bytes);
        bcs::serialize_into(&mut hasher, key_type_tag)
            .expect("bcs serialization of `TypeTag` cannot fail");
        let digest = hasher.finalize();

        Self::new(digest.into_inner())
    }
}

#[cfg(test)]
mod test {
    use super::HashingIntent;
    use crate::SignatureScheme;
    use test_strategy::proptest;

    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::wasm_bindgen_test as test;

    impl HashingIntent {
        fn from_byte(byte: u8) -> Result<Self, u8> {
            match byte {
                0xf0 => Ok(Self::ChildObjectId),
                0xf1 => Ok(Self::RegularObjectId),
                invalid => Err(invalid),
            }
        }
    }

    #[proptest]
    fn hashing_intent_does_not_overlap_with_signature_scheme(intent: HashingIntent) {
        SignatureScheme::from_byte(intent as u8).unwrap_err();
    }

    #[proptest]
    fn signature_scheme_does_not_overlap_with_hashing_intent(scheme: SignatureScheme) {
        HashingIntent::from_byte(scheme.to_u8()).unwrap_err();
    }

    #[proptest]
    fn roundtrip_hashing_intent(intent: HashingIntent) {
        assert_eq!(Ok(intent), HashingIntent::from_byte(intent as u8));
    }
}
