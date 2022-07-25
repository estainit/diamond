#[cfg(test)]
pub mod tests_crypto_rsa {
    use crate::lib::ccrypto;

    #[test]
    pub fn test_js_generated_keys_1()
    {
        let pem_prv_key_1_generated_by_js: String = "-----BEGIN PRIVATE KEY-----\n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\nPFrnBwb6E/LcKjzkUat3PUFREuhAKQ==\n-----END PRIVATE KEY-----\n".to_string();
        assert!(ccrypto::rsa_is_valid_prv_key(&pem_prv_key_1_generated_by_js));
    }

    #[test]
    pub fn test_js_generated_keys_2()
    {
        let pem_pup_key_1_generated_by_js: String = "-----BEGIN PUBLIC KEY-----\nMIICIjANBgkqhkiG9w0BAQEFAAOCAg8AMIICCgKCAgEA1RG+nLSuYWszuVQL9ZaJ\nMflUZXlGfPKk+tmFxUnEGLKG4/QuTN/1m/Bm6AkFnHZkXWhGisyHG8ujgSAQZQnK\nUWsI+VGJ41YnqvxAKYIL3qvBSPLo8ppvN21tr7pbCL3uR0isHjXSp6XGH3mVBTd6\ntaJhRBtuQKdeFd3QMZCyofnaagA1wPHtT8wCz4X+7LckrfSRGhdjPUoT3pZ2R3Z8\noyAOtBzr7IRHDObs11Z5sdFmZVRQV1iSgxZyS3jEYjMqZN5FaxVYLq64MHIEYIdw\nLpofmWqkDrKUws9jTmiirmDfaAqsu6siHdCbCpnV026QMtbQukguJv3UFbdN/lh2\n2Obz9OKw802xMSgt4nULDSAvt8mrJsbyWbX66yVNkmN3OKiy36Ig9faCoxJTjzjW\nS5Kr7JXcBCyavog1n0NcNCOApde3TsoHNAt/5GJ8pMON2jG+i58Ug4/1mtz1tYEs\ndKFj4tbAVXgNPKNl0MlmReSjFati3K8H14twvOLsN0wnycWqJThwFCRFRfSV2weY\nw1w+k4hmsL0FvbZtl0OdQePvqbTQQTQc8SROc3Ejq/04oyc5S9D1MdaDEfdXqcqk\nnFzc3u3rzw1BPGdkw0LoiwDjp0WOSSB5u5NRI9UYxDOWdTaEPGpChKycm4kgUjYK\nvucjKoPGeLsBGmH8+NRT+RsCAwEAAQ==\n-----END PUBLIC KEY-----\n".to_string();
        assert!(ccrypto::rsa_is_valid_pub_key(&pem_pup_key_1_generated_by_js));
    }

    #[test]
    pub fn test_js_generated_keys_enc_dec()
    {
        let prv_key_1: String = "-----BEGIN PRIVATE KEY-----
MIHDAgEAMA0GCSqGSIb3DQEBAQUABIGuMIGrAgEAAiEAmejA9KUlMfQyiKAidMQr
d1kxyhf7C/FxJAXLg5Mfxb0CAwEAAQIgF/3YVkqICclvNyz/MgSCYdJ1paMZLEmH
Pj2c/Q4hGWECEQDBWV1seI3CFQRkqNvVbjJHAhEAy8fId2raVVxUbok6m3Wl2wIR
AKOxghv8sJ+5ZX/HSl8XLa8CEQC9bjhCGkhf9XZfUp3yicjlAhAls3hmNZLw0ncD
6u/ITGXF
-----END PRIVATE KEY-----".to_string();
        let pub_key_1: String = "-----BEGIN PUBLIC KEY-----
MDwwDQYJKoZIhvcNAQEBBQADKwAwKAIhAJnowPSlJTH0MoigInTEK3dZMcoX+wvx
cSQFy4OTH8W9AgMBAAE=
-----END PUBLIC KEY-----".to_string();
        let message_1: String = "Hi".to_string();

        assert!(ccrypto::rsa_is_valid_prv_key(&prv_key_1));
        assert!(ccrypto::rsa_is_valid_pub_key(&pub_key_1));
    }

    #[test]
    pub fn test_autogen_rsa_key_pairs()
    {
        let message: String = "Hi 1234567890123".to_string();
        let _signature: String = "".to_string();
        let _cipher: String = "".to_string();
        let _recovered: String = "".to_string();
        let _is_verified: bool = false;
        let (status, pem_prv_key, pem_pub_key) = ccrypto::rsa_generate_key_pair();
        assert!(status);

        assert!(ccrypto::rsa_is_valid_prv_key(&pem_prv_key));
        let regen_prv_key = ccrypto::rsa_read_pem_prv_key(&pem_prv_key);
        let regen_pem_prv = ccrypto::rsa_convert_prv_obj_to_pem_str(regen_prv_key);
        assert_eq!(regen_pem_prv, pem_prv_key);

        assert!(ccrypto::rsa_is_valid_pub_key(&pem_pub_key));

        let regen_pub_key = ccrypto::rsa_read_pem_pub_key(&pem_pub_key);
        let regen_pem_pub = ccrypto::rsa_convert_pub_obj_to_pem_str(regen_pub_key);
        assert_eq!(regen_pem_pub, pem_pub_key);

        // test signature & verification
        let (sign_status, signature) = ccrypto::rsa_sign(
            &pem_prv_key,
            &message,
        );
        assert!(sign_status);

        assert!(ccrypto::rsa_verify_signature(&pem_pub_key, &message, &signature));

        let (enc_status, enc_msg) = ccrypto::rsa_encrypt_with_pub_key(&pem_pub_key, &message);
        assert!(enc_status);

        let (dec_status, dec_msg) = ccrypto::rsa_decrypt_with_prv_key(&pem_prv_key, &enc_msg);
        assert!(dec_status);

        assert_eq!(dec_msg, message);

        // test auto encryption Pub -> Priv
        let (status, cipher) = ccrypto::rsa_encrypt_with_pub_key(
            &pem_pub_key,
            &message,
        );
        assert!(status);

        let (status, recovered) = ccrypto::rsa_decrypt_with_prv_key(
            &pem_prv_key,
            &cipher,
        );
        assert_eq!(recovered, message);
    }
}