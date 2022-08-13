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
pub fn encryptPGP(
    message: &str,
    sender_priv_key: &str,
    receiver_pub_key: &str,
    secret_key: &str,
    initialization_vector: &str,
    should_compress: bool,
    should_sign: bool) -> (bool, String)
{
    return (true, "".to_string());
    /*
      String pgp_versaion = "0.0.1";

      // base64 encoding
      String base64Encoded = ccrypto::base64Encode(message);

      String signature = "";
      if (should_sign) {
        signature = ccrypto::nativeSign(sender_priv_key, ccrypto::keccak256(base64Encoded));
      }
      JSonObject Jmsg_plus_signature {
        {"sigVersion", "0.0.2"},
        {"isSigned", (should_sign ? String::fromStdString("true") : String::fromStdString("false"))}, //  // this version denotes to used hashing and signature
        {"message", base64Encoded},
        {"signature", signature }};

      String msg_plus_signature = cutils::serializeJson(Jmsg_plus_signature);
      CLog::log("msg_plus_signature: " + msg_plus_signature, "app", "trace");

      String compressed;
      if (should_compress)
      {
        JSonObject Jcompressed {
          {"zipVersion", "0.0.0"},
          {"zipAlgorithm", "zip..."},
          {"isCompressed", "true"},
          {"message", Compressor::compress(msg_plus_signature)}}; // TODO: implementing compress function
        compressed = cutils::serializeJson(Jcompressed);
      }
      else
      {
        JSonObject Jcompressed {
          {"isCompressed", "false"},
        {"message", msg_plus_signature}};
        compressed = cutils::serializeJson(Jcompressed);
      }
      CLog::log("compressed1: " + compressed, "app", "trace");

      String pgpRes = "";

      if (receiver_pub_key == "")
      {
        // there is no pub key, so ther is no sence to encryption
        JSonObject JpgpRes {
          {"iPGPVersion", pgp_versaion},
          {"secretKey", CConsts::MESSAGE_TAGS::NO_ENCRYPTION},
          {"message", compressed}, // to support javascript nodes, maybe need to use \"compresses\"
          {"isAuthenticated", "false"}};

        pgpRes = cutils::serializeJson(JpgpRes);

      } else {
        if (secret_key == "")
          secret_key = ccrypto::getRandomNumber(32);
        if (initialization_vector == "")
          initialization_vector = ccrypto::getRandomNumber(16);

        secret_key = secret_key.midRef(0, 32).toString();
        initialization_vector = initialization_vector.midRef(0, 16).toString();

        // conventional symmetric encryption
        auto[aes_status, aesEncoded] = ccrypto::AESencrypt(
          compressed,
          secret_key,
          initialization_vector);

        if (!aes_status)
          return {false, "failed in AESencrypt"};
        String pgp_encrypted_secret_key, pgp_encrypted_iv;
        if (CConsts::CURRENT_AES_VERSION == "0.0.0")
        {
          pgp_encrypted_secret_key = CConsts::MESSAGE_TAGS::NO_ENCRYPTION;
          pgp_encrypted_iv = "";
        }else{
          pgp_encrypted_secret_key = ccrypto::encryptStringWithPublicKey(
            receiver_pub_key,
            secret_key);

          pgp_encrypted_iv = ccrypto::encryptStringWithPublicKey(
            receiver_pub_key,
            initialization_vector);
        }
        JSonObject JaesEncoded {
          {"aesVersion", CConsts::CURRENT_AES_VERSION},
          {"encrypted", aesEncoded}};

        JSonObject JpgpRes {
          {"iPGPVersion", pgp_versaion},
          {"secretKey", pgp_encrypted_secret_key},
          {"iv", pgp_encrypted_iv},
          {"message", JaesEncoded},
          {"isAuthenticated", "true"}};

        pgpRes = cutils::serializeJson(JpgpRes);
      }
      CLog::log("pgpRes: " + pgpRes, "app", "trace");

      // base64 encoding
      return {true, ccrypto::base64Encode(pgpRes)};
    */
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
  finalDec.m_is_authenticated = obj.value("isAuthenticated").toString() == "true";
  finalDec.m_pgp_versaion = obj.value("iPGPVersion").toString();
  finalDec.m_secret_key = obj.value("secretKey").toString();
  finalDec.m_initialization_vector = obj.value("iv").toString();

  JSonObject AESdecrypted;

  if (finalDec.m_secret_key == CConsts::MESSAGE_TAGS::NO_ENCRYPTION) {
    CLog::log("AESdecrypted = " + obj.value("message").toString(), "app", "trace");
    auto tmpAESdecrypted = obj.value("message");
    if (tmpAESdecrypted.isObject())
    {
      AESdecrypted = tmpAESdecrypted.toObject();
    } else{
      // backward compatibility with Javascrit nodes
      AESdecrypted = cutils::parseToJsonObj(tmpAESdecrypted.toString());
      AESdecrypted["message"] = cutils::parseToJsonObj(AESdecrypted.value("message").toString());
    }
  }
  else
  {
    JSonObject aesEncoded = obj.value("message").toObject();
    finalDec.m_aes_version = aesEncoded.value("aesVersion").toString(); //, "Unknown AES Version!"

    String AESencryptedMsg = aesEncoded.value("encrypted").toString();

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

  finalDec.m_is_compressed = AESdecrypted.value("isCompressed").toString() == "true";
  finalDec.m_zip_version = AESdecrypted.keys().contains("zipVersion") ? AESdecrypted.value("isCompressed").toString() : "Unknown zip version!";
  finalDec.m_zip_algorithm = AESdecrypted.keys().contains("algorithm") ? AESdecrypted.value("algorithm").toString() : "Unknown zip algorithm!";

  JSonObject msg_plus_signature;
  if (finalDec.m_secret_key == CConsts::MESSAGE_TAGS::NO_ENCRYPTION)
  {
    msg_plus_signature = AESdecrypted.value("message").toObject();
  } else {
    msg_plus_signature = cutils::parseToJsonObj(AESdecrypted.value("message").toString());
  }

  // decompress it if it is compressed
  if (finalDec.m_is_compressed)
  {
    // decompress message
    message = Compressor::decompress(msg_plus_signature.value("message").toString());
  }

  // control signature if is signed
  finalDec.m_is_signed = msg_plus_signature.value("isSigned").toString() == "true";
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
    finalDec.m_signature_version = msg_plus_signature.value("sigVersion").toString();
    String signature = msg_plus_signature.value("signature").toString();
    String hash = ccrypto::keccak256(msg_plus_signature.value("message").toString());
    finalDec.m_is_verified = ccrypto::nativeVerifySignature(sender_pub_key, hash, signature);
  }
  finalDec.m_decryption_status = true;
  finalDec.m_message = ccrypto::base64Decode(msg_plus_signature.value("message").toString());
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
  secret_key = secret_key.midRef(0, 32).toString();

  String initialization_vector = ccrypto::getRandomNumber();
  initialization_vector = initialization_vector.midRef(0, 16).toString();

  return {secret_key, initialization_vector};
}


*/