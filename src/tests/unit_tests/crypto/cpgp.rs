#[cfg(test)]
pub mod tests_crypto_cpgp {
    use crate::lib::ccrypto;
    use crate::lib::pgp::cpgp::{CPGPMessage};
    use crate::lib::pgp::cpgp_decrypt::pgp_decrypt;
    use crate::lib::pgp::cpgp_encrypt::pgp_encrypt;

    #[test]
    #[ignore]
    pub fn test_autogen_pgp_1()
    {
        let (status, sender_pem_prv_key, sender_pem_pub_key) = ccrypto::rsa_generate_key_pair(0);
        assert!(status);
        let (status, receiver_pem_prv_key, receiver_pem_pub_key) = ccrypto::rsa_generate_key_pair(0);
        assert!(status);

        let msg = "Hello World".to_string();
        let should_sign = true;
        let (status, encrypted_msg) = pgp_encrypt(
            &msg,
            &sender_pem_prv_key,
            &receiver_pem_pub_key,
            &"".to_string(),
            "",
            true,
            should_sign);
        assert!(status);

        let final_decoded_info: CPGPMessage = pgp_decrypt(
            &encrypted_msg,
            &receiver_pem_prv_key,
            &sender_pem_pub_key);
        assert!(final_decoded_info.m_decryption_status);
        assert_eq!(final_decoded_info.m_message, msg);
        assert_eq!(final_decoded_info.m_is_signed, should_sign);
        assert_eq!(final_decoded_info.m_is_verified, should_sign);
        assert_eq!(final_decoded_info.m_is_authenticated, should_sign);
    }

    #[test]
    #[ignore]
    pub fn test_autogen_pgp_long()
    {
        let (status, sender_pem_prv_key, sender_pem_pub_key) = ccrypto::rsa_generate_key_pair(0);
        assert!(status);
        let (status, receiver_pem_prv_key, receiver_pem_pub_key) = ccrypto::rsa_generate_key_pair(0);
        assert!(status);

        let msg = "Too long msg Hello World".to_string();
        let should_sign = true;
        let (status, encrypted_msg) = pgp_encrypt(
            &msg,
            &sender_pem_prv_key,
            &receiver_pem_pub_key,
            &"".to_string(),
            "",
            true,
            should_sign);
        assert!(status);

        let final_decoded_info: CPGPMessage = pgp_decrypt(
            &encrypted_msg,
            &receiver_pem_prv_key,
            &sender_pem_pub_key);
        assert!(final_decoded_info.m_decryption_status);
        assert_eq!(final_decoded_info.m_message, msg);
        assert_eq!(final_decoded_info.m_is_signed, should_sign);
        assert_eq!(final_decoded_info.m_is_verified, should_sign);
        assert_eq!(final_decoded_info.m_is_authenticated, should_sign);
    }
}