/*

#ifndef CPGP_H
#define CPGP_H




class CPGPMessage
{
public:
  bool m_decryption_status = false;
  String m_pgp_versaion = "0.0.1";
  String m_secret_key = "";
  String m_initialization_vector = "";
  String m_decrypted_secret_key = "";
  String m_decrypted_initialization_vector = "";
  String m_message = "";
  String m_aes_version = "";
  String m_signature_version = "";
  bool m_is_signed = false;
  bool m_is_compressed = false;
  bool m_is_verified = false;
  bool m_is_authenticated = false;
  String m_zip_version = "";
  String m_zip_algorithm = "";

};

class CPGP
{

  static CPGPMessage decryptPGP(
    const String& message_,
    const String& priv_key,
    const String& sender_pub_key);

  static String wrapPGPEnvelope(const String& content);

  static String stripPGPEnvelope(const String& content);

  static std::tuple<String, String> generateSecretKeyIV();

};

#endif // CPGP_H

#include "stable.h"
#include "lib/ccrypto.h"

#include "cpgp.h"

CPGP::CPGP()
{

}
*/
use serde_json::json;
use crate::{ccrypto, constants, cutils, dlog};
use crate::lib::custom_types::JSonObject;
use crate::lib::utils::compressor::compress;

//old_name_was encryptPGP
pub fn encrypt_pgp(
    message: &String,
    sender_priv_key: &String,
    receiver_pub_key: &String,
    secret_key_: &String,
    initialization_vector: &str,
    should_compress: bool,
    should_sign: bool) -> (bool, String)
{
    let pgp_versaion = "0.0.1";

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

    let mut j_msg_plus_signature: JSonObject = json!({
        "sigVersion": "0.0.2",
        "isSigned": is_signed, //  // this version denotes to used hashing and signature
        "message": b64_encoded_msg,
        "signature": signature });

    let msg_plus_signature: String = cutils::serializeJson(&j_msg_plus_signature);
    dlog(
        &format!("msg_plus_signature: {}", msg_plus_signature),
        constants::Modules::App,
        constants::SecLevel::Trace);

    let mut compressed: String = "".to_string();
    if should_compress {
        let json_compressed: JSonObject = json!({
        "zipVersion": "0.0.0",
        "zipAlgorithm": "zip...",
        "isCompressed": "true",
        "message": compress(&msg_plus_signature)}); // TODO: implementing compress function
        compressed = cutils::serializeJson(&json_compressed);
    } else {
        let json_compressed: JSonObject = json!({
            "isCompressed": "false",
            "message": msg_plus_signature});
        compressed = cutils::serializeJson(&json_compressed);
    }
    dlog(
        &format!("compressed 1: {}", compressed),
        constants::Modules::App,
        constants::SecLevel::Trace);

    let mut pgp_response: String = "".to_string();

    let mut secret_key: String = secret_key_.clone();
    if receiver_pub_key == ""
    {
        // there is no pub key, so ther is no sence to encryption
        let j_pgp_res: JSonObject = json!({
          "iPGPVersion": pgp_versaion,
          "secretKey": constants::message_tags::NO_ENCRYPTION,
          "message": compressed,
          "isAuthenticated": "false"});

        pgp_response = cutils::serializeJson(&j_pgp_res);
    } else {
        /*
        if secret_key == "" {
            secret_key = ccrypto::get_random_number(32);
        }
        if initialization_vector == "" {
            initialization_vector = ccrypto::getRandomNumber(16);
        }

        secret_key = secret_key.midRef(0, 32).to_string();
        initialization_vector = initialization_vector.midRef(0, 16).to_string();

        // conventional symmetric encryption
        auto
        [aes_status, aesEncoded] = ccrypto::AESencrypt(
            compressed,
            secret_key,
            initialization_vector);

        if !aes_status {
            return (false, "failed in AESencrypt".to_string());
        }

        let pgp_encrypted_secret_key = ccrypto::encryptStringWithPublicKey(
            receiver_pub_key,
            secret_key);

        let pgp_encrypted_iv: String = ccrypto::encryptStringWithPublicKey(
            receiver_pub_key,
            initialization_vector);

        let JaesEncoded: JSonObject = json!({
            "aesVersion":constants::CURRENT_AES_VERSION,
            "encrypted": aesEncoded});


         */

        let pgp_encrypted_secret_key = "DUMMY VALUE, remove ASAP".to_string();
        let pgp_encrypted_iv = "DUMMY VALUE, remove ASAP".to_string();
        let JaesEncoded = "DUMMY VALUE, remove ASAP".to_string();

        let j_pgp_res: JSonObject = json!({
        "iPGPVersion": pgp_versaion,
        "secretKey": pgp_encrypted_secret_key ,
        "iv": pgp_encrypted_iv ,
        "message": JaesEncoded ,
        "isAuthenticated": "true"});

        pgp_response = cutils::serializeJson(&j_pgp_res);
    }
    dlog(
        &format!("pgp_response: {}", pgp_response),
        constants::Modules::App,
        constants::SecLevel::Trace);

    // base64 encoding the final message
    return (true, ccrypto::b64_encode(&pgp_response));
}

/*

CPGPMessage CPGP::decryptPGP(
  const String& message_,
  const String& priv_key,
  const String& sender_pub_key)
{
  CPGPMessage finalDec;
  String message = stripPGPEnvelope(message_);


  // decode base64
  String base64Decoded = ccrypto::base64Decode(message);
  CLog::log("base64Decoded" + base64Decoded);
  JSonObject obj = cutils::parseToJsonObj(base64Decoded);
  finalDec.m_is_authenticated = obj.value("isAuthenticated").to_string() == "true";
  finalDec.m_pgp_versaion = obj.value("iPGPVersion").to_string();
  finalDec.m_secret_key = obj.value("secretKey").to_string();
  finalDec.m_initialization_vector = obj.value("iv").to_string();

  JSonObject AESdecrypted;

  if (finalDec.m_secret_key == CConsts::MESSAGE_TAGS::NO_ENCRYPTION) {
    CLog::log("AESdecrypted = " + obj.value("message").to_string(), "app", "trace");
    auto tmpAESdecrypted = obj.value("message");
    if (tmpAESdecrypted.isObject())
    {
      AESdecrypted = tmpAESdecrypted.toObject();
    } else{
      // backward compatibility with Javascrit nodes
      AESdecrypted = cutils::parseToJsonObj(tmpAESdecrypted.to_string());
      AESdecrypted["message"] = cutils::parseToJsonObj(AESdecrypted.value("message").to_string());
    }
  }
  else
  {
    JSonObject aesEncoded = obj.value("message").toObject();
    finalDec.m_aes_version = aesEncoded.value("aesVersion").to_string(); //, "Unknown AES Version!"

    String AESencryptedMsg = aesEncoded.value("encrypted").to_string();

    // decrypt secret key
    String  decryptedSecretKey;
    try {
      finalDec.m_decrypted_secret_key = ccrypto::decryptStringWithPrivateKey(priv_key, finalDec.m_secret_key);
      finalDec.m_decrypted_initialization_vector = ccrypto::decryptStringWithPrivateKey(priv_key, finalDec.m_initialization_vector);
    } catch (std::exception) {
      finalDec.m_decryption_status = false;
      finalDec.m_is_verified = false;
      finalDec.m_message = "wrong priv_key or encrypted_secret_key";
      return finalDec;
    }

    // decrypt message body
    try {
        auto [status_aes_dec, aes_dec] = ccrypto::AESdecrypt(
          AESencryptedMsg,
          finalDec.m_decrypted_secret_key,
          finalDec.m_decrypted_initialization_vector,
          finalDec.m_aes_version);
        Q_UNUSED(status_aes_dec);
        AESdecrypted = cutils::parseToJsonObj(aes_dec);

    } catch (std::exception) {
      finalDec.m_decryption_status = false;
      finalDec.m_is_verified = false;
      finalDec.m_message = "AESdecrypt failed";
      return finalDec;
    }
  }

  finalDec.m_is_compressed = AESdecrypted.value("isCompressed").to_string() == "true";
  finalDec.m_zip_version = AESdecrypted.keys().contains("zipVersion") ? AESdecrypted.value("isCompressed").to_string() : "Unknown zip version!";
  finalDec.m_zip_algorithm = AESdecrypted.keys().contains("algorithm") ? AESdecrypted.value("algorithm").to_string() : "Unknown zip algorithm!";

  JSonObject msg_plus_signature;
  if (finalDec.m_secret_key == CConsts::MESSAGE_TAGS::NO_ENCRYPTION)
  {
    msg_plus_signature = AESdecrypted.value("message").toObject();
  } else {
    msg_plus_signature = cutils::parseToJsonObj(AESdecrypted.value("message").to_string());
  }

  // decompress it if it is compressed
  if (finalDec.m_is_compressed)
  {
    // decompress message
    message = Compressor::decompress(msg_plus_signature.value("message").to_string());
  }

  // control signature if is signed
  finalDec.m_is_signed = msg_plus_signature.value("isSigned").to_string() == "true";
  finalDec.m_is_verified = false;
  if (finalDec.m_is_signed)
  {
    if (sender_pub_key == "")
    {
      finalDec.m_decryption_status = false;
      finalDec.m_is_verified = false;
      finalDec.m_message = "missed sender_pub_key";
      return finalDec;
    }
    finalDec.m_signature_version = msg_plus_signature.value("sigVersion").to_string();
    String signature = msg_plus_signature.value("signature").to_string();
    String hash = ccrypto::keccak256(msg_plus_signature.value("message").to_string());
    finalDec.m_is_verified = ccrypto::nativeVerifySignature(sender_pub_key, hash, signature);
  }
  finalDec.m_decryption_status = true;
  finalDec.m_message = ccrypto::base64Decode(msg_plus_signature.value("message").to_string());
  return finalDec;
}


String CPGP::wrapPGPEnvelope(const String& content)
{
    return CConsts::MESSAGE_TAGS::iPGPStartEnvelope + content + CConsts::MESSAGE_TAGS::iPGPEndEnvelope;
}

String CPGP::stripPGPEnvelope(const String& content_)
{
  String content = content_.simplified(); // remove extra spaces
  if (content.contains(CConsts::MESSAGE_TAGS::iPGPStartEnvelope))
  {
    content = content.split(CConsts::MESSAGE_TAGS::iPGPStartEnvelope)[1];
  }
  if (content.contains(CConsts::MESSAGE_TAGS::iPGPEndEnvelope))
  {
    content = content.split(CConsts::MESSAGE_TAGS::iPGPEndEnvelope)[0];
  }
  return content;
}

std::tuple<String, String> CPGP::generateSecretKeyIV()
{
  String secret_key = ccrypto::getRandomNumber();
  secret_key = secret_key.midRef(0, 32).to_string();

  String initialization_vector = ccrypto::getRandomNumber();
  initialization_vector = initialization_vector.midRef(0, 16).to_string();

  return {secret_key, initialization_vector};
}


*/