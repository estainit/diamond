use std::collections::HashMap;
use rand::Rng;
use serde_json::{json};
use serde::{Serialize, Deserialize};
use crate::lib::transactions::basic_transactions::signature_structure_handler::individual_signature::IndividualSignature;
use crate::lib::transactions::basic_transactions::signature_structure_handler::unlock_document::UnlockDocument;
use crate::lib::transactions::basic_transactions::signature_structure_handler::unlock_set::UnlockSet;
use crate::{application, ccrypto, constants, cutils, dlog, PermutationHandler};
use crate::cmerkle::{generate_m, get_root_by_a_prove};
use crate::lib::custom_types::{CAddressT, CCoinCodeT, CDocHashT, CMPAIValueT, COutputIndexT, JSonObject, VVString};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TInput
{
    pub m_transaction_hash: CDocHashT,
    // the reffered transaction hash
    pub m_output_index: COutputIndexT,

    pub m_owner: CAddressT,
    pub m_amount: CMPAIValueT,
    pub m_private_keys: Vec<String>,
    // they are need to sign the coin in order to spend it
    pub m_unlock_set: JSonObject,
}

impl TInput {
    //old_name_was getCoinCode
    #[allow(unused, dead_code)]
    pub fn get_coin_code(&self) -> CCoinCodeT
    {
        return cutils::pack_coin_code(&self.m_transaction_hash, self.m_output_index);
    }

    #[allow(unused, dead_code)]
    pub fn dump(&self) -> String
    {
        let mut out: String = "\nCoin Code: ".to_owned() + &self.m_transaction_hash + ":" + &self.m_output_index.to_string();
        out += &*("\nOwner: ".to_owned() + &self.m_owner.clone());
        out += &*("\nAmount: ".to_owned() + &self.m_amount.to_string());
        out += &*("\nUnlockset: ".to_owned() + &cutils::controlled_json_stringify(&self.m_unlock_set));
        //  out += "\nPrivate keys: " + m_private_keys.join(", ");
        return out;
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TOutput
{
    pub m_address: CAddressT,
    pub m_amount: CMPAIValueT,
    pub m_output_charachter: String,
    pub m_output_index: COutputIndexT,// = - 1;
}

impl TOutput {
    #[allow(unused, dead_code)]
    pub fn new() -> TOutput {
        TOutput {
            m_address: "".to_string(),
            m_amount: 0,
            m_output_charachter: constants::OUTPUT_NORMAL.to_string(),
            m_output_index: 0,
        }
    }

    #[allow(unused, dead_code)]
    pub fn dump(&self) -> String {
        let mut out: String = "\nTrx output: m_address".to_owned() + &self.m_address;
        out += &*("\nm_amount: ".to_owned() + &self.m_amount.to_string());
        out += &*("\nm_output_charachter: ".to_owned() + &self.m_output_charachter);
        out += &*("\nm_output_index: ".to_owned() + &self.m_output_index.to_string());
        //  out += "\nPrivate keys: " + m_private_keys.join(", ");
        return out.to_string();
    }
}

/*




*/

pub fn my_get<'a>(the_map: &'a HashMap<&str, &str>, the_key: &'a str, default_value: &'a str) -> &'a str {
    if the_map.contains_key(the_key) {
        return the_map.get(the_key).unwrap();
    }
    return default_value;
}

//old_name_was createCompleteUnlockSets
pub fn create_complete_unlock_sets<'a>(
    individuals_signing_sets: HashMap<String, IndividualSignature>,
    neccessary_signatures_count: u16,
    options: &'a HashMap<&str, &str>) -> UnlockDocument
{
    let signature_type: String = my_get(options, "signature_type", "").to_string();
    let signature_version: String = my_get(options, "signature_version", "").to_string();
    let custom_salt: String = my_get(options, "customSalt", "PURE_LEAVE").to_string();

    // let signers_ids: Vec<String> = vec![];

    // generate permutation of signatures. later will be used as tree leaves
    let mut leave_ids: Vec<String> = individuals_signing_sets
        .iter()
        .map(|(k, _v)| k.clone())
        .collect::<Vec<String>>();
    leave_ids.sort();
    let sign_permutations = PermutationHandler::new(
        &leave_ids,
        neccessary_signatures_count,
        true,
        &vec![],
        &vec![],
    );

    let mut custom_types: HashMap<String, HashMap<String, String>> = HashMap::new();
    let mut unlock_sets: Vec<UnlockSet> = vec![];
    let mut custom_salts: HashMap<String, String> = HashMap::new();
    for an_unlock_individuals_combination in sign_permutations.m_permutations
    {
        let mut a_signature_combination: Vec<IndividualSignature> = vec![];
        for an_individual_id in an_unlock_individuals_combination {
            let sign_set = individuals_signing_sets[&an_individual_id].clone();
            a_signature_combination.push(sign_set);
        }
        let custom_key = cutils::hash16c(
            &ccrypto::keccak256(
                &custom_stringify_signature_sets(&a_signature_combination)
            )
        );
        let mut an_unlock: UnlockSet = UnlockSet::new();
        an_unlock.m_signature_sets = a_signature_combination;
        unlock_sets.push(an_unlock);

        if (signature_type != "") && (signature_version != "")
        {
            let custom: HashMap<String, String> = HashMap::from([
                ("signature_type".to_string(), signature_type.to_string()),
                ("signature_version".to_string(), signature_version.to_string())
            ]);
            custom_types.insert(custom_key.clone(), custom);
        }

        if custom_salt != ""
        {
            if custom_salt == "PURE_LEAVE" {
                custom_salts.insert(custom_key.clone(), custom_key.clone());
            } else {
                custom_salts.insert(custom_key.clone(), custom_salt.clone());
            }
        }
    }

    return create_m_of_n_merkle(
        &mut unlock_sets,
        custom_types,
        custom_salts,
        options,
    );
}


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

    let (merkle_root, m_verifies, m_version, _levels, _leaves) = generate_m(
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
        m_merkle_version: m_version,
        m_private_keys: Default::default(),
    };

    // asign merkle proofs to sign_set itself
    for key in tmp_unlockers.keys()
    {
        let tmp: UnlockSet = UnlockSet {
            m_signature_type: tmp_unlockers[key].m_signature_type.clone(),
            m_signature_ver: tmp_unlockers[key].m_signature_ver.clone(),
            m_signature_sets: tmp_unlockers[key].m_signature_sets.clone(),
            m_merkle_proof: m_verifies.get(key).unwrap().m_merkle_proof.iter().map(|x| x.clone()).collect::<Vec<String>>(),
            m_left_hash: m_verifies.get(key).unwrap().m_left_hash.clone(),
            m_salt: tmp_unlockers[key].m_salt.clone(),
        };

        unlock_document.m_unlock_sets.push(tmp);
    }
    return unlock_document;
}

//old_name_was calcUnlockHash
pub fn calc_unlock_hash(unlock_set: &UnlockSet, hash_algorithm: &str) -> String
{
//  std::string potential_strings = R"(
//  Basic:0.0.0:[{"sKey":"0353444374647c47d52534fb9e0c1d7767d1a143ac5511c1ea45f98bc321732a0c"}]:a1ca0290308ed6a3
//  Basic:0.0.0:[{"sKey":"0353444374647c47d52534fb9e0c1d7768d1a143ac5511c1ee45f98bc321732a0c"},{"sKey":"0257040e1bf22ab8785c56d21615b140b766dee38ec8f1a7db49a6ebf98977a6bc"}]:b84100a46012d51a
//  Strict:0.0.0:[{"sKey":"022968b10e02e2af51a5965b9735ac2c75c51c71207f87bec0bd49fa61902f8619",
// "pPledge":"Y","pDelegate":"Y"},
// {"sKey":"0339129227adebcb49c89fdcbf036249b1e277727895b6803378a0364c33bc0b46","pPledge":"N","pDelegate":"N"},
//{"sKey":"03a797608e14ee87a93c0bf7d7d121593c5985030e9053e7d062bf081d59da956b","pPledge":"N","pDelegate":"N"},
//{"sKey":"03f4e4a46c160246e518c41c661b6eeae89aee2188e9dd454274bcca3414a2ed54","pPledge":"N","pDelegate":"N"},
//{"sKey":"03c146c6e887a1be14606d4a56a72905620064d30a0efa61bf99c1b4dcd10412ad","pPledge":"Y","pDelegate":"Y"},
//{"sKey":"02bcf7558f443691819af3d1ab6661f379efb8bbda9791f81156749f218ad2101a","pPledge":"N","pDelegate":"N"}]:a9de676dd54b672c
//  )";


    let to_be_hashed = unlock_set.m_signature_type.to_owned()
        + ":" + &unlock_set.m_signature_ver
        + ":" + &custom_stringify_signature_sets(&unlock_set.m_signature_sets)
        + ":" + &unlock_set.m_salt;//  hash_algorithm(${sType}:${sVer}:${JSON.stringify(sSet)}:${salt})
    dlog(
        &format!("Custom stringyfied unlock_struct: {}", to_be_hashed),
        constants::Modules::App,
        constants::SecLevel::TmpDebug);
    if hash_algorithm == "keccak256"
    {
        return ccrypto::keccak256(&to_be_hashed);
    }

    dlog(
        &format!("Invalid hash algorythm! {}", hash_algorithm),
        constants::Modules::App,
        constants::SecLevel::Fatal);
    panic!("Invalid hash algorythm! {}", hash_algorithm);
}

/*

UnlockSet convertJsonUSetToStruct(const JSonObject& unlockSet)
{
  QVector<IndividualSignature> sSets;
  for(QJsonValueRef an_s_set: unlockSet.value("sSets").toArray())
  {
    JSonObject an_s_setJ = an_s_set.toObject();
    IndividualSignature an_s_setDoc {
      an_s_setJ.value("sKey").to_string(),
      an_s_setJ.value("pPledge").to_string(),
      an_s_setJ.value("pDelegate").to_string()};

    if (an_s_setJ.keys().contains("iTLock"))
      an_s_setDoc.m_input_time_lock = an_s_setJ.value("iTLock").toDouble();

    if (an_s_setJ.keys().contains("oTLock"))
      an_s_setDoc.m_output_time_lock = an_s_setJ.value("oTLock").toDouble();

    if (an_s_setJ.keys().contains("iTLockSt"))
      an_s_setDoc.m_input_time_lock_strict = an_s_setJ.value("iTLockSt").toDouble();

    sSets.push(an_s_setDoc);
  }

  UnlockSet out {
    sSets,
    unlockSet.value("salt").to_string(),
    unlockSet.value("sType").to_string(),
    unlockSet.value("sVer").to_string(),
    unlockSet.value("lHash").to_string(),
    cutils::convertJSonArrayToStringVector(unlockSet.value("mProof").toArray())
  };

  return out;
}

String stringify_inputs(const JSonArray inputs)
{
  StringList inputs_list = {};
  for(auto an_input: inputs)
    inputs_list.push("[\"" + an_input[0].to_string() + "\"," + String::number(an_input[1].toDouble()) + "]");
  String inputs_string = "[" + inputs_list.join(",") + "]";
  return inputs_string;
}
*/
pub fn stringify_inputs(inputs: &Vec<TInput>) -> String
{
    let mut inputs_list: Vec<String> = vec![];
    for an_input in inputs
    {
        inputs_list.push("[\"".to_owned() + &an_input.m_transaction_hash + &"\"," + &*an_input.m_output_index.to_string() + "]");
    }
    let inputs_string: String = "[".to_owned() + &inputs_list.join(",") + "]";
    return inputs_string;
}

/*

String stringify_outputs(const JSonArray outputs)
{
  StringList outputs_list = {};
  for(auto an_input: outputs)
    outputs_list.push("[\"" + an_input[0].to_string() + "\"," + String::number(an_input[1].toDouble()) + "]");
  String outputs_string = "[" + outputs_list.join(",") + "]";
  return outputs_string;
}
*/

//old_name_was stringifyOutputs
pub fn stringify_outputs(outputs: &Vec<TOutput>) -> String
{
    let mut outputs_list: Vec<String> = vec![];
    for an_output in outputs
    {
        outputs_list.push("[\"".to_owned() + &an_output.m_address + "\"," + &an_output.m_amount.to_string() + "]");
    }
    let outputs_string: String = "[".to_owned() + &outputs_list.join(",") + "]";
    return outputs_string;
}

pub fn make_outputs_tuples(outputs: &Vec<TOutput>) -> VVString
{
    let mut outputs_list: VVString = vec![];
    for an_output in outputs
    {
        outputs_list.push(
            vec![an_output.m_address.clone(), an_output.m_amount.to_string()]
        );
    }
    return outputs_list;
}

/*

JSonObject compactUnlocker(const JSonObject& u_set)
{
  StringList optional_attributes_string = {"pPledge", "pDelegate"};
  StringList optional_attributes_int = {"iTLockSt", "iTLock", "oTLock"};

  JSonArray new_sign_sets {};

  for (auto a_sign_set_: u_set["sSets"].toArray())
  {
    JSonObject a_sign_set = a_sign_set_.toObject();
    StringList a_sign_set_keys = a_sign_set.keys();
    JSonObject a_new_sign_set {
      {"sKey", a_sign_set["sKey"]}
    };

    for (String a_key: optional_attributes_string)
      if (a_sign_set_keys.contains(a_key) && (a_sign_set[a_key] != ""))
        a_new_sign_set[a_key] = a_sign_set[a_key];

    for (String a_key: optional_attributes_int)
      if (a_sign_set_keys.contains(a_key) && (a_sign_set[a_key] != 0))
        a_new_sign_set[a_key] = a_sign_set[a_key];

    new_sign_sets.push(a_new_sign_set);
  }
  JSonObject new_u_set {
    {"lHash", u_set["lHash"]},
    {"mProof", u_set["mProof"]},
    {"sSets", new_sign_sets},
    {"sType", u_set["sType"]},
    {"sVer", u_set["sVer"]},
    {"salt", u_set["salt"]}};

  return new_u_set;
}

*/

//old_name_was compactUnlockersArray
pub fn compact_unlockers_array(doc_ext_info: &JSonObject) -> JSonObject
{
    let new_doc_ext_info: JSonObject = json!({});
    for an_ext in doc_ext_info.as_array().unwrap()
    {
        println!("compacting an_ext: {:?}", an_ext);
        /*
        JSonObject unlock_doc = an_ext.toObject();
        JSonObject new_unlock_doc {};
        for (String a_key: unlock_doc.keys())
        {
          if (a_key == "uSet")
          {
            JSonObject u_set = unlock_doc["uSet"].toObject();
            JSonObject new_u_set = compactUnlocker(u_set);
            new_unlock_doc[a_key] = new_u_set;
          }else{
            new_unlock_doc[a_key] = unlock_doc[a_key];
          }
        }
        new_doc_ext_info.push(new_unlock_doc);
         */
    }

    return new_doc_ext_info;
}

/*
String safeStringifySigntureSets(const JSonArray& signture_sets)
{
  StringList sets_str;
  for(QJsonValue an_s_set: signture_sets)
  {
    JSonObject an_s_setJ = an_s_set.toObject();

    String a_set = "{";
    a_set += "\"sKey\":\"" + an_s_setJ.value("sKey").to_string() + "\"";

    if (an_s_setJ.keys().contains("iTLock") && (an_s_setJ["iTLock"].toDouble() > 0))
      a_set += ",\"iTLock\":" + String::number(an_s_setJ.value("iTLock").toDouble());

    if (an_s_setJ.keys().contains("iTLockSt") && (an_s_setJ["iTLockSt"].toDouble() > 0))
      a_set += ",\"iTLockSt\":" + String::number(an_s_setJ.value("iTLockSt").toDouble()) ;

    if (an_s_setJ.keys().contains("oTLock") && (an_s_setJ["oTLock"].toDouble() > 0))
      a_set += ",\"oTLock\":" + String::number(an_s_setJ.value("oTLock").toDouble());

    if(an_s_setJ.value("pDelegate").to_string() != "")
      a_set += ",\"pDelegate\":\"" + an_s_setJ.value("pDelegate").to_string() + "\"";

    if (an_s_setJ.value("pPledge").to_string() != "")
      a_set += ",\"pPledge\":\"" + an_s_setJ.value("pPledge").to_string() + "\"";

    a_set += "}";

    sets_str.push(a_set);
  }
  String out = "[" + sets_str.join(",") + "]";
  return out;
}

String safeStringifyUnlockSet(const JSonObject& unlockSet)
{
  String out = "{";
  if (unlockSet.value("lHash").to_string() == "")
  {
    out += "\"lHash\":\"\",";
  }else{
    out += "\"lHash\":\"" + unlockSet.value("lHash").to_string() + "\",";
  }
  if (unlockSet.value("mProof").toArray().len() > 0)
  {
    out += "\"mProof\":" + cutils::serializeJson(unlockSet.value("mProof").toArray()) + ",";
  }else{
    out += "\"mProof\":[],";
  }
  out += "\"salt\":\"" + unlockSet.value("salt").to_string() + "\",";
  out += "\"sSets\":" + safeStringifySigntureSets(unlockSet.value("sSets").toArray()) + ",";
  out += "\"sType\":\"" + unlockSet.value("sType").to_string() + "\",";
  out += "\"sVer\":\"" + unlockSet.value("sVer").to_string() + "\"";
  out += "}";
  return out;
}

*/

//old_name_was validateSigStruct
pub fn validate_sig_struct(
    unlock_set: &UnlockSet,
    address: &str,
    options: &HashMap<&str, &str>) -> bool
{
    dlog(
        &format!("validate Sig Struct: {}", unlock_set.dump()),
        constants::Modules::App,
        constants::SecLevel::Debug);

    let hash_algorithm: String = my_get(options, "hash_algorithm", "keccak256").to_string();
    let input_type: String = my_get(options, "input_type", "hashed").to_string();
    let do_permutation: String = my_get(options, "do_permutation", constants::NO).to_string();

    // console.log(validate StructureOfAnUnlockMOfN.args: ${utils.stringify(args)});
    if unlock_set.m_signature_type == constants::signature_types::STRICT {
        if !validate_structure_restrictions(unlock_set, &options)
        {
            dlog(
                &format!("Invalid strict address! {}", unlock_set.dump()),
                constants::Modules::App,
                constants::SecLevel::Error);
            return false;
        }
    }

    // normally the wallets SHOULD send the saved order of a sSets, so we do not need to Permutation
    let leave_hash = calc_unlock_hash(&unlock_set, "keccak256");

    let merkle_proof = &unlock_set.m_merkle_proof.iter().map(|x| x.to_string()).collect::<Vec<String>>();
    let mut merkle_root = get_root_by_a_prove(
        &leave_hash,
        merkle_proof,
        &unlock_set.m_left_hash.to_string(),
        &input_type,
        &hash_algorithm);
    merkle_root = ccrypto::keccak256_dbl(&merkle_root);  // because of securiy, MUST use double hash

    if (vec![constants::HU_SHARE_ADDRESS, constants::HU_INAME_OWNER_ADDRESS].contains(&address)) &&
        (unlock_set.m_signature_type == constants::signature_types::MIX23) {
        merkle_root = ccrypto::sha256_dbl(&merkle_root);  // Mixed extra securiy level
    }

    let bech32 = ccrypto::bech32_encode(&merkle_root);
    if address == bech32
    {
        return true;
    }


    if do_permutation == constants::NO
    {
        dlog(
            &format!("Invalid unlock structure! {}", unlock_set.dump()),
            constants::Modules::App,
            constants::SecLevel::Error);
        return false;
    }

    // FIXME: implement it ASAP
    // in case of disordinating inside sSets
//     let hp = new utils.heapPermutation();
//     hp.heapP(unlock_set.sSets)
//     for (let premu of hp.premutions) {
//         leave_hash = hash_algorithm(${unlock_set.sType}:${unlock_set.sVer}:${JSON.stringify(premu)}:${unlock_set.salt});
//         merkle_root = crypto.merkleGetRootByAProve(leave_hash, unlock_set.proofs, unlock_set.lHash, input_type, hash_algorithm);
//         bech32 = crypto.bech32_encodePub(crypto.keccak256_dbl(merkle_root)).encoded;  // because of securiy, MUST use double hash
//         if (_.has(unlock_set, 'address')) {
//             if (unlock_set.address == bech32) {
//                 return true;
//             }
//         } else if (_.has(unlock_set, 'merkle_root')) {
//             if (unlock_set.merkle_root == merkle_root) {
//                 return true;
//             }
//         }
//     };
//     return false;

    dlog(
        &format!("Invalid unlock structure, even after premutation1! {}", unlock_set.dump()),
        constants::Modules::App,
        constants::SecLevel::Error);

    return false;
}

pub fn custom_stringify_signature_sets(signature_sets: &Vec<IndividualSignature>) -> String
{
    let mut s_sets_serial: Vec<String> = vec![];
    for a_sig in signature_sets
    {
        let mut tmp = "{\"sKey\":\"".to_owned() + &*a_sig.m_signature_key + "\"";

        if a_sig.m_permitted_to_pledge != ""
        {
            tmp += &(",\"pPledge\":\"".to_owned() + &*a_sig.m_permitted_to_pledge + "\"");
        }

        if a_sig.m_permitted_to_delegate != ""
        {
            tmp += &(",\"pDelegate\":\"".to_owned() + &*a_sig.m_permitted_to_delegate + "\"");
        }

        tmp += "}";

        s_sets_serial.push(tmp);
    }
    let custom_stringify = "[".to_owned() + &s_sets_serial.join(",") + "]";  //  JSON.stringify(sSet)
    return custom_stringify;
}

//old_name_was validateStructureStrictions
pub fn validate_structure_restrictions(
    unlock_set: &UnlockSet,
    _options: &HashMap<&str, &str>) -> bool
{
    // console.log(validate StructureStrictions.args: ${utils.stringify(args)});
    // let hash_algorithm = my_get(&options, "hash_algorithm", "keccak256").to_string();

    if unlock_set.m_signature_type == constants::signature_types::STRICT
    {
        //  * this strict type of signature MUST have and ONLY have these 3 features
        //  * sKey: can be a public key(and later can be also another bech32 address, after implementing nested signature feature)
        //  * pPledge: means the signer Permitted to Pleadge this account
        //  * pDelegate: means the signer Permited to Delegate some rights (binded to this address) to others

        if cutils::hash16c(&ccrypto::keccak256(&custom_stringify_signature_sets(&unlock_set.m_signature_sets))) != unlock_set.m_salt
        {
            dlog(
                &format!("invalid strict structure of signature of salt({}) ", unlock_set.m_salt),
                constants::Modules::App,
                constants::SecLevel::Info);
            return false;
        }

        for a_sign_set in &unlock_set.m_signature_sets
        {
            if (a_sign_set.m_signature_key == "") || (a_sign_set.m_permitted_to_pledge == "") ||
                (a_sign_set.m_permitted_to_delegate == "")

            {
                dlog(
                    &format!("invalid strict structure of signature: {}", unlock_set.dump()),
                    constants::Modules::App,
                    constants::SecLevel::Info);

                return false;
            }
        }

        return true;
    }

    return true;
}
