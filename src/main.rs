extern crate core;

use std::env;
// use der::Encode;
// use pkcs1::LineEnding;
// use rsa::pkcs8::{EncodePrivateKey, EncodePublicKey};
// use std::fmt::Display;
// use tokio::task;
// use tokio::time::{sleep, Duration};


// use std::thread;
// use std::thread::sleep;
// use std::time::Duration;

// use lib::c_log::log;

mod config;
mod lib;
mod tests;

// use std::thread::sleep;
// use std::time::Duration;

// use crate::lib::constants as CConsts;
// use lib::threads_handler::launch_threads;
use lib::machine::machine_handler as machine_handler;
use lib::utils::cutils as cutils;

use crate::lib::ccrypto;

// use crate::tests::unit_tests::cutils::test_chunk_qstring_list;

#[tokio::main]
async fn main() {
    //! # Diamond, the Community Maker Engine
    //! ```
    //! fn main()
    //! ```
    //!
    //! This starts whole game
    //!
    //!
    //!
    //!
    //!

    // let resss = cutils::chunk_string(&"a123456789bcdef0a123456789bcdef0".to_string(), 16);
    // println!("in main chunk_string: {:?}", resss.len());
    // println!("in main chunk_string: {:?}", resss);

    let (status, pem_prv_key, pem_pub_key) = ccrypto::rsa_generate_key_pair();
    // println!("privk: {}", pem_prv_key);
    let message = "The message to be signed and encrypt and decrypt".to_string();// to be signed and encrypt and decrypt
    let (sign_status, signature) = ccrypto::rsa_sign(
        &pem_prv_key,
        &message,
    );
    assert!(sign_status);
    assert!(ccrypto::rsa_verify_signature(&pem_pub_key, &message, &signature));

    let (enc_status, enc_msg) = ccrypto::rsa_encrypt_with_pub_key(&pem_pub_key, &message);
    assert!(enc_status);
    println!("in main enc_msg: {}", enc_msg);

    let (dec_status, dec_msg) = ccrypto::rsa_decrypt_with_prv_key(&pem_prv_key, &enc_msg);
    assert!(dec_status);
    println!("in main dec_msg: {}", dec_msg);


    use rsa::{PublicKey, RsaPrivateKey, RsaPublicKey, PaddingScheme};

    let mut rng = rand::thread_rng();

    let bits = 256;
    let private_key = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate a key");
    let public_key = RsaPublicKey::from(&private_key);

// Encrypt
    let data = b"hello world";
    let padding = PaddingScheme::new_pkcs1v15_encrypt();
    let enc_data = public_key.encrypt(&mut rng, padding, &data[..]).expect("failed to encrypt");
    assert_ne!(&data[..], &enc_data[..]);

// Decrypt
    let padding = PaddingScheme::new_pkcs1v15_encrypt();
    let dec_data = private_key.decrypt(padding, &enc_data).expect("failed to decrypt");
    assert_eq!(&data[..], &dec_data[..]);


    // use rsa::{pkcs1::DecodeRsaPublicKey};
    //
    // let pem = "-----BEGIN RSA PUBLIC KEY-----
    // MIIBCgKCAQEAtsQsUV8QpqrygsY+2+JCQ6Fw8/omM71IM2N/R8pPbzbgOl0p78MZ
    // GsgPOQ2HSznjD0FPzsH8oO2B5Uftws04LHb2HJAYlz25+lN5cqfHAfa3fgmC38Ff
    // wBkn7l582UtPWZ/wcBOnyCgb3yLcvJrXyrt8QxHJgvWO23ITrUVYszImbXQ67YGS
    // 0YhMrbixRzmo2tpm3JcIBtnHrEUMsT0NfFdfsZhTT8YbxBvA8FdODgEwx7u/vf3J
    // 9qbi4+Kv8cvqyJuleIRSjVXPsIMnoejIn04APPKIjpMyQdnWlby7rNyQtE4+CV+j
    // cFjqJbE/Xilcvqxt6DirjFCvYeKYl1uHLwIDAQAB
    // -----END RSA PUBLIC KEY-----";
    //
    // let public_keyZZ = RsaPublicKey::from_pkcs1_pem(pem);
    //

    // use pkcs8::{EncodePrivateKey, DecodePrivateKey};

    // let private_pem = private_key.to_pkcs8_pem(rsa::pkcs1::LineEnding::CRLF).expect("Failed in private convert to pem");
    // let private_pem = format!("{:?}", private_pem.to_string());
    // println!("private_pem: {}", private_pem);

    // let mut public_pem:String = match public_key.to_public_key_pem(rsa::pkcs1::LineEnding::CRLF) {
    //     Ok(the_pub)=>{
    //         println!("Success generate pem pub key(RSA) {}", the_pub);
    //         the_pub.clone()
    //     },
    //     Err(e)=>{
    //         println!("Error in generate pem pub key(RSA) {}", e);
    //         "".to_string()
    //     },
    // };
    // // let public_pem = format!("{:?}", public_pem);
    // println!("public_pem: {}", public_pem);


    // let tt:RsaPrivateKey = pkcs8::DecodePrivateKey::from_pkcs8_pem(&private_pem).expect("reed");
    // use rsa::{RsaPublicKey, pkcs8::DecodePublicKey};

//         use rsa::{RsaPublicKey, pkcs8::DecodePublicKey};
//     {
//
// //         let pem = "-----BEGIN PUBLIC KEY-----
// // MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAtsQsUV8QpqrygsY+2+JC
// // Q6Fw8/omM71IM2N/R8pPbzbgOl0p78MZGsgPOQ2HSznjD0FPzsH8oO2B5Uftws04
// // LHb2HJAYlz25+lN5cqfHAfa3fgmC38FfwBkn7l582UtPWZ/wcBOnyCgb3yLcvJrX
// // yrt8QxHJgvWO23ITrUVYszImbXQ67YGS0YhMrbixRzmo2tpm3JcIBtnHrEUMsT0N
// // fFdfsZhTT8YbxBvA8FdODgEwx7u/vf3J9qbi4+Kv8cvqyJuleIRSjVXPsIMnoejI
// // n04APPKIjpMyQdnWlby7rNyQtE4+CV+jcFjqJbE/Xilcvqxt6DirjFCvYeKYl1uH
// // LwIDAQAB
// // -----END PUBLIC KEY-----";
//
//         let re_public_key = RsaPublicKey::from_public_key_pem(&public_pem);
//         // let public_ddd = format!("{:?}", re_public_key);
//         // println!("re_public_key2(RSA) {}", re_public_key2)
//         match re_public_key{
//             Ok(re_public_key2)=>{
//                 println!("re_public_key2(RSA) done",);
//             },
//             Err(e)=>{
//                 println!("Error in regenerate pub key(RSA) {}", e);
//             }
//         }
//
//     }

// config::print_config();

// use Merkle crate, if exist

    let manual_clone_id: i8 = 0;
// CMachine::onAboutToQuit(&w);

    machine_handler::CMachine::init();
    machine_handler::CMachine::parse_args(env::args().collect(), manual_clone_id);

    /*

      InitCCrypto::init();

      CMachine::setLaunchDateAndCloneId("2021-03-02 00:20:00", manual_clone_id);

      w.initMachineEnvironment();

      if (true)
      {
        dummyTestsHandler();
      }
         */

// launch_threads().await;

// sleep(Duration::from_secs(5)).await;
}

