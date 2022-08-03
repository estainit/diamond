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
    pub fn test_cpp_generated_keys_1()
    {
        let aaa_pgp_private_key: String = "-----BEGIN PRIVATE KEY-----\n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\nmNI4cf8hgrmIb2AgIyZ9LJMhSDaF+g==\n-----END PRIVATE KEY-----".to_string();
        assert!(ccrypto::rsa_is_valid_prv_key(&aaa_pgp_private_key));
        let aaa_pgp_public_key: String = "-----BEGIN PUBLIC KEY-----\nMIICIDANBgkqhkiG9w0BAQEFAAOCAg0AMIICCAKCAgEAjSeu+Qny40lpzFfDy1XN\nWYyNHAldVUwH2I++PwV+xN9LE0JIbg3kh4BMHB5zN468qmlvE8QvTYbA2NNkHDYH\ngbi+cszEZveERJK8i34ZqqrgFh8+nBiRExVFjmtJMqxjUpGUIURlw4U0/EC3fM5O\nuQjX4z992F0L0vs0gWgFd3o3mapoh+NPTeyJ+b6IZ8JUm0Mw/i0hLGyBoK5wCY1Z\npqBuMmnK+st8/QG2UxS1TifiEy/zSFDdmvJ14B2CdckAY3PEtzCPYnm7Q1NpM9vs\nDdjrkOtiaREJaqvfuEw9g9IlECb5wkmXUNpWMfgd31ecD766SK/YP8dqR5zVW+Vh\ndp8FtRZGiGyWbOglyARKh4GC37LoJZJk8uTvp1FmEnUZK6nSSDmDzNAq8M8849zH\nMGXpoRd2A04nmuL5JAA2WpkW7WQqbvpmunXHRrOKt9cFmE6zuslm8TiRTigjD69Y\nOuSl48x7o/3LLJw4dbgPlmYi5Ezf2ZncllFSbIPxXSP5t95cqOrxTA7/Ue8/VJnu\nD3GJym8lXPFn8zHMjVH9ujxpm69+0XqSsBnkiQtcv0qeaPEBMobbABBcvMumvUAb\ndU1oMVr4Md+5Oapn7M03w+ZNIqofZtQFbmEiWajAYTixhYQHSqLXdp1YXgVIcfBa\nIyo+tB2Zu7WF6b4mp0tD0tUCARE=\n-----END PUBLIC KEY-----".to_string();
        assert!(ccrypto::rsa_is_valid_pub_key(&aaa_pgp_public_key));
    }

    pub fn test_cpp_generated_keys_2()
    {
        let bbb_pgp_private_key: String = "-----BEGIN PRIVATE KEY-----\n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\nbxC8SYNfb397H1VNQUzlPIW6oA==\n-----END PRIVATE KEY-----".to_string();
        assert!(ccrypto::rsa_is_valid_prv_key(&bbb_pgp_private_key));
        let bbb_pgp_public_key: String = "-----BEGIN PUBLIC KEY-----\nMIICIDANBgkqhkiG9w0BAQEFAAOCAg0AMIICCAKCAgEAzruXdhodXPeHsaMgNSw6\nUN4sRVGXeQRri8PJ4EFGKuTcVF6/2ZWy+I/VHHoq1Y/15vm/RFR/Qfit9TIgi+p2\neCw9djM7jlrKvTZfs+vAKx+qJ56x1gu0Cja+RkdbJdliqVKALZfY0+jWaco+zH+e\nuE8qPjUygUoISzmsNdcZrNNi2eLJ0yNMVa4x5y0Y0ywuhJvbxwMfvdRxMuy0YZwc\nC8i5lkCPCPKjr9OzR+jC7weUM/O/HmfqJOFhXiCKXvjXaBZdEt5WqU86lMRElk9p\nXhyglOlrPe/39n1Bhs0wu4MDbhcrXPj8pfEmxplhQfaR3/ezxwVxY0MEffFGlMgX\n60RwoEgR7BX1BZv2qwp7KBf7gYYJeOVYCjxdbKYRBLdOMfqQ3rXbtpljioa1975X\nZu3bAH0ByhKKPZKqVApL3sA1Pm7vWjS8XEUOcLgLKxYeqjrZeTes0VW30y/tHnkU\nOHT0zV8SYpwuS3mgQmbivrvCLhdAYE1e8HGztb2bhBPHsQgBnqRnfNDHyrErMQHB\nfwXT79uN/oyj4CReoMqfgm0n/GH2ZijKb94LR2MMHeTrTxOh3rI94t0HCqVxB7Ks\nh2H5H9ynZ6YUtM0FQQIjzUgU/bTQuAQHqOBTiiSCKFGrPf0iUcY0tt/pX8HQMP82\nzLZXKLKXNWgRfTAXiZTOzVUCARE=\n-----END PUBLIC KEY-----".to_string();
        assert!(ccrypto::rsa_is_valid_pub_key(&bbb_pgp_public_key));
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