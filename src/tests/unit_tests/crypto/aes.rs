#[cfg(test)]
pub mod tests_crypto_aes {
    use crate::ccrypto::{aes_decrypt, aes_encrypt};

    #[test]
    pub fn test_autogen_aes_1() {
        let msg = "Hello World".to_string();
        let key = "encryption key..".to_string();
        let (status, encrypted_msg) = aes_encrypt(
            msg.clone(),
            key.clone(),
        );
        assert!(status);

        let (status, dec_msg) = aes_decrypt(
            encrypted_msg,
            key.clone(),
            "".to_string()
        );
        assert!(status);
        assert_eq!(dec_msg, msg);
    }

    #[test]
    pub fn test_autogen_aes_2() {
        let msg = "Hello World.....".to_string();
        let key = "encryption key..".to_string();
        let (status, encrypted_msg) = aes_encrypt(
            msg.clone(),
            key.clone(),
        );
        assert!(status);

        let (status, dec_msg) = aes_decrypt(
            encrypted_msg,
            key.clone(),
            "".to_string()
        );
        assert!(status);
        assert_eq!(dec_msg, msg);
    }

    #[test]
    pub fn test_autogen_aes_3() {
        let msg = "Hello World.... .".to_string();
        let key = "encryption key..".to_string();
        let (status, encrypted_msg) = aes_encrypt(
            msg.clone(),
            key.clone(),
        );
        assert!(status);

        let (status, dec_msg) = aes_decrypt(
            encrypted_msg,
            key.clone(),
            "".to_string()
        );
        assert!(status);
        assert_eq!(dec_msg, msg);
    }

    #[test]
    pub fn test_autogen_aes_4() {
        let msg = "Hello World.... 123456789 123  6".to_string();
        let key = "encryption key..".to_string();
        let (status, encrypted_msg) = aes_encrypt(
            msg.clone(),
            key.clone(),
        );
        assert!(status);

        let (status, dec_msg) = aes_decrypt(
            encrypted_msg,
            key.clone(),
            "".to_string()
        );
        assert!(status);
        assert_eq!(dec_msg, msg);
    }

}