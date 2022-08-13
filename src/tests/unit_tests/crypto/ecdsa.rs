#[cfg(test)]
pub mod tests_crypto_ecdsa {
    use substring::Substring;
    use crate::lib::constants as cconsts;
    // use crate::lib::utils::cutils as cutils;
    use crate::lib::ccrypto;

    #[test]
    pub fn test_autogen_ecdsa_key_pair() {
        let (status, private_key, public_key) = ccrypto::ecdsa_generate_key_pair();
        assert!(status);
        assert!(ccrypto::ecdsa_validate_private_key(&private_key));
        assert!(ccrypto::ecdsa_validate_public_key(&public_key));

        {
            let msg = "Hello world".to_string();
            let (status, signature_hex, _signature) = ccrypto::ecdsa_sign_message(&private_key, &msg);
            assert!(status);
            assert!(ccrypto::ecdsa_verify_signature(&public_key, &msg, &signature_hex));
        }
    }

    #[test]
    pub fn test_prive_to_pub()
    {
        struct ECDSAKeyPair {
            pub_key: String,
            prv_key: String,
        }
        let cpp_generated_ecdsa_pairs: Vec<ECDSAKeyPair> = vec![
            ECDSAKeyPair {
                pub_key: "02615f10a0a6661ef79463a0094449852d3ef2931a2a8890b5286404a24e5eed99".to_string(),
                prv_key: "f2ce8cf06e1eaa7be6683a9acc57b74203f96df2f8ae0df37e73323f8a2b1594".to_string(),
            },
            ECDSAKeyPair {
                pub_key: "022968b10e02e2af51a5965b9735ac2c75c51c71207f85bec0bd49fa61902f8619".to_string(),
                prv_key: "2aecb3ced6bbccd018bbae143f1e0edd6af6eb855a0ab969612bd6f7854b3f9e".to_string(),
            },
            ECDSAKeyPair {
                pub_key: "02ca33b31985b7780cd73a5c56370c67a29ae90c761409452f7f40db46ac68f026".to_string(),
                prv_key: "31885ade263a13ff87bda340bc96cd9845add45f4fff54cff7ca3e625e331a05".to_string(),
            },
            ECDSAKeyPair {
                pub_key: "03a797608e14ee87a93c0bf7d7d121593c5985030e9053e4d062bf081d59da956b".to_string(),
                prv_key: "edae1b71c0c4f745447aa315524b3229c3aa50cc4e25579bfae98f501ebb0f16".to_string(),
            },
            ECDSAKeyPair {
                pub_key: "02098b33886fb84c18b6fcd698588114817b5ba0087765c843801226751ff38856".to_string(),
                prv_key: "30475956bba8843899615b5d4b744b086070ec72c732a514656d1339c583157c".to_string(),
            },
        ];
        for a_key_pair in cpp_generated_ecdsa_pairs {
            let pub_key = ccrypto::ecdsa_generate_pub_by_priv(&a_key_pair.prv_key);
            assert_eq!(pub_key, a_key_pair.pub_key);
        }

        let js_generated_ecdsa_pairs: Vec<ECDSAKeyPair> = vec![
            ECDSAKeyPair {
                pub_key: "02615f10a0a6661ef79463a0094449852d3ef2931a2a8890b5286404a24e5eed99".to_string(),
                prv_key: "f2ce8cf06e1eaa7be6683a9acc57b74203f96df2f8ae0df37e73323f8a2b1594".to_string(),
            },
            ECDSAKeyPair {
                pub_key: "022968b10e02e2af51a5965b9735ac2c75c51c71207f85bec0bd49fa61902f8619".to_string(),
                prv_key: "2aecb3ced6bbccd018bbae143f1e0edd6af6eb855a0ab969612bd6f7854b3f9e".to_string(),
            },
            ECDSAKeyPair {
                pub_key: "02ca33b31985b7780cd73a5c56370c67a29ae90c761409452f7f40db46ac68f026".to_string(),
                prv_key: "31885ade263a13ff87bda340bc96cd9845add45f4fff54cff7ca3e625e331a05".to_string(),
            },
            ECDSAKeyPair {
                pub_key: "03a797608e14ee87a93c0bf7d7d121593c5985030e9053e4d062bf081d59da956b".to_string(),
                prv_key: "edae1b71c0c4f745447aa315524b3229c3aa50cc4e25579bfae98f501ebb0f16".to_string(),
            },
            ECDSAKeyPair {
                pub_key: "02098b33886fb84c18b6fcd698588114817b5ba0087765c843801226751ff38856".to_string(),
                prv_key: "30475956bba8843899615b5d4b744b086070ec72c732a514656d1339c583157c".to_string(),
            },
        ];
        for a_key_pair in js_generated_ecdsa_pairs {
            let pub_key = ccrypto::ecdsa_generate_pub_by_priv(&a_key_pair.prv_key);
            assert_eq!(pub_key, a_key_pair.pub_key);
        }
    }

    #[test]
    pub fn test_sign_verification()
    {
        {
            let mut _msg = ccrypto::keccak256(&"5df1b8f3e190197cd317df102be85dda74acaddb741ed5f2cbb46d73dd79e01b".to_string()).substring(0, constants::SIGN_MSG_LENGTH as usize).to_string();
            // println!("---------------------------------::: {}" ,msg);
            // let msg = "3f060510ab5dcdf8b0f8c2a427fc96a3".to_string();
            let _signature_hex = "7538349870784dc8da9b4d624905453db99cb08072dd0c264dc82696d77f01ce134ed738a09a98d53a4d34b40f01f099309d00662a99cb23f11211a5bd20bde0".to_string();
            let _pub_key = "022968b10e02e2af51a5965b9735ac2c75c51c71207f85bec0bd49fa61902f8619".to_string();
            // assert!(ccrypto::ecdsa_verify_signature(&pub_key, &msg, &signature_hex));
        }

        {
            let msg = "Hello world.....................".to_string();
            let cpp_priv_key = "e94356090b28fb09da4c03acb1f46181b094826613f5c7445cc3a5ecb5f0bf02".to_string();
            let cpp_pub_key = "02447246566f387d0c13912f87f5c309e3690890024131c0435d72a1443b8f2efd".to_string();
            assert_eq!(ccrypto::ecdsa_generate_pub_by_priv(&cpp_priv_key), cpp_pub_key);
            let (sign_status, signature_hex, _signature) = ccrypto::ecdsa_sign_message(&cpp_priv_key, &msg);
            assert!(sign_status);
            assert!(ccrypto::ecdsa_verify_signature(&cpp_pub_key, &msg, &signature_hex));
        }
    }

}