use rand_core::OsRng;
// requires 'getrandom' feature
use secp256k1::{Secp256k1, Message, SecretKey, PublicKey,
                ecdsa::{Signature}};
use secp256k1::hashes::sha256;

use std::str::FromStr;

use crate::lib::constants as CConsts;
use crate::lib::bech_32;

//old name was isValidBech32
pub fn is_valid_bech32(inp_str: &str) -> bool
{
    return bech_32::comen_is_valid(inp_str);
}

//old_name_was bech32Encode
pub fn bech32_encode(str: &str) -> String
{
    bech_32::comen_encode(str, CConsts::SOCIETY_NAME)
}


/*


#include "ccrypto.h"

InitCCrypto InitCCrypto::s_instance;
void InitCCrypto::Iinit()
{

  //   ... Do some crypto stuff here ...

  //   Clean up

  //   Removes all digests and ciphers

  CLog::log("InitCCrypto::IinitLog");
  return;
}

const std::string CCrypto::PUBLIC_BEGIN = "-----BEGIN PUBLIC KEY-----";
const std::string CCrypto::PUBLIC_END = "-----END PUBLIC KEY-----";
const std::string CCrypto::PRIVATE_BEGIN = "-----BEGIN PRIVATE KEY-----";
const std::string CCrypto::PRIVATE_END = "-----END PRIVATE KEY-----";
const std::string CCrypto::RSA_PRIVATE_BEGIN = "-----BEGIN RSA PRIVATE KEY-----";
const std::string CCrypto::RSA_PRIVATE_END = "-----END RSA PRIVATE KEY-----";

QString CCrypto::convertTitleToHash(const QString &title)
{
  return keccak256(title);
}

using std::cout, std::endl;

//  -  -  -  general

std::tuple<bool, QString> CCrypto::AESencrypt(
    const QString &clear_text,
    const QString &secret_key,
    const QString &initialization_vector,
    const QString &aes_version)
{
  if (aes_version == "0.0.0")
  {
    // no encryption
    //    QString out = "{";
    //    out += "\"aesVersion\":\"" + aes_version + "\",";
    //    out += "\"encrypted\":\"" + base64Encode(clear_text) + "\"}";
    return {true, base64Encode(clear_text)};
  }
  else if (aes_version == "0.1.0")
  {
    // compatible version with legacy javascript clients. FIXME: implement it ASAP
    CLog::log("not suppoerted AES version", "app", "error");
    return {false, ""};
  }
  else if (aes_version == "0.2.0")
  {
    return AESencrypt020(clear_text, secret_key, initialization_vector, aes_version);
  }

  return {false, ""};
}

std::tuple<bool, QString> CCrypto::AESencrypt020(
    const QString &clear_text,
    const QString &secret_key,
    const QString &initialization_vector,
    const QString &aes_version)
{
  Q_UNUSED(aes_version);
  using namespace CryptoPP;
  AutoSeededRandomPool prng;

  std::string cipher, encoded, recovered;

  try
  {
    SecByteBlock aes_key(NULL, 32);
    SecByteBlock ivF(NULL, 16);

    // stub for how you really get it, e.g. reading it from a file, off of a
    //   network socket encrypted with an asymmetric cipher, or whatever
    //  read_key(aes_key, aes_key.size());
    std::string secret_key_ = secret_key.toStdString();
    for (uint i = 0; i < secret_key_.length(); i++)
    {
      aes_key[i] = secret_key_[i];
    }

    // stub for how you really get it, e.g. filling it with random bytes or
    //   reading it from the other side of the socket since both sides have
    //   to use the same IV as well as the same key
    //  read_initialization_vector(iv);
    std::string initialization_vector_ = initialization_vector.toStdString();
    for (uint i = 0; i < initialization_vector_.length(); i++)
    {
      ivF[i] = initialization_vector_[i];
    }

    // the final argument is specific to CFB mode, and specifies the refeeding
    //   size in bytes. This invocation corresponds to Java's
    //   Cipher.getInstance("AES/CFB8/NoPadding")
    //  auto enc1 = new CBC_Mode<AES>::Encryption(aes_key, sizeof(aes_key), ivF, 1);
    CBC_Mode<AES>::Encryption enc1(aes_key, sizeof(aes_key), ivF);
    StringSource ss2(clear_text.toStdString(), true,
                     new StreamTransformationFilter(enc1,
                                                    new Base64Encoder(
                                                        new StringSink(cipher), true, 64) // Base64Encoder
                                                    )                                     // StreamTransformationFilter
    );                                                                                    // StringSource
    return {true, QString::fromStdString(cipher)};
  }
  catch (const CryptoPP::Exception &ex)
  {
    std::cerr << ex.what() << endl;
    CUtils::exiter("failed in AESencrypt 0.2.0", 908);
    return {false, ""};
  }
}

std::tuple<bool, QString> CCrypto::AESdecrypt(
    const QString &cipher,
    const QString &secret_key,
    const QString &initialization_vector,
    const QString &aes_version)
{
  if (aes_version == "0.0.0")
  {
    // only base 64 decryption
    return {true, base64Decode(cipher)};
  }
  else if (aes_version == "0.1.0")
  {
    // compatible version with legacy javascript clients. FIXME: implement it ASAP
    CLog::log("not suppoerted AES version", "app", "error");
    return {false, ""};
  }
  else if (aes_version == "0.2.0")
  {
    return AESdecrypt020(cipher, secret_key, initialization_vector, aes_version);
  }

  return {false, ""};
}

std::tuple<bool, QString> CCrypto::AESdecrypt020(
    const QString &cipher,
    const QString &secret_key,
    const QString &initialization_vector,
    const QString &aes_version)
{
  using namespace CryptoPP;
  AutoSeededRandomPool prng;

  std::string recovered;
  Q_UNUSED(aes_version);

  try
  {
    SecByteBlock aes_key(NULL, 32);
    SecByteBlock ivF(NULL, 16);

    std::string secret_key_ = secret_key.toStdString();
    for (uint i = 0; i < secret_key_.length(); i++)
    {
      aes_key[i] = secret_key_[i];
    }

    std::string initialization_vector_ = initialization_vector.toStdString();
    for (uint i = 0; i < initialization_vector_.length(); i++)
    {
      ivF[i] = initialization_vector_[i];
    }

    CBC_Mode<AES>::Decryption dec1(aes_key, sizeof(aes_key), ivF);

    StringSource ss(cipher.toStdString(), true,
                    new Base64Decoder(
                        new StreamTransformationFilter(dec1,
                                                       new StringSink(recovered)) // StreamTransformationFilter
                        )                                                         // base64
    );                                                                            // StringSource

    return {true, QString::fromStdString(recovered)};
  }
  catch (const CryptoPP::Exception &e)
  {
    std::cerr << e.what() << endl;
    return {false, ""};
  }
}

QString CCrypto::getRandomNumber(int len)
{
  // FIXME: maybe use more random source (e.g. open ssl)
  srand(time(NULL));
  QString res = QString::number(rand());
  while (res.length() < len)
    res += QString::number(rand());

  return res.midRef(0, len).toString();
}

QString CCrypto::keccak256(const uint64_t num)
{
  return keccak256(QString::number(num));
}

QString CCrypto::keccak256(const uint32_t num)
{
  return keccak256(QString::number(num));
}

QString CCrypto::keccak256(const int64_t num)
{
  return keccak256(QString::number(num));
}

QString CCrypto::keccak256(const int32_t num)
{
  return keccak256(QString::number(num));
}

QString CCrypto::keccak256(const QString &msg)
{
  std::string m = msg.toStdString();
  //  using namespace CryptoPP;
  CryptoPP::Keccak_256 hash;
  std::string hashed;
  CryptoPP::HexEncoder encoder(new CryptoPP::StringSink(hashed));
  std::string digest;
  hash.Update((const byte *)m.data(), m.size());
  digest.resize(hash.DigestSize());
  hash.Final((byte *)&digest[0]);
  CryptoPP::StringSource(digest, true, new CryptoPP::Redirector(encoder));
  std::transform(hashed.begin(), hashed.end(), hashed.begin(), [](unsigned char c)
                 { return std::tolower(c); });
  return QString::fromStdString(hashed);
}

QString CCrypto::keccak256Dbl(const QString &msg)
{
  return keccak256(keccak256(msg));
}

QString CCrypto::sha256Dbl(const QString &msg)
{
  return sha256(sha256(msg));
}

QString CCrypto::sha256(const QString &msg)
{
  std::string m = msg.toStdString();
  CryptoPP::SHA256 hash;
  std::string hashed;
  CryptoPP::HexEncoder encoder(new CryptoPP::StringSink(hashed));
  std::string digest;
  hash.Update((const byte *)m.data(), m.size());
  digest.resize(hash.DigestSize());
  hash.Final((byte *)&digest[0]);
  CryptoPP::StringSource(digest, true, new CryptoPP::Redirector(encoder));
  std::transform(hashed.begin(), hashed.end(), hashed.begin(), [](unsigned char c)
                 { return std::tolower(c); });
  return QString::fromStdString(hashed);
}

QString CCrypto::base64Encode(const QString &decoded)
{
  QByteArray ba;
  ba.append(decoded);
  return QString(ba.toBase64());
}

QString CCrypto::base64Decode(const QString &encoded)
{
  QByteArray ba;
  ba.append(encoded);
  return QString(QByteArray::fromBase64(ba));
}

QString CCrypto::bech32Encode(const QString &str)
{
  return bech32::ComenEncode(str, "im");
}

bool CCrypto::isValidBech32(const QString &str)
{
  return bech32::ComenIsValid(str.toStdString());
}

std::pair<std::string, std::vector<uint8_t> > CCrypto::bech32Decode(const QString &str)
{
  return bech32::Decode(str.toStdString());
}

std::tuple<bool, std::string> CCrypto::stripHeaderAndFooter(const std::string &s, const std::string &header, const std::string &footer)
{

  size_t pos1, pos2;
  pos1 = s.find(header);
  if (pos1 == std::string::npos)
    return {false, ""};

  pos2 = s.find(footer, pos1 + 1);
  if (pos2 == std::string::npos)
    return {false, ""};

  // Start position and length
  pos1 = pos1 + header.length();
  pos2 = pos2 - pos1;
  return {true, s.substr(pos1, pos2)};
}
*/






// - - - - - - - - ECDSA key managements - - - - - - -

//old_name_was ECDSAGenerateKeyPair
pub fn ecdsa_generate_key_pair() -> (bool, String, String)
{
    let secp = Secp256k1::new();
    let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);
    (true, secret_key.display_secret().to_string(), public_key.to_string())
}

//old_name_was ECDSAimportHexPrivateKey
pub fn ecdsa_import_hex_private_key(private_key_string: &String) -> (bool, SecretKey)
{
    (true, SecretKey::from_str(private_key_string).unwrap())
}

//old_name_was ECDSAimportHexPublicKey
pub fn ecdsa_import_hex_public_key(compact_point: &String) -> (bool, PublicKey)
{
    (true, PublicKey::from_str(&compact_point ).unwrap())
}

//old_name_was ECDSAValidatePrivateKey
pub fn ecdsa_validate_private_key(hex_private_key: &String) -> bool {
    let (status, _) = ecdsa_import_hex_private_key(hex_private_key);
    status
}

//old_name_was ECDSAValidatePublicKey
pub fn ecdsa_validate_public_key(hex_public_key: &String) -> bool {
    let (status, _) = ecdsa_import_hex_public_key(hex_public_key);
    status
}


// std::tuple<bool, std::string, std::string>
//old_name_was ECDSAsignMessage
pub fn ecdsa_sign_message(private_key: &String, message: &String) -> (bool, String, Signature) {
    let (private_status, regenerated_private_key) = ecdsa_import_hex_private_key(private_key);
    let message = Message::from_hashed_data::<sha256::Hash>(message.as_bytes());
    let secp = Secp256k1::new();
    let sig: Signature = secp.sign_ecdsa(&message, &regenerated_private_key);
    if !private_status { return (false, "".to_string(), sig) };
    let signature_string: String = sig.to_string();
    return (true, signature_string, sig);
}

//old_name_was ECDSAVerifysignature
pub fn ecdsa_verify_signature(public_key: &String, message: &String, signature_string: &String) -> bool {
    let secp = Secp256k1::new();
    let re_sig = Signature::from_str(&signature_string).unwrap();
    let (staus, public_key) = ecdsa_import_hex_public_key(public_key);
    if !staus { return false; }
    let message = Message::from_hashed_data::<sha256::Hash>(message.as_bytes());
    secp.verify_ecdsa(&message, &re_sig, &public_key).is_ok()
}

/*















// - - - - - - - - native RSA key managements - - - - - - -

bool CCrypto::VerifyMessage(const CryptoPP::ECDSA<CryptoPP::ECP, CryptoPP::SHA256>::PublicKey &key, const std::string &message, const std::string &signature)
{
  using namespace CryptoPP;
  bool result = false;

  StringSource(signature + message, true,
               new SignatureVerificationFilter(
                   ECDSA<ECP, SHA1>::Verifier(key),
                   new ArraySink((byte *)&result, sizeof(result))) // SignatureVerificationFilter
  );

  return result;
}






*/


/*
// QString signMsg(const QString& message, const QString& private_key)
//{
//   // FIXME: implement it
//   return "";

//}

// bool verifySignature(const QString& signed_message, const QString& signature, const QString&  public_key)
//{
//   // FIXME: implement it
//   return true;
// }








//  -  -  -  Native RSA part codes -  -  -  -  -  -  -  -

 */

//old_name_was nativeGenerateKeyPair
pub fn native_generate_key_pair() -> (bool, String, String) {
    let secp = Secp256k1::new();
    let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);
    return (true, secret_key.display_secret().to_string(), public_key.to_string());
}

//old_name_was isValidRSAPrivateKey

/*
pub fn is_valid_rsa_private_key(prv_key: &String)
{
auto [isValid, prv] = read_PEM_private_key(prv_key.toStdString());
Q_UNUSED(prv);
return isValid;
}

std::tuple<bool, CryptoPP::RSA::PublicKey> CCrypto::read_PEM_public_key(std::string RSA_PUB_KEY, const std::string &header, const std::string &footer)
{
  using namespace CryptoPP;
  CryptoPP::RSA::PublicKey rsa;
  try
  {
    auto [status, strPrivKey] = stripHeaderAndFooter(RSA_PUB_KEY, header, footer);
    if (!status)
      return {false, rsa};

    ByteQueue t1, t2;
    StringSource ss(strPrivKey, true,
                    new CryptoPP::Redirector(t1));
    CPEM_Base64Decode(t1, t2);
    CPEM_LoadPublicKey(t2, rsa, true);
    return {true, rsa};
  }
  catch (const CryptoPP::Exception &ex)
  {
    return {false, rsa};
  }
}

void CCrypto::CPEM_LoadPublicKey(CryptoPP::BufferedTransformation &src, CryptoPP::X509PublicKey &key, bool subjectInfo)
{
  using namespace CryptoPP;
  X509PublicKey &pk = dynamic_cast<X509PublicKey &>(key);

  if (subjectInfo)
    pk.Load(src);
  else
    pk.BERDecode(src);

#define PEM_KEY_OR_PARAMETER_VALIDATION
#if defined(PEM_KEY_OR_PARAMETER_VALIDATION) && !defined(NO_OS_DEPENDENCE)
  AutoSeededRandomPool prng;
  if (!pk.Validate(prng, 2))
    throw Exception(Exception::OTHER_ERROR, "PEM_LoadPublicKey: key validation failed");
#endif
}

void CCrypto::CPEM_Base64Decode(CryptoPP::BufferedTransformation &source, CryptoPP::BufferedTransformation &dest)
{
  using namespace CryptoPP;
  Base64Decoder decoder(new Redirector(dest));
  source.TransferTo(decoder);
  decoder.MessageEnd();
}

void CCrypto::CPEM_Base64Encode(CryptoPP::BufferedTransformation &source, CryptoPP::BufferedTransformation &dest)
{
  using namespace CryptoPP;
  Base64Encoder encoder(new Redirector(dest), true, 64);
  source.TransferTo(encoder);
  encoder.MessageEnd();
}

bool CCrypto::isValidRSAPublicKey(const QString &pubKey)
{
  auto [isValid, publicKey] = read_PEM_public_key(pubKey.toStdString());
  Q_UNUSED(publicKey);
  return isValid;
}


void CCrypto::CPEM_LoadPrivateKey(CryptoPP::BufferedTransformation &src, CryptoPP::PKCS8PrivateKey &key, bool subjectInfo)
{
  using namespace CryptoPP;
  if (subjectInfo)
    key.Load(src);
  else
    key.BERDecodePrivateKey(src, 0, src.MaxRetrievable());

#define PEM_KEY_OR_PARAMETER_VALIDATION
#if defined(PEM_KEY_OR_PARAMETER_VALIDATION) && !defined(NO_OS_DEPENDENCE)
  AutoSeededRandomPool prng;
  if (!key.Validate(prng, 2))
    throw Exception(Exception::OTHER_ERROR, "PEM_LoadPrivateKey: key validation failed");
#endif
}

std::tuple<bool, CryptoPP::RSA::PrivateKey> CCrypto::read_PEM_private_key(std::string rsa_priv_key, const std::string &header, const std::string &footer)
{
  using std::string, std::runtime_error;
  using namespace CryptoPP;
  CryptoPP::RSA::PrivateKey rsa;
  try
  {
    auto [status, strPrivKey] = stripHeaderAndFooter(rsa_priv_key, header, footer);
    if (!status)
      return {false, rsa};

    CryptoPP::ByteQueue t1, t2;
    StringSource ss(strPrivKey, true,
                    new CryptoPP::Redirector(t1));
    CPEM_Base64Decode(t1, t2);
    CPEM_LoadPrivateKey(t2, rsa, true);
    return {true, rsa};
  }
  catch (const CryptoPP::Exception &ex)
  {
    std::cerr << ex.what() << endl;
    return {false, rsa};
  }
}

QString CCrypto::nativeSign(const QString &prvKey, const QString &msg)
{
  using namespace CryptoPP;
  auto [isValid, privateKey] = read_PEM_private_key(prvKey.toStdString());
  Q_UNUSED(isValid);
  //  privateKey.BEREncode();

  //  PKCS8PrivateKey
  std::string message, signature;
  message = msg.toStdString();
  // Sign and Encode
  //  RSASSA_PKCS1v15_SHA_Signer signer(privateKey);
  RSASS<PKCS1v15, SHA256>::Signer signer(privateKey);
  CryptoPP::AutoSeededRandomPool rng;

  // Create signature space
  size_t length = signer.MaxSignatureLength();
  SecByteBlock signature2(length);

  // Sign message
  length = signer.SignMessage(
      rng, (const byte *)message.c_str(),
      message.length(), signature2);

  // Resize now we know the true size of the signature
  signature2.resize(length);

  // base64-encode signature to pass to forge
  std::string encoded;
  CryptoPP::StringSource ss(
      signature2.data(), signature2.size(), true,
      new CryptoPP::Base64Encoder(
          new CryptoPP::StringSink(encoded)));

  return CUtils::removeNewLine(QString::fromStdString(encoded));
};

bool CCrypto::nativeVerifySignature(const QString &publicKeyStr, const QString &msg, const QString &sig)
{

  return true; // FIXME: implement it ASAP

  //  https://github.com/digitalbazaar/forge/issues/117
  using namespace CryptoPP;
  std::string message = msg.toStdString();
  std::string signature = sig.toStdString();
  auto [isValid, publicKey] = read_PEM_public_key(publicKeyStr.toStdString());
  if (!isValid)
    return isValid;

  RSASS<PKCS1v15, SHA256>::Verifier verifier(publicKey);
  auto sss = (const byte *)signature.c_str();
  bool result = verifier.VerifyMessage((const byte *)message.c_str(),
                                       message.length(), sss, signature.size());
  return result;
}

QString CCrypto::decryptStringWithPrivateKey(const QString &privateKeyStr, const QString &toDecrypt)
{
  using namespace CryptoPP;
  auto [isValid, privateKey] = read_PEM_private_key(privateKeyStr.toStdString());
  Q_UNUSED(isValid);
  std::string cipher = (toDecrypt).toStdString();
  std::string recovered;

  // Decryption
  //  RSAES_OAEP_SHA_Decryptor d(privateKey);
  RSAES<OAEP<SHA1> >::Decryptor d(privateKey);
  CryptoPP::AutoSeededRandomPool rng;

  StringSource ss2(cipher, true,
                   new CryptoPP::Base64Decoder(
                       new PK_DecryptorFilter(rng, d,
                                              new StringSink(recovered)) // PK_DecryptorFilter
                       ));                                               // StringSource

  return QString::fromStdString(recovered);
};

QString CCrypto::encryptStringWithPublicKey(const QString &publicKeyStr, const QString &toEncrypt)
{
  std::string plain = toEncrypt.toStdString();
  std::string cipher = "";

  using namespace CryptoPP;
  auto [isValid, publicKey] = read_PEM_public_key(publicKeyStr.toStdString());
  Q_UNUSED(isValid);
  CryptoPP::AutoSeededRandomPool rng;

  // Encryption
  RSAES<OAEP<SHA1> >::Encryptor e(publicKey);

  StringSource ss1(plain, true,
                   new PK_EncryptorFilter(rng, e,
                                          new CryptoPP::Base64Encoder(
                                              new StringSink(cipher))) // PK_EncryptorFilter
  );                                                                   // StringSource

  return CUtils::removeNewLine(QString::fromStdString(cipher));
};

 */