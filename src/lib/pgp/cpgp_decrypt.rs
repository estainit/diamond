use crate::{ccrypto, constants, cutils, dlog};
use crate::cutils::remove_quotes;
use crate::lib::pgp::cpgp::{CPGPMessage, strip_pgp_envelope};
use crate::lib::utils::compressor::decompress;

//old_name_was decryptPGP
pub fn pgp_decrypt(
    message: &String,
    priv_key: &String,
    sender_pub_key: &String) -> CPGPMessage
{
    let mut final_decoded_msg: CPGPMessage = CPGPMessage::new();
    let mut message: String = strip_pgp_envelope(message);
    // decode base64
    let (status, base64_decoded) = ccrypto::b64_decode(&message);
    if !status
    {
        dlog(
            &format!("failed in base64 Decoded in pgp decrypt: {}", base64_decoded),
            constants::Modules::Sec,
            constants::SecLevel::Error);
        final_decoded_msg.m_decryption_status = false;
        return final_decoded_msg;
    }

    dlog(
        &format!("base64 Decoded: {}", base64_decoded),
        constants::Modules::App,
        constants::SecLevel::Info);

    let (status, decode_j_obj) = cutils::controlled_str_to_json(&base64_decoded);
    if !status
    {
        dlog(
            &format!("failed deserialize msg in cpgp! {}", base64_decoded),
            constants::Modules::App,
            constants::SecLevel::Error);
        final_decoded_msg.m_decryption_status = false;
        return final_decoded_msg;
    }

    dlog(
        &format!("deserialized message json object: {}", decode_j_obj),
        constants::Modules::App,
        constants::SecLevel::Info);

    final_decoded_msg.m_is_authenticated = remove_quotes(&decode_j_obj["isAuthenticated"]) == "true";
    final_decoded_msg.m_is_signed = remove_quotes(&decode_j_obj["isSigned"]) == "true";
    final_decoded_msg.m_signature_version = remove_quotes(&decode_j_obj["sigVersion"]);
    final_decoded_msg.m_pgp_versaion = remove_quotes(&decode_j_obj["iPGPVersion"]);
    final_decoded_msg.m_secret_key = remove_quotes(&decode_j_obj["secretKey"]);
    final_decoded_msg.m_initialization_vector = remove_quotes(&decode_j_obj["iv"]);

     eprintln!("final_decoded_msg.m_secret_key: {}", final_decoded_msg.m_secret_key);
    if final_decoded_msg.m_secret_key == constants::message_tags::NO_ENCRYPTION
    {
        dlog(
            &format!("AES decrypted = {}", decode_j_obj["message"].to_string()),
            constants::Modules::App,
            constants::SecLevel::Trace);

        final_decoded_msg.m_message = remove_quotes(&decode_j_obj["message"]);
    } else {
        final_decoded_msg.m_aes_version = remove_quotes(&decode_j_obj["aesVersion"]); //, "Unknown AES Version!"

        // decrypt secret key
        let (status, decrypted_secret_key) = ccrypto::rsa_decrypt_with_prv_key(
            priv_key,
            &final_decoded_msg.m_secret_key);
        if !status {
            dlog(
                &format!("Failed in CPGP, RSA decrypting the secret key."),
                constants::Modules::App,
                constants::SecLevel::Fatal);
            final_decoded_msg.m_message = "Failed in CPGP, RSA decrypting the secret key.".to_string();
            final_decoded_msg.m_decryption_status = false;
            final_decoded_msg.m_is_verified = false;
            return final_decoded_msg.clone();
        }
        final_decoded_msg.m_decrypted_secret_key = decrypted_secret_key.clone();
        // panic!("m_decrypted_secret_key: {}", final_decoded_msg.m_decrypted_secret_key.clone());

        // decrypt message body
        let (status_aes_dec, aes_dec) = ccrypto::aes_decrypt(
            remove_quotes(&decode_j_obj["message"]),
            final_decoded_msg.m_decrypted_secret_key.clone(),
            final_decoded_msg.m_aes_version.clone());
        if !status_aes_dec
        {
            dlog(
                &format!("Failed in CPGP, AES decrypting the message."),
                constants::Modules::App,
                constants::SecLevel::Fatal);
            final_decoded_msg.m_message = "Failed in CPGP, AES decrypting the message.".to_string();
            final_decoded_msg.m_decryption_status = false;
            final_decoded_msg.m_is_verified = false;
            return final_decoded_msg.clone();
        }
        // decrypted_aes = cutils::parseToJsonObj(&aes_dec);
        final_decoded_msg.m_message = aes_dec;
    }

    final_decoded_msg.m_is_compressed = remove_quotes(&decode_j_obj["isCompressed"]) == "true";
    // finalDec.m_zip_version = AESdecrypted.keys().contains("zipVersion") ? AESdecrypted["isCompressed"].to_string() : "Unknown zip version!";
    // finalDec.m_zip_algorithm = AESdecrypted.keys().contains("algorithm") ? AESdecrypted["algorithm"].to_string() : "Unknown zip algorithm!";

    if final_decoded_msg.m_is_compressed
    {
        final_decoded_msg.m_message = decompress(&final_decoded_msg.m_message);
    }

    final_decoded_msg.m_is_verified = false;
    if final_decoded_msg.m_is_signed
    {
        if sender_pub_key == ""
        {
            final_decoded_msg.m_decryption_status = false;
            final_decoded_msg.m_is_verified = false;
            final_decoded_msg.m_message = "missed sender_pub_key".to_string();
            return final_decoded_msg.clone();
        }
        let hash: String = ccrypto::keccak256(&final_decoded_msg.m_message);
        final_decoded_msg.m_is_verified = ccrypto::rsa_verify_signature(
            sender_pub_key,
            &ccrypto::keccak256(&final_decoded_msg.m_message),
            &remove_quotes(&decode_j_obj["signature"]));
    }

    let (status, base64_decoded) = ccrypto::b64_decode(&final_decoded_msg.m_message);
    if !status
    {
        dlog(
            &format!("failed in base64 Decoded in pgp final decrypt: {}", base64_decoded),
            constants::Modules::Sec,
            constants::SecLevel::Error);
        final_decoded_msg.m_decryption_status = false;
        return final_decoded_msg;
    }
    final_decoded_msg.m_message = base64_decoded;

    final_decoded_msg.m_decryption_status = true;

    return final_decoded_msg.clone();
}