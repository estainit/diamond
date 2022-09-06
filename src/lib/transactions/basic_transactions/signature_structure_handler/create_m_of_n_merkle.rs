use std::collections::HashMap;
use rand::Rng;
use crate::{application, ccrypto, constants, cutils};
use crate::cmerkle::generate_m;
use crate::lib::custom_types::VString;
use crate::lib::transactions::basic_transactions::signature_structure_handler::general_structure::{calc_unlock_hash, custom_stringify_signature_sets, my_get};
use crate::lib::transactions::basic_transactions::signature_structure_handler::unlock_document::UnlockDocument;
use crate::lib::transactions::basic_transactions::signature_structure_handler::unlock_set::UnlockSet;

//old_name_was createMOfNMerkle
pub fn create_m_of_n_merkle<'a>(
    unlock_sets: &mut Vec<UnlockSet>,
    custom_types: HashMap<String, HashMap<String, String>>,
    custom_salts: HashMap<String, String>,
    options: &'a HashMap<&str, &str>) -> UnlockDocument
{
//  CLog::log("create M Of N Merkle creating unlock_sets" + cutils::dumpIt(unlock_sets), "app", "trace");

    let hash_algorithm: String = my_get(options, "hash_algorithm", "keccak256").to_string();
    let input_type: String = my_get(options, "input_type", "hashed").to_string();

    let mut hashed_unlocks: Vec<String> = vec![];
    let mut tmp_unlockers: HashMap<String, &UnlockSet> = HashMap::new();
    let mut custom_key: String;
    let mut leave_hash: String;
    for mut an_unlock_set in unlock_sets
    {
        custom_key = cutils::hash16c(&ccrypto::keccak256(&custom_stringify_signature_sets(&an_unlock_set.m_signature_sets)));
        if custom_types.contains_key(&custom_key)
        {
            an_unlock_set.m_signature_type = custom_types[&custom_key]["signature_type"].clone();
            an_unlock_set.m_signature_ver = custom_types[&custom_key]["signature_version"].clone();
        } else {
            an_unlock_set.m_signature_type = constants::signature_types::BASIC.to_string();
            an_unlock_set.m_signature_ver = "0.0.0".to_string();
        }

        // adding random salt to obsecure/unpredictable final address
        // TODO: maybe use crypto secure random generator
        if custom_salts.contains_key(&custom_key) {
            an_unlock_set.m_salt = custom_salts[&custom_key].clone();
        } else {
            an_unlock_set.m_salt = cutils::hash16c(
                &ccrypto::keccak256(
                    &(an_unlock_set.dump()
                        + &application().now()
                        + &format!("{}", rand::thread_rng().gen::<u32>())
                    )
                )
            );
        }
        leave_hash = calc_unlock_hash(an_unlock_set, &hash_algorithm);
        hashed_unlocks.push(leave_hash.clone());
        tmp_unlockers.insert(leave_hash, an_unlock_set);
    }

    let (merkle_root,
        the_verifies,
        the_version,
        _levels, _leaves) = generate_m(
        hashed_unlocks,
        &input_type,
        &hash_algorithm,
        &"".to_string());
    let mut merkle_root = ccrypto::keccak256_dbl(&merkle_root); // because of securiy, MUST use double hash

    // FIXME: m_signature_type is aplyable for each uSet in a m of n shema, wherase for merkle root we can apply only one signature_type.
    // for now we just use the signature_type of fist uSet.
    // BTW for now all signature_types in an address are same
    let first_unlocker: String = tmp_unlockers.keys().map(|k| k.clone()).collect::<Vec<String>>()[0].clone();
    if tmp_unlockers.get(&first_unlocker).unwrap().m_signature_type == constants::signature_types::MIX23
    {
        merkle_root = ccrypto::sha256_dbl(&merkle_root);  // Extra securiy level
    }

    let mut unlock_document: UnlockDocument = UnlockDocument {
        m_unlock_sets: vec![],
        m_merkle_root: merkle_root.clone(),
        m_account_address: ccrypto::bech32_encode(&merkle_root),
        m_merkle_version: the_version,
        m_private_keys: Default::default(),
    };

    // assign merkle proofs to sign_set itself
    let the_keys = tmp_unlockers
        .keys()
        .cloned()
        .collect::<VString>();
    for key in &the_keys
    {
        let merkle_proof: VString;
        let left_hash: String;

        if the_keys.len() == 1
        {
            merkle_proof = vec![];
            left_hash = "".to_string();
        } else {
            merkle_proof = the_verifies
                .get(key)
                .unwrap()
                .m_merkle_proof
                .iter()
                .map(|x| x.clone())
                .collect::<VString>();
            left_hash = the_verifies.get(key).unwrap().m_left_hash.clone();
        }
        let tmp: UnlockSet = UnlockSet {
            m_signature_type: tmp_unlockers[key].m_signature_type.clone(),
            m_signature_ver: tmp_unlockers[key].m_signature_ver.clone(),
            m_signature_sets: tmp_unlockers[key].m_signature_sets.clone(),
            m_merkle_proof: merkle_proof,
            m_left_hash: left_hash,
            m_salt: tmp_unlockers[key].m_salt.clone(),
        };

        unlock_document.m_unlock_sets.push(tmp);
    }
    return unlock_document;
}