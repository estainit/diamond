#[cfg(test)]
pub mod tests_crypto {
    // use crate::lib::constants as cconsts;
    use crate::lib::ccrypto;

    // use bech32::{self, FromBase32, ToBase32, Variant};

    #[test]
    pub fn test_bech32() {
        // let encoded = bech32::encode("bech32", vec![0x00, 0x01, 0x02].to_base32(), Variant::Bech32).unwrap();
        // assert_eq!(encoded, "bech321qqqsyrhqy2a".to_string());
        // let (hrp, data, variant) = bech32::decode(&encoded).unwrap();
        // assert_eq!(hrp, "bech32");
        // assert_eq!(Vec::<u8>::from_base32(&data).unwrap(), vec![0x00, 0x01, 0x02]);
        // assert_eq!(variant, Variant::Bech32);
        assert_eq!(ccrypto::is_valid_bech32("TP_DP"), false);
        assert_eq!(ccrypto::is_valid_bech32("im1xpjkywf48yckgepcvdnrgdrx8qurgdeevf3kyenyv9snvve5v5ung9axujl"), true);
    }

    #[test]
    pub fn test_bech32_encode() {
        let bech_address = ccrypto::bech32_encode("test");
        assert_eq!(bech_address, "im1xp6x2um5kv6r6h");
    }

    #[test]
    pub fn test_keccak256()
    {
        // assert_eq!(ccrypto::keccak256(&"abc".to_string()), "3a985da74fe225b2045c172d6bd390bd855f086e3e9d525b46bfe24511431532");
    }

    #[test]
    pub fn test_b64()
    {
        assert_eq!(ccrypto::b64_encode(&"hello world".to_string()), "aGVsbG8gd29ybGQ=");
        assert_eq!(ccrypto::b64_decode(&"aGVsbG8gd29ybGQ=".to_string()), "hello world");
    }

    #[test]
    pub fn test_sha256()
    {
        // assert_eq!(ccrypto::do_sha256(&"imagine".to_string()), "7fdd65bbaaa687d48dc85a26a6dd7ef17fa379994fd8f3b26e19fe36976aeadf");
    }
}