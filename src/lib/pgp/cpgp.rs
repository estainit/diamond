use serde_json::json;
use crate::{ccrypto, constants, cutils, dlog};
use crate::lib::custom_types::JSonObject;
use crate::lib::utils::compressor::{compress, decompress};
use serde::{Serialize, Deserialize};
use crate::cutils::remove_quotes;

#[derive(Clone, Serialize, Deserialize)]
pub struct CPGPMessage
{
    pub m_decryption_status: bool,
    pub m_pgp_versaion: String,
    pub m_secret_key: String,
    pub m_initialization_vector: String,
    pub m_decrypted_secret_key: String,
    pub m_decrypted_initialization_vector: String,
    pub m_message: String,
    pub m_aes_version: String,
    pub m_signature_version: String,
    pub m_is_signed: bool,
    pub m_is_compressed: bool,
    pub m_is_verified: bool,
    pub m_is_authenticated: bool,
    pub m_zip_version: String,
    pub m_zip_algorithm: String,
}

impl CPGPMessage {
    pub fn new() -> CPGPMessage {
        CPGPMessage {
            m_decryption_status: false,
            m_pgp_versaion: constants::CURRENT_PGP_VERSION.to_string(),
            m_secret_key: "".to_string(),
            m_initialization_vector: "".to_string(),
            m_decrypted_secret_key: "".to_string(),
            m_decrypted_initialization_vector: "".to_string(),
            m_message: "".to_string(),
            m_aes_version: "".to_string(),
            m_signature_version: "".to_string(),
            m_is_signed: false,
            m_is_compressed: false,
            m_is_verified: false,
            m_is_authenticated: false,
            m_zip_version: "".to_string(),
            m_zip_algorithm: "".to_string(),
        }
    }
}

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
        pgp_enc_jobj["message"] = compress(&remove_quotes(&pgp_enc_jobj["message"].to_string())).into();
        pgp_enc_jobj["isCompressed"] = "true".into();
        pgp_enc_jobj["zipVersion"] = "0.0.0".into();
        pgp_enc_jobj["zipAlgorithm"] = "zip...".into();
    } else {
        pgp_enc_jobj["isCompressed"] = "false".into();
    }
    dlog(
        &format!("compressed 1: {}", pgp_enc_jobj),
        constants::Modules::App,
        constants::SecLevel::Trace);

    let mut secret_key: String = secret_key_.clone();
    if receiver_pub_key.to_string() == "".to_string()
    {
        // there is no pub key, so ther is no sence to encryption
        pgp_enc_jobj["iPGPVersion"] = constants::CURRENT_PGP_VERSION.to_string().into();
        pgp_enc_jobj["secretKey"] = constants::message_tags::NO_ENCRYPTION.into();
        pgp_enc_jobj["isAuthenticated"] = "false".into();
    } else {
        if secret_key == "" {
            secret_key = ccrypto::get_random_number(16);
        }
        // conventional symmetric encryption
        let (aes_status, aes_encoded) = ccrypto::aes_encrypt(
            remove_quotes(&pgp_enc_jobj["message"].to_string()),
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

    let mut pgp_response: String = cutils::serializeJson(&pgp_enc_jobj);
    dlog(
        &format!("pgp_response: {}", pgp_enc_jobj),
        constants::Modules::App,
        constants::SecLevel::Trace);

    // base64 encoding the final message
    return (true, ccrypto::b64_encode(&pgp_response));
}

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
        constants::SecLevel::Trace);

    let decode_j_obj: JSonObject = cutils::parseToJsonObj(&base64_decoded);
    final_decoded_msg.m_is_authenticated = remove_quotes(&decode_j_obj["isAuthenticated"].to_string()) == "true";
    final_decoded_msg.m_is_signed = remove_quotes(&decode_j_obj["isSigned"].to_string()) == "true";
    final_decoded_msg.m_signature_version = remove_quotes(&decode_j_obj["sigVersion"].to_string());
    final_decoded_msg.m_pgp_versaion = remove_quotes(&decode_j_obj["iPGPVersion"].to_string());
    final_decoded_msg.m_secret_key = remove_quotes(&decode_j_obj["secretKey"].to_string());
    final_decoded_msg.m_initialization_vector = remove_quotes(&decode_j_obj["iv"].to_string());

    if final_decoded_msg.m_secret_key == constants::message_tags::NO_ENCRYPTION
    {
        dlog(
            &format!("AES decrypted = {}", decode_j_obj["message"].to_string()),
            constants::Modules::App,
            constants::SecLevel::Trace);

        final_decoded_msg.m_message = remove_quotes(&decode_j_obj["message"].to_string());
    } else {
        final_decoded_msg.m_aes_version = remove_quotes(&decode_j_obj["aesVersion"].to_string()); //, "Unknown AES Version!"

        // decrypt secret key
        let (status, decrypted_secret_key) = ccrypto::rsa_decrypt_with_prv_key(priv_key, &final_decoded_msg.m_secret_key);
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

        // decrypt message body
        let (status_aes_dec, aes_dec) = ccrypto::aes_decrypt(
            remove_quotes(&decode_j_obj["message"].to_string()),
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

    final_decoded_msg.m_is_compressed = remove_quotes(&decode_j_obj["isCompressed"].to_string()) == "true";
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
            &remove_quotes(&decode_j_obj["signature"].to_string()));
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

//old_name_was stripPGPEnvelope
pub fn strip_pgp_envelope(content: &String) -> String
{
    let mut content: String = content.trim().to_string(); // remove extra spaces
    if content.contains(constants::message_tags::iPGPStartEnvelope)
    {
        content = content.split(constants::message_tags::iPGPStartEnvelope).collect::<Vec<&str>>()[1].to_string().clone();
    }
    if content.contains(constants::message_tags::iPGPEndEnvelope)
    {
        content = content.split(constants::message_tags::iPGPEndEnvelope).collect::<Vec<&str>>()[0].to_string().clone();
    }
    return content;
}

//old_name_was wrapPGPEnvelope
pub fn wrap_pgp_envelope(content:&String) ->String
{
    return constants::message_tags::iPGPStartEnvelope.to_owned() + content + constants::message_tags::iPGPEndEnvelope;
}
