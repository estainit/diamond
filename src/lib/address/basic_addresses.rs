use std::collections::HashMap;
use substring::Substring;
use warp::body::form;
use crate::{ccrypto, cutils};
use crate::lib::constants;
use crate::lib::dlog::dlog;
use crate::lib::transactions::basic_transactions::signature_structure_handler::general_structure::{createCompleteUnlockSets, validateSigStruct};
use crate::lib::transactions::basic_transactions::signature_structure_handler::individual_signature::IndividualSignature;
use crate::lib::transactions::basic_transactions::signature_structure_handler::unlock_document::UnlockDocument;

pub fn createANewBasicAddress<'a>(
    signature_mod: &str,
    signature_version: &'a str) -> (bool, UnlockDocument<'a>)
{
    let signature_type = constants::signature_types::Basic; // by default is two of three (m of n === m/n)
    dlog(
        &format!("creating a new signature_type:signature_mod {}:{}", signature_type, signature_mod),
        constants::Modules::App,
        constants::SecLevel::Info);


    let mut map_pub_key_to_priv_key: &HashMap<&str, &str> = &HashMap::new();
    let signatures_dtl: Vec<String> = signature_mod.to_string().split("/").map(|x| x.to_string()).collect::<Vec<String>>();
    let m_signatures_count: u16 = signatures_dtl[0].parse::<u16>().unwrap();
    let n_signatures_count: u16 = signatures_dtl[1].parse::<u16>().unwrap();
    let mut individuals_signing_sets: &HashMap<&str, &IndividualSignature> = &HashMap::new();

    for i in 0..n_signatures_count
    {
        let (status, private_key, public_key) = ccrypto::ecdsa_generate_key_pair();
        if (!status)
        {
            dlog(
                &format!("Couldn't create Strict ECDSA key pair"),
                constants::Modules::App,
                constants::SecLevel::Fatal);

            return (false, UnlockDocument::new());
        }

        map_pub_key_to_priv_key.insert(&public_key[..], &private_key[..]);

        let mut a_sign_set: IndividualSignature = IndividualSignature {
            m_signer_id: "",
            m_signature_key: &public_key[..],
            m_permitted_to_pledge: constants::NO,
            m_permitted_to_delegate: constants::NO,
            m_input_time_lock: 0,
            m_output_time_lock: 0,
        };
        a_sign_set.m_signer_id = &*cutils::padding_length_value(format!("{}", i), 7);
        individuals_signing_sets.insert(a_sign_set.m_signer_id, &a_sign_set);
    }


    let options: HashMap<&str, &str> = HashMap::from([
        ("signature_type",
         signature_type, ),
        ("signature_version", signature_version),
        ("customSalt", "PURE_LEAVE"),
    ]);
    let mut unlock_info: UnlockDocument = createCompleteUnlockSets(
        individuals_signing_sets,
        m_signatures_count,
        &options);

    for &an_unlocker_set in unlock_info.m_unlock_sets
    {
        let mut private_keys: Vec<&str> = vec![];
        for &aSignSet in an_unlocker_set.m_signature_sets
        {
            private_keys.push(&map_pub_key_to_priv_key.get(aSignSet.m_signature_key).unwrap().to_string());
        }
        unlock_info.m_private_keys.insert(an_unlocker_set.m_salt, private_keys);

        // test unlock structure&  signature
        let is_valid: bool = validateSigStruct(
            an_unlocker_set,
            unlock_info.m_account_address,
            &HashMap::new()
        );
        if (is_valid) {
            dlog(
                &format!("The new address {} created & tested successfully", cutils::shortBech16(&unlock_info.m_account_address.to_string())),
                constants::Modules::App,
                constants::SecLevel::Info);
        } else {
            dlog(
                &format!("Curropted strict address created!?"),
                constants::Modules::App,
                constants::SecLevel::Fatal);


            panic!("Curropted strict address created!? {}", unlock_info.dump());
        }
    }

    // console.log(\n unlock_info: ${utils.stringify(unlock_info)}\n);

    //TODO: FIXME: implement key signature potential ASAP
    // validate signature of new address
    let message = ccrypto::convert_title_to_hash(
        &"Imagine all the people living life in peace"
            .to_string()
            .substring(0, constants::SIGN_MSG_LENGTH as usize)
            .to_string()
    );
    for &an_unlock_set in unlock_info.m_unlock_sets
    {
        for inx in 0..an_unlock_set.m_signature_sets.len()
        {
            let salk_key = an_unlock_set.m_salt;
            let (status, signature_hex, signature) = ccrypto::ecdsa_sign_message(
                &unlock_info.m_private_keys.get(salk_key).unwrap()[inx].to_string(),
                &message);
            if !status
            {
                dlog(
                    &format!("Curropted strict address created signature status!? {}", unlock_info.dump()),
                    constants::Modules::App,
                    constants::SecLevel::Fatal);

                panic!("Curropted strict address created signature status!? {}", unlock_info.dump());
            }
            let verifyRes = ccrypto::ecdsa_verify_signature(
                &an_unlock_set.m_signature_sets[inx].m_signature_key.to_string(),
                &message,
                &signature_hex);
            if !verifyRes
            {
                dlog(
                    &format!("Curropted strict address created signature!? {}", unlock_info.dump()),
                    constants::Modules::App,
                    constants::SecLevel::Fatal);

                panic!("Curropted strict address created signature!? {}", unlock_info.dump());
            }
        }
    }

    return (true, unlock_info);
}


