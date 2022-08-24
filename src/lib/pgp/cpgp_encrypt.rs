use serde_json::json;
use crate::{ccrypto, constants, cutils, dlog};
use crate::lib::custom_types::JSonObject;
use crate::cutils::remove_quotes;
use crate::lib::utils::compressor::compress;

//old_name_was encryptPGP
pub fn pgp_encrypt(
    message: &String,
    sender_priv_key: &String,
    receiver_pub_key: &String,
    secret_key_: &String,
    _initialization_vector: &str,
    should_compress: bool,
    should_sign: bool) -> (bool, String)
{

    // base64 encoding
    let b64_encoded_msg = ccrypto::b64_encode(message);

    let mut signature: String = "".to_string();
    if should_sign {
        let (status, the_signature) = ccrypto::rsa_sign(
            &sender_priv_key,
            &ccrypto::keccak256(&*b64_encoded_msg),
        );
        if !status {
            dlog(
                &format!("Failed in RSA signing the msg: {}", message),
                constants::Modules::App,
                constants::SecLevel::Fatal);
            return (false, "Failed in RSA signing the in CPGP!".to_string());
        }
        signature = the_signature;
    }

    let mut is_signed = "false".to_string();
    if should_sign {
        is_signed = "true".to_string();
    }

    let mut pgp_enc_jobj: JSonObject = json!({
        "sigVersion": "0.0.2",
        "isSigned": is_signed, //  // this version denotes to used hashing and signature
        "message": b64_encoded_msg,
        "signature": signature });

    if should_compress {
        // TODO: implementing compress function
        pgp_enc_jobj["message"] = compress(&remove_quotes(&pgp_enc_jobj["message"])).into();
        pgp_enc_jobj["isCompressed"] = "true".into();
        pgp_enc_jobj["zipVersion"] = "0.0.0".into();
        pgp_enc_jobj["zipAlgorithm"] = "zip...".into();
    } else {
        pgp_enc_jobj["isCompressed"] = "false".into();
    }
    dlog(
        &format!("compressed 1: {}", pgp_enc_jobj),
        constants::Modules::App,
        constants::SecLevel::TmpDebug);

    let mut secret_key: String = secret_key_.clone();
    if receiver_pub_key.to_string() == "".to_string()
    {
        // there is no pub key, so ther is no sence to encryption
        pgp_enc_jobj["iPGPVersion"] = constants::CURRENT_PGP_VERSION.to_string().into();
        pgp_enc_jobj["secretKey"] = constants::message_tags::NO_ENCRYPTION.into();
        pgp_enc_jobj["isAuthenticated"] = "false".into();
    } else {
        if secret_key == "" {
            secret_key = ccrypto::get_random_number_as_string(16);
        }
        // conventional symmetric encryption
        let (aes_status, aes_encoded) = ccrypto::aes_encrypt(
            remove_quotes(&pgp_enc_jobj["message"]),
            secret_key.clone());


        if !aes_status {
            return (false, "failed in AES encrypt".to_string());
        }

        let (status, pgp_encrypted_secret_key) = ccrypto::rsa_encrypt_with_pub_key(
            receiver_pub_key,
            &secret_key);
        if !status {
            dlog(
                &format!("Failed in secret key (pub enc) in CPGP: {}", receiver_pub_key),
                constants::Modules::App,
                constants::SecLevel::Error);
            return (false, "Failed in secret key (pub enc) in CPGP".to_string());
        }

        pgp_enc_jobj["aesVersion"] = constants::CURRENT_AES_VERSION.into();
        pgp_enc_jobj["message"] = aes_encoded.into();
        pgp_enc_jobj["iPGPVersion"] = constants::CURRENT_PGP_VERSION.to_string().into();
        pgp_enc_jobj["secretKey"] = pgp_encrypted_secret_key.into();
        pgp_enc_jobj["iv"] = "implement it ASAP".into();
        pgp_enc_jobj["isAuthenticated"] = "true".into();
    }

    let mut pgp_response: String = cutils::serialize_json(&pgp_enc_jobj);
    dlog(
        &format!("pgp_response: {}", pgp_enc_jobj),
        constants::Modules::App,
        constants::SecLevel::TmpDebug);

    // base64 encoding the final message
    return (true, ccrypto::b64_encode(&pgp_response));
}