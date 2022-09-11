use std::collections::HashMap;
use substring::Substring;
use crate::{ccrypto, cutils};
use crate::lib::constants;
use crate::lib::dlog::dlog;
use crate::lib::transactions::basic_transactions::signature_structure_handler::general_structure::{create_complete_unlock_sets, validate_sig_struct};
use crate::lib::transactions::basic_transactions::signature_structure_handler::individual_signature::IndividualSignature;
use crate::lib::transactions::basic_transactions::signature_structure_handler::unlock_document::UnlockDocument;

//old_name_was createANewBasicAddress
pub fn create_a_new_basic_address<'a>(
    signature_mod: &str,
    signature_version: &'a str) -> (bool, UnlockDocument)
{
    let signature_type = constants::signature_types::BASIC; // by default is two of three (m of n === m/n)
    dlog(
        &format!("creating a new signature_type:signature_mod {}:{}", signature_type, signature_mod),
        constants::Modules::App,
        constants::SecLevel::Info);


    let mut map_pub_key_to_priv_key: HashMap<String, String> = HashMap::new();
    let signatures_dtl: Vec<String> = signature_mod.to_string().split("/").map(|x| x.to_string()).collect::<Vec<String>>();
    let m_signatures_count: u16 = signatures_dtl[0].parse::<u16>().unwrap();
    let n_signatures_count: u16 = signatures_dtl[1].parse::<u16>().unwrap();
    let mut individuals_signing_sets: HashMap<String, IndividualSignature> = HashMap::new();

    for i in 0..n_signatures_count
    {
        let (status, private_key, public_key) = ccrypto::ecdsa_generate_key_pair();
        if !status
        {
            dlog(
                &format!("Couldn't create Strict ECDSA key pair"),
                constants::Modules::App,
                constants::SecLevel::Fatal);

            return (false, UnlockDocument::new());
        }

        map_pub_key_to_priv_key.insert(public_key.clone(), private_key.clone());

        let mut a_sign_set: IndividualSignature = IndividualSignature {
            m_signer_id: "".to_string(),
            m_signature_key: public_key,
            m_permitted_to_pledge: constants::NO.to_string(),
            m_permitted_to_delegate: constants::NO.to_string(),
            m_input_time_lock: 0.0,
            m_input_time_lock_strickt: 0.0,
            m_output_time_lock: 0.0,
        };
        a_sign_set.m_signer_id = cutils::padding_length_value(i.to_string(), constants::LEN_PROP_LENGTH);
        individuals_signing_sets.insert(a_sign_set.m_signer_id.clone(), a_sign_set);
    }


    let options: HashMap<&str, &str> = HashMap::from([
        ("signature_type",
         signature_type, ),
        ("signature_version", signature_version),
        ("customSalt", "PURE_LEAVE"),
    ]);
    let mut unlock_info: UnlockDocument = create_complete_unlock_sets(
        individuals_signing_sets,
        m_signatures_count,
        &options);

    for an_unlocker_set in &unlock_info.m_unlock_sets
    {
        let mut private_keys: Vec<String> = vec![];
        for a_sign_set in &an_unlocker_set.m_signature_sets
        {
            let private_key = map_pub_key_to_priv_key.get(&a_sign_set.m_signature_key).unwrap().clone();
            private_keys.push(private_key);
        }
        unlock_info.m_private_keys.insert(an_unlocker_set.m_salt.clone(), private_keys);

        // test unlock structure&  signature
        let is_valid: bool = validate_sig_struct(
            &an_unlocker_set,
            &unlock_info.m_account_address,
            &HashMap::new(),
        );
        if is_valid {
            dlog(
                &format!("The new address {} created & tested successfully. {}",
                         cutils::short_bech16(&unlock_info.m_account_address.to_string()),
                         unlock_info.dump()
                ),
                constants::Modules::App,
                constants::SecLevel::Info);
        } else {
            dlog(
                &format!("Corrupted strict address created!?"),
                constants::Modules::App,
                constants::SecLevel::Fatal);


            panic!("Curropted strict address created!? {}", unlock_info.dump());
        }
    }

    //TODO: FIXME: implement key signature potential ASAP
    // validate signature of new address
    let message = ccrypto::convert_title_to_hash(
        &"Imagine all the people living life in peace"
            .to_string()
            .substring(0, constants::SIGN_MSG_LENGTH as usize)
            .to_string()
    );
    for an_unlock_set in &unlock_info.m_unlock_sets
    {
        for inx in 0..an_unlock_set.m_signature_sets.len()
        {
            let (status, signature_hex, _signature) = ccrypto::ecdsa_sign_message(
                &unlock_info.m_private_keys.get(&an_unlock_set.m_salt).unwrap()[inx].to_string(),
                &message);
            if !status
            {
                dlog(
                    &format!("Corrupted strict address created signature status!? {}", unlock_info.dump()),
                    constants::Modules::App,
                    constants::SecLevel::Fatal);

                panic!("Corrupted strict address created signature status!? {}", unlock_info.dump());
            }
            let verify_res = ccrypto::ecdsa_verify_signature(
                &an_unlock_set.m_signature_sets[inx].m_signature_key.to_string(),
                &message,
                &signature_hex);
            if !verify_res
            {
                dlog(
                    &format!("Curropted strict address created signature!? {}", unlock_info.dump()),
                    constants::Modules::App,
                    constants::SecLevel::Fatal);

                panic!("Corrupted strict address created signature!? {}", unlock_info.dump());
            }
        }
    }

    return (true, unlock_info);
}


