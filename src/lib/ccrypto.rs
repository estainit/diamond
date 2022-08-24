use rand_core::OsRng;
use std::str::FromStr;
use secp256k1::hashes::{sha256};
use secp256k1::{Secp256k1, Message, SecretKey, PublicKey,
                ecdsa::{Signature}};
use rsa::{RsaPrivateKey, RsaPublicKey,
          PaddingScheme, PublicKey as rsa_pub, //pkcs1::LineEnding,
          pkcs8::{EncodePrivateKey, DecodePrivateKey, EncodePublicKey, DecodePublicKey}};

use base64::{encode, decode};
use std::str;
use rand::Rng;

use aes::Aes128;
use aes::cipher::{
    BlockEncrypt, BlockDecrypt, KeyInit,
    generic_array::GenericArray,
};

use substring::Substring;
use crate::constants;

use crate::lib::utils::cutils as cutils;
use crate::lib::bech_32;
use crate::lib::dlog::dlog;

//old name was isValidBech32
pub fn is_valid_bech32(inp_str: &str) -> bool
{
    return bech_32::comen_is_valid(inp_str);
}

//old_name_was bech32Encode
pub fn bech32_encode(str: &str) -> String
{
    bech_32::comen_encode(str, constants::SOCIETY_NAME)
}

pub fn keccak256(msg: &str) -> String {
    use crypto::{sha3::Sha3, digest::Digest};
    let mut hasher = Sha3::keccak256();
    hasher.input_str(msg);
    let hex = hasher.result_str();
    hex
}

//old_name_was keccak256Dbl
pub fn keccak256_dbl(msg: &str) -> String
{
    keccak256(&keccak256(msg))
}

//old_name_was convertTitleToHash
pub fn convert_title_to_hash(title: &String) -> String {
    keccak256(title)
}


#[allow(dead_code, unused)]
pub const PUBLIC_BEGIN: &str = "-----BEGIN PUBLIC KEY-----";
#[allow(dead_code, unused)]
pub const PUBLIC_END: &str = "-----END PUBLIC KEY-----";
#[allow(dead_code, unused)]
pub const PRIVATE_BEGIN: &str = "-----BEGIN PRIVATE KEY-----";
#[allow(dead_code, unused)]
pub const PRIVATE_END: &str = "-----END PRIVATE KEY-----";
#[allow(dead_code, unused)]
pub const RSA_PRIVATE_BEGIN: &str = "-----BEGIN RSA PRIVATE KEY-----";
#[allow(dead_code, unused)]
pub const RSA_PRIVATE_END: &str = "-----END RSA PRIVATE KEY-----";

#[allow(unused, dead_code)]
pub fn do_sha256(_message: &String) -> String {
    "".to_string()
}

pub fn sha256_dbl(_message: &String) -> String {
    "".to_string()
}


pub fn b64_encode(message: &String) -> String {
    encode(message.as_bytes())
}

#[allow(unreachable_code)]
pub fn b64_decode(message: &String) -> (bool, String)
{
    let (status, buf) = match decode(&message) {
        Ok(r) => (true, r),
        Err(e) => {
            dlog(&format!("Failed in b64 decoding: {} {}", message, e),
                 constants::Modules::Sec,
                 constants::SecLevel::Error);
            panic!("ttttttttt");
            (false, vec![])
        }
    };
    if !status
    { return (false, "".to_string()); }

    let (status, out) = match str::from_utf8(&buf) {
        Ok(v) => (true, v.to_string()),
        Err(e) => {
            dlog(&format!("Failed in b64 decoding to string: {} {}", message, e),
                 constants::Modules::Sec,
                 constants::SecLevel::Error);
            (false, "".to_string())
        }
    };
    return (status, out);
}

//old_name_was getRandomNumber
pub fn get_random_number_as_string(length: u8) -> String
{
    // FIXME: maybe use more random source
    let mut rng = rand::thread_rng();

    let mut rnd_str = "".to_string();
    while rnd_str.len() < length as usize {
        rnd_str += &*format!("{}", rng.gen::<u32>());
    }

    rnd_str.substring(0, length as usize).to_string()
}

pub fn get_random_number(max: u32) -> u32
{
    rand::thread_rng().gen_range(0..max)
}

pub fn aes_encrypt(msg: String, key: String) -> (bool, String)
{
    // FIXME: audit, security check and refactor this aes codes
    let msg = msg.trim().to_string();
    let chunks = cutils::chunk_string(&msg, 16);
    let mut out: String = "".to_string();
    let mut a_block: String;
    for a_chunk in chunks
    {
        if a_chunk.len() < 16
        {
            a_block = aes_encrypt_16(cutils::right_padding_custom(a_chunk, 16, " ".to_string()), &key);
        } else {
            a_block = aes_encrypt_16(a_chunk, &key);
        }
        out += &*a_block;
    }
    return (true, out);
}

pub fn aes_encrypt_16(msg: String, key: &String) -> String
{
    let key = <[u8; 16]>::try_from(key.as_bytes()).unwrap();
    let key = GenericArray::from(key);//[42u8; 16]

    let block: [u8; 16] = <[u8; 16]>::try_from(msg.as_bytes()).unwrap();
    let mut block = GenericArray::from(block);//[42u8; 16]

    // Initialize cipher
    let cipher = Aes128::new(&key);

    // Encrypt block in-place
    cipher.encrypt_block(&mut block);
    let s = encode(block);
    return s;
}

pub fn aes_decrypt(encrypted_msg: String, key: String, _aes_version: String) -> (bool, String)
{
    let chunks = cutils::chunk_string(&encrypted_msg, 24);

    let mut out: String = "".to_string();
    let mut a_block: String;
    for a_chunk in chunks
    {
        if a_chunk.len() < 24
        {
            panic!("inproper cipher length! {}", a_chunk);
        } else {
            a_block = aes_decrypt_24(a_chunk, &key);
        }
        out += &*a_block;
    }

    return (true, out.trim().to_string());
}

pub fn aes_decrypt_24(msg: String, key: &String) -> String
{
    let key = <[u8; 16]>::try_from(key.as_bytes()).unwrap();
    let key = GenericArray::from(key);//[42u8; 16]

    let block = decode(msg).unwrap();
    let block: [u8; 16] = <[u8; 16]>::try_from(block).unwrap();
    let mut block = GenericArray::from(block);//[42u8; 16]

    let cipher = Aes128::new(&key);
    cipher.decrypt_block(&mut block);
    let s = String::from_utf8(block.to_vec()).expect("Found invalid UTF-8 in deciphering aes!");
    return s;
}

/*

using std::cout, std::endl;

//  -  -  -  general

std::tuple<bool, String> ccrypto::AESencrypt(
    const String &clear_text,
    const String &secret_key,
    const String &initialization_vector,
    const String &aes_version)
{
  if (aes_version == "0.0.0")
  {
    // no encryption
    //    String out = "{";
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

std::tuple<bool, String> ccrypto::AESencrypt020(
    const String &clear_text,
    const String &secret_key,
    const String &initialization_vector,
    const String &aes_version)
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
    //  read_key(aes_key, aes_key.len());
    std::string secret_key_ = secret_key.toStdString();
    for (uint i = 0; i < secret_key_.len(); i++)
    {
      aes_key[i] = secret_key_[i];
    }

    // stub for how you really get it, e.g. filling it with random bytes or
    //   reading it from the other side of the socket since both sides have
    //   to use the same IV as well as the same key
    //  read_initialization_vector(iv);
    std::string initialization_vector_ = initialization_vector.toStdString();
    for (uint i = 0; i < initialization_vector_.len(); i++)
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
    return {true, String::fromStdString(cipher)};
  }
  catch (const CryptoPP::Exception &ex)
  {
    std::cerr << ex.what() << endl;
    cutils::exiter("failed in AESencrypt 0.2.0", 908);
    return {false, ""};
  }
}

std::tuple<bool, String> ccrypto::AESdecrypt(
    const String &cipher,
    const String &secret_key,
    const String &initialization_vector,
    const String &aes_version)
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

std::tuple<bool, String> ccrypto::AESdecrypt020(
    const String &cipher,
    const String &secret_key,
    const String &initialization_vector,
    const String &aes_version)
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
    for (uint i = 0; i < secret_key_.len(); i++)
    {
      aes_key[i] = secret_key_[i];
    }

    std::string initialization_vector_ = initialization_vector.toStdString();
    for (uint i = 0; i < initialization_vector_.len(); i++)
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

    return {true, String::fromStdString(recovered)};
  }
  catch (const CryptoPP::Exception &e)
  {
    std::cerr << e.what() << endl;
    return {false, ""};
  }
}

String ccrypto::sha256Dbl(const String &msg)
{
  return sha256(sha256(msg));
}



String ccrypto::base64Encode(const String &decoded)
{
  QByteArray ba;
  ba.push(decoded);
  return String(ba.toBase64());
}

String ccrypto::base64Decode(const String &encoded)
{
  QByteArray ba;
  ba.push(encoded);
  return String(QByteArray::fromBase64(ba));
}

String ccrypto::bech32_encode(const String &str)
{
  return bech32::ComenEncode(str, "im");
}

bool ccrypto::isValidBech32(const String &str)
{
  return bech32::ComenIsValid(str.toStdString());
}

std::pair<std::string, std::vector<uint8_t> > ccrypto::bech32Decode(const String &str)
{
  return bech32::Decode(str.toStdString());
}

std::tuple<bool, std::string> ccrypto::stripHeaderAndFooter(const std::string &s, const std::string &header, const std::string &footer)
{

  size_t pos1, pos2;
  pos1 = s.find(header);
  if (pos1 == std::string::npos)
    return {false, ""};

  pos2 = s.find(footer, pos1 + 1);
  if (pos2 == std::string::npos)
    return {false, ""};

  // Start position and length
  pos1 = pos1 + header.len();
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
    (true, PublicKey::from_str(&compact_point).unwrap())
}

//old_name_was ECDSAValidatePrivateKey
#[allow(unused, dead_code)]
pub fn ecdsa_validate_private_key(hex_private_key: &String) -> bool {
    let (status, _) = ecdsa_import_hex_private_key(hex_private_key);
    status
}

//old_name_was ECDSAValidatePublicKey
#[allow(unused, dead_code)]
pub fn ecdsa_validate_public_key(hex_public_key: &String) -> bool {
    let (status, _) = ecdsa_import_hex_public_key(hex_public_key);
    status
}

#[allow(unused, dead_code)]
pub fn ecdsa_generate_pub_by_priv(hex_private_key: &String) -> String {
    let secp = Secp256k1::new();
    let (_, secret_key) = ecdsa_import_hex_private_key(hex_private_key);
    let pk2 = PublicKey::from_secret_key(&secp, &secret_key);
    let public_key_string2: String = pk2.to_string();
    public_key_string2
}

// std::tuple<bool, std::string, std::string>
//old_name_was ECDSAsignMessage
pub fn ecdsa_sign_message(private_key: &String, message: &String) -> (bool, String, Signature) {
    let (private_status, regenerated_private_key) = ecdsa_import_hex_private_key(private_key);
    let message = Message::from_hashed_data::<sha256::Hash>(message.as_bytes());
    let secp = Secp256k1::new();
    let sig: Signature = secp.sign_ecdsa(&message, &regenerated_private_key);
    if !private_status { return (false, "".to_string(), sig); };
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

bool ccrypto::VerifyMessage(const CryptoPP::ECDSA<CryptoPP::ECP, CryptoPP::SHA256>::PublicKey &key, const std::string &message, const std::string &signature)
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
// String signMsg(const String& message, const String& private_key)
//{
//   // FIXME: implement it
//   return "";

//}

// bool verifySignature(const String& signed_message, const String& signature, const String&  public_key)
//{
//   // FIXME: implement it
//   return true;
// }








//  -  -  -  Native RSA part codes -  -  -  -  -  -  -  -

 */

//old_name_was nativeGenerateKeyPair
pub fn rsa_generate_key_pair(key_length: usize) -> (bool, String, String) {
    let mut rng = rand::thread_rng();
    let mut bits = key_length;
    if bits == 0
    { bits = constants::DEFAULT_RSA_KEY_LENGTH }
    let private_key: RsaPrivateKey = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate a key");
    let public_key: RsaPublicKey = RsaPublicKey::from(&private_key);

    // use rsa::{pkcs1::DecodeRsaPublicKey};

    // use pkcs8::{EncodePrivateKey};

    let private_pem = rsa_convert_prv_obj_to_pem_str(private_key);
    // let private_pem = private_key.to_pkcs8_pem(rsa::pkcs1::LineEnding::CRLF).expect("Failed in private convert to pem");
    // let private_pem = format!("{:?}", private_pem.to_string());

    let public_pem: String = rsa_convert_pub_obj_to_pem_str(public_key);
    // let mut public_pem:String = match public_key.to_public_key_pem(rsa::pkcs1::LineEnding::CRLF) {
    //     Ok(the_pub)=>{
    //         println!("Success generate pem pub key(RSA) {}", the_pub);
    //         the_pub.clone()
    //     },
    //     Err(e)=>{
    //         dlog(&format!("Failed in public convert to pem: {}", e), constants::Modules::App, constants::SecLevel::Error);
    //         "".to_string()
    //     },
    // };

    // let public_pem = public_key.to_public_key_pem(rsa::pkcs1::LineEnding::CRLF).expect("");
    // let public_pem = format!("{:?}", public_pem);

    (true, private_pem, public_pem)
}


//old_name_was read_PEM_private_key
pub fn rsa_read_pem_prv_key(pem_prv_key: &String) -> RsaPrivateKey
{
    RsaPrivateKey::from_pkcs8_pem(pem_prv_key).unwrap()
}

//old_name_was isValidRSAPrivateKey
#[allow(unused, dead_code)]
pub fn rsa_is_valid_prv_key(pem_prv_key: &String) -> bool
{
    return match RsaPrivateKey::from_pkcs8_pem(pem_prv_key) {
        Ok(_re_prv_key) => {
            true
        }
        Err(e) => {
            dlog(&format!("Failed in regenerate prv key from pem(RSA) {}", e), constants::Modules::App, constants::SecLevel::Error);
            false
        }
    };
}

//old_name_was read_PEM_public_key
pub fn rsa_read_pem_pub_key(pem_pub_key: &String) -> RsaPublicKey {
    RsaPublicKey::from_public_key_pem(pem_pub_key).unwrap()
}

//old_name_was isValidRSAPublicKey
pub fn rsa_is_valid_pub_key(pem_pub_key: &String) -> bool
{
    return match RsaPublicKey::from_public_key_pem(pem_pub_key) {
        Ok(_re_public_key) => {
            true
        }
        Err(e) => {
            dlog(&format!("Failed in regenerate pub key from pem(RSA) {}", e), constants::Modules::App, constants::SecLevel::Error);
            false
        }
    };
}

pub fn rsa_convert_prv_obj_to_pem_str(private_key: RsaPrivateKey) -> String {
    // use std::str;
    let private_pem = private_key.to_pkcs8_pem(rsa::pkcs1::LineEnding::CRLF).expect("Failed in private convert to pem");
    let private_pem = private_pem.to_string();
    return private_pem;
    // match private_pem.to_vec(){
    //     Ok(prv_vec) => {
    //         println!("Success generate pem pub key(RSA) {:?}", prv_vec);
    //         let sparkle_heart = str::to_string(&prv_vec).unwrap();
    //
    //         // let stuff_str: String = prv_vec.into_iter().map(|i| i.to_string()).collect::<String>();
    //         // let s = format!("{:?}", &prv_vec);
    //         sparkle_heart.to_string().clone()
    //     }
    //     Err(e) => {
    //         dlog(&format!("Failed in public convert to pem: {}", e), constants::Modules::App, constants::SecLevel::Error);
    //         "".to_string()
    //     }
    // }
}

pub fn rsa_convert_pub_obj_to_pem_str(public_key: RsaPublicKey) -> String {
    match public_key.to_public_key_pem(rsa::pkcs1::LineEnding::CRLF) {
        Ok(the_pub) => {
            the_pub.clone()
        }
        Err(e) => {
            dlog(&format!("Failed in public convert to pem: {}", e), constants::Modules::App, constants::SecLevel::Error);
            "".to_string()
        }
    }
}

//old_name_was nativeSign
pub fn rsa_sign(pem_prv_key: &String, msg: &String) -> (bool, String) {
    let prv_key: RsaPrivateKey = rsa_read_pem_prv_key(pem_prv_key);
    let padding = PaddingScheme::new_pkcs1v15_sign(None);
    // println!("------msg.as_bytes(): {:?}", msg[0..16].as_bytes());

    match prv_key.sign(padding, msg[0..16].as_bytes()) {
        Ok(sig_vec) => {
            (true, hex::encode(sig_vec).clone().to_string())
        }
        Err(e) => {
            dlog(&format!("Failed in RSA signing: {}", e), constants::Modules::App, constants::SecLevel::Error);
            (false, "".to_string())
        }
    }
}


//old_name_was nativeVerifySignature
pub fn rsa_verify_signature(pem_pub_key: &String, message: &String, signature: &String) -> bool {
    let pub_key = rsa_read_pem_pub_key(&pem_pub_key);
    let sig = &hex::decode(signature).unwrap()[..];
    let msg = message[0..16].as_bytes();
    let padding = PaddingScheme::new_pkcs1v15_sign(None);
    match pub_key.verify(padding, msg, sig) {
        Ok(_r) => {
            return true;
        }
        Err(e) => {
            dlog(&format!("Failed in verifying RSA signature {}", e), constants::Modules::App, constants::SecLevel::Error);
            return false;
        }
    }
}

//old_name_was encryptStringWithPublicKey
pub fn rsa_encrypt_with_pub_key_16(pub_key: &RsaPublicKey, message: &String) -> (bool, String) {
    // let msg = b"hello world";
    let mut rng = rand::thread_rng();
    let msg = message[..].as_bytes();
    let padding = PaddingScheme::new_pkcs1v15_encrypt();
    let enc_data = pub_key.encrypt(&mut rng, padding, &msg[..]).expect("failed to encrypt");
    // assert_ne!(&msg[..], &enc_data[..]);
    (true, hex::encode(enc_data).clone().to_string())
}

//old_name_was encryptStringWithPublicKey
pub fn rsa_encrypt_with_pub_key(pem_pub_key: &String, message: &String) -> (bool, String) {
    let mut output: String = "".to_string();
    let pub_key = rsa_read_pem_pub_key(&pem_pub_key);
    for a_chunk in cutils::chunk_string(&message, 16) {
        let (status, a_chunk_enc) = rsa_encrypt_with_pub_key_16(&pub_key, &a_chunk);
        if !status { return (false, "".to_string()); }
        output += &a_chunk_enc;
    }
    return (true, output);
}

pub fn rsa_decrypt_with_prv_key(pem_prv_key: &String, cipher: &String) -> (bool, String) {
    let mut output: String = "".to_string();
    let prv_key = rsa_read_pem_prv_key(&pem_prv_key);
    for a_chunk in cutils::chunk_string(&cipher, 512) {
        let (status, a_chunk_dec) = rsa_decrypt_with_prv_key_512(&prv_key, &a_chunk);
        if !status { return (false, "".to_string()); }
        output += &a_chunk_dec;
    }
    return (true, output);
}

pub fn rsa_decrypt_with_prv_key_512(prv_key: &RsaPrivateKey, cipher: &String) -> (bool, String) {
    let ciph = hex::decode(cipher).unwrap();
    let padding = PaddingScheme::new_pkcs1v15_encrypt();
    match prv_key.decrypt(padding, &ciph) {
        Ok(dec_data) => {
            let decoded = format!("{}", dec_data.into_iter().map(|i| char::from(i)).collect::<String>());
            return (true, decoded.clone().to_string());
        }
        Err(e) => {
            dlog(&format!("Failed in decrypt by RSA private key {}", e), constants::Modules::App, constants::SecLevel::Error);
            return (false, "".to_string());
        }
    }
}


/*

void ccrypto::CPEM_LoadPublicKey(CryptoPP::BufferedTransformation &src, CryptoPP::X509PublicKey &key, bool subjectInfo)
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

void ccrypto::CPEM_Base64Decode(CryptoPP::BufferedTransformation &source, CryptoPP::BufferedTransformation &dest)
{
  using namespace CryptoPP;
  Base64Decoder decoder(new Redirector(dest));
  source.TransferTo(decoder);
  decoder.MessageEnd();
}

void ccrypto::CPEM_Base64Encode(CryptoPP::BufferedTransformation &source, CryptoPP::BufferedTransformation &dest)
{
  using namespace CryptoPP;
  Base64Encoder encoder(new Redirector(dest), true, 64);
  source.TransferTo(encoder);
  encoder.MessageEnd();
}



void ccrypto::CPEM_LoadPrivateKey(CryptoPP::BufferedTransformation &src, CryptoPP::PKCS8PrivateKey &key, bool subjectInfo)
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




String ccrypto::decryptStringWithPrivateKey(const String &privateKeyStr, const String &toDecrypt)
{
  using namespace CryptoPP;
  auto [isValid, privateKey] = rsa_read_pem_prv_key(privateKeyStr.toStdString());
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

  return String::fromStdString(recovered);
};


 */