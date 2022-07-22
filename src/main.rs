extern crate core;

use std::env;
// use std::fmt::Display;
// use tokio::task;
use tokio::time::{sleep, Duration};



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
use lib::threads_handler::launch_threads;
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

    let (isOk, prv1, pub1) = ccrypto::native_generate_key_pair();
    println!("RES isOk: {}", isOk);
    println!("RES prv1: {}", prv1);
    println!("RES pub1: {}", pub1);

/*
    {
        use p256::{
            ecdsa::{SigningKey, Signature, signature::Signer, },
        };
        use rand_core::{OsRng, SeedableRng}; // requires 'getrandom' feature

        // Signing
        let signing_key = SigningKey::random(&mut OsRng); // Serialize with `::to_bytes()`
        println!("signing_key: {:?}", signing_key);
        let message = b"ECDSA proves knowledge of a secret number in the context of a single message";
        let signature = signing_key.sign(message);

        // Verification
        use p256::ecdsa::{VerifyingKey, signature::Verifier};

        let verifier = VerifyingKey::from(&signing_key); // Serialize with `::to_encoded_point()`
        assert!(verifier.verify(message, &signature).is_ok());
    }

    {

        use p256::{
            ecdsa::{signature::Signer, Signature, signature::RandomizedSigner, SigningKey},
            //elliptic_curve::{Generate},
            SecretKey,
        };
        use  ecdsa::signer::Signer;
        use rand_core::OsRng; // requires 'getrandom' feature

        // Signing
        //let secret_key = SecretKey::generate(&mut OsRng);
        let secret_key = SigningKey::random(&mut OsRng); // Serialize with `::to_bytes()`
        //let signer = <dyn Signer<SigningKey>>::new(&secret_key).expect("secret key invalid");
        let signer = Signer::new(&secret_key).expect("secret key invalid");
        let message = b"ECDSA proves knowledge of a secret number in the context of a single message";

        // Note: the signature type must be annotated or otherwise inferrable as
        // `Signer` has many impls of the `RandomizedSigner` trait (for both
        // regular and recoverable signature types).
        let signature: Signature = signer.sign_with_rng(&mut OsRng, message);

        // Verification
        use p256::{PublicKey};
        use p256::ecdsa::{VerifyingKey, signature::Verifier}; // , ecdsa::{Verifier, signature::Verifier as _}

        let public_key = PublicKey::from_secret_key(&secret_key, true).expect("secret key invalid");
        let verifier = Verifier::new(&public_key).expect("public key invalid");

        assert!(verifier.verify(message, &signature).is_ok());
    }
*/
    {
        use rand_core::OsRng; // requires 'getrandom' feature
        //use secp256k1::rand::rngs::OsRng;
        use secp256k1::{Secp256k1, Message, SecretKey, PublicKey,
                        ecdsa::{Signature, SerializedSignature}};
        use secp256k1::hashes::sha256;
        use std::str::FromStr;

        let secp = Secp256k1::new();
        let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);
        // let ser_prive= Serialize::serialize(&secret_key).unwrap();
        // println!("secret_key: {}", ser_prive);
        let private_key_string: String = secret_key.display_secret().to_string();
        println!("private_key_string: {}", private_key_string);
        let pk2 = PublicKey::from_secret_key(&secp, &secret_key);
        let public_key_string: String = public_key.to_string();
        let public_key_string2: String = pk2.to_string();
        println!("public_key_string: {}", public_key_string);
        println!("public_key_string2: {}", public_key_string2);

        let message = Message::from_hashed_data::<sha256::Hash>("Hello World!".as_bytes());
        println!("message:::: {}", message);

        let sig: Signature = secp.sign_ecdsa(&message, &secret_key);
        let ser_sig = SerializedSignature::from_signature(&sig);
        let ser_vec = ser_sig.to_vec();
        println!("ser_vec: len({}) {:?}", ser_vec.len(), ser_vec);

        let signature_string: String = sig.to_string();
        println!("signature_string: {}", signature_string);

        let re_sig = Signature::from_str(&signature_string).unwrap();
        assert_eq!(re_sig, sig);
        assert!(secp.verify_ecdsa(&message, &sig, &public_key).is_ok());


        let re_secret_key = SecretKey::from_str(&private_key_string).unwrap();
        assert_eq!(re_secret_key,secret_key);

        let re_public_key = PublicKey::from_str(&public_key_string ).unwrap();
        assert_eq!(re_public_key, public_key);
    }


//     {
//         use rsa::{PublicKey, RsaPrivateKey, RsaPublicKey, PaddingScheme, rand_core};
//
//         let mut rng = rand::thread_rng();
//
//         let bits = 2048;
//         let private_key = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate a key");
//         let public_key = RsaPublicKey::from(&private_key);
//
// // Encrypt
//         let data = b"hello world";
//         let padding = PaddingScheme::new_oaep::<sha2::Sha256>();
//         let enc_data = public_key.encrypt(&mut rng, padding, &data[..]).expect("failed to encrypt");
//         assert_ne!(&data[..], &enc_data[..]);
//
// // Decrypt
//         let padding = PaddingScheme::new_oaep::<sha2::Sha256>();
//         let dec_data = private_key.decrypt(padding, &enc_data).expect("failed to decrypt");
//         assert_eq!(&data[..], &dec_data[..]);
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

    launch_threads().await;

    sleep(Duration::from_secs(5)).await;
}

