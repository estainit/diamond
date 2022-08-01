/*

QString TInput::dumpMe() const
{
  QString out = "\nCoin Code: " + m_transaction_hash + ":" + QString::number(m_output_index);
  out += "\nOwner: " + m_owner;
  out += "\nAmount: " + QString::number(m_amount);
  out += "\nUnlockset: " + cutils::serializeJson(m_unlock_set);
//  out += "\nPrivate keys: " + m_private_keys.join(", ");
  return out;
}

CCoinCodeT TInput::getCoinCode()
{
  return cutils::packCoinCode(m_transaction_hash, m_output_index);
}


*/

use std::collections::HashMap;
use rand::Rng;
use serde_json::Value;
use crate::lib::transactions::basic_transactions::signature_structure_handler::individual_signature::IndividualSignature;
use crate::lib::transactions::basic_transactions::signature_structure_handler::unlock_document::UnlockDocument;
use crate::lib::transactions::basic_transactions::signature_structure_handler::unlock_set::UnlockSet;
use crate::{ccrypto, constants, cutils, dlog, PermutationHandler};
use crate::cmerkle::{generate_m, get_root_by_a_prove};

pub fn my_get<'a>(the_map: &'a HashMap<&str, &str>, the_key: &'a str, default_value: &'a str) -> &'a str {
    if the_map.contains_key(the_key) {
        return the_map.get(the_key).unwrap();
    }
    return default_value;
}

pub fn createCompleteUnlockSets<'a>(
    individuals_signing_sets: &'a HashMap<&str, &IndividualSignature>,
    neccessary_signatures_count: u16,
    options: &'a HashMap<&str, &str>) -> UnlockDocument<'a>
{
    let signature_type: String = my_get(options, "signature_type", "").to_string();
    let signature_version: String = my_get(options, "signature_version", "").to_string();
    let customSalt: String = my_get(options, "customSalt", "PURE_LEAVE").to_string();

    let mut signers_ids: Vec<String> = vec![];

    // generate permutation of signatures. later will be used as tree leaves
    let mut leave_ids: Vec<String> = individuals_signing_sets
        .iter()
        .map(|(&k, &v)| k.to_string())
        .collect::<Vec<String>>();
    leave_ids.sort();
    let signPermutations = PermutationHandler::new(
        &leave_ids,
        neccessary_signatures_count,
        true,
        &vec![],
        &vec![],
    );

    let mut custom_types: HashMap<&str, HashMap<&str, &str>> = HashMap::new();
    let mut unlock_sets: Vec<UnlockSet> = vec![];
    let mut custom_salts: HashMap<&str, &str> = HashMap::new();
    for an_unlock_individuals_combination in signPermutations.m_permutations
    {
        let mut a_signature_combination: Vec<&IndividualSignature> = vec![];
        for an_individual_id in an_unlock_individuals_combination {
            let sign_set = *individuals_signing_sets.get(&*an_individual_id).unwrap();
            a_signature_combination.push(sign_set);
        }
        let mut an_unlock: UnlockSet = UnlockSet::new();
        an_unlock.m_signature_sets = &a_signature_combination;
        unlock_sets.push(an_unlock);

        let custom_key = cutils::hash16c(&ccrypto::keccak256(&customStringifySignatureSets(&a_signature_combination)));
        if (signature_type != "") && (signature_version != "")
        {
            let custom: HashMap<&str, &str> = HashMap::from([
                ("signature_type", &*signature_type),
                ("signature_version", &*signature_version)
            ]);
            custom_types.insert(&custom_key, custom);
        }

        if customSalt != ""
        {
            if customSalt == "PURE_LEAVE" {
                custom_salts.insert(&custom_key, &custom_key);
            } else {
                custom_salts.insert(&custom_key, &customSalt.clone());
            }
        }
    }

    return createMOfNMerkle(
        &mut unlock_sets,
        custom_types,
        custom_salts,
        options,
    );
}


pub fn createMOfNMerkle<'a>(
    unlock_sets: &mut Vec<UnlockSet<'a>>,
    custom_types: HashMap<&str, HashMap<&str, &str>>,
    custom_salts: HashMap<&str, &str>,
    options: &'a HashMap<&str, &str>) -> UnlockDocument<'a>
{
//  CLog::log("createMOfNMerkle creating unlock_sets" + cutils::dumpIt(unlock_sets), "app", "trace");

    let hash_algorithm: String = my_get(options, "hash_algorithm", "keccak256").to_string();
    let input_type: String = my_get(options, "input_type", "hashed").to_string();

    let mut hashed_unlocks: Vec<&str> = vec![];
    let mut tmp_unlockers: HashMap<&str, &UnlockSet> = HashMap::new();
    let mut custom_key: &str = "";
    let mut leave_hash: &str = "";
    for mut an_unlock_set in unlock_sets
    {
        custom_key = &cutils::hash16c(&ccrypto::keccak256(&customStringifySignatureSets(an_unlock_set.m_signature_sets)));
        if custom_types.contains_key(custom_key)
        {
            an_unlock_set.m_signature_type = &custom_types[custom_key]["signature_type"].to_string();
            an_unlock_set.m_signature_ver = &custom_types[custom_key]["signature_version"].to_string();
        } else {
            an_unlock_set.m_signature_type = &constants::signature_types::Basic;
            an_unlock_set.m_signature_ver = &"0.0.0";
        }

        // adding random salt to obsecure/unpredictable final address
        // TODO: maybe use crypto secure random generator
        if (custom_salts.contains_key(custom_key))
        {
            an_unlock_set.m_salt = &custom_salts[custom_key].to_string();
        } else {
            let tt = &cutils::hash16c(
                &ccrypto::keccak256(&(an_unlock_set.dump() + &cutils::get_now() + &format!("{}", rand::thread_rng().gen::<u32>()))
                )
            );
            an_unlock_set.m_salt = &&tt[..];
        }
        leave_hash = &calcUnlockHash(an_unlock_set, &hash_algorithm);
        hashed_unlocks.push(leave_hash);
        tmp_unlockers[leave_hash] = an_unlock_set;
    }

    let (merkle_root, mVerifies, mVersion, _levels, _leaves) = generate_m(
        &hashed_unlocks,
        &input_type,
        &hash_algorithm,
        &"".to_string());
    let mut merkle_root = ccrypto::keccak256_dbl(&merkle_root); // because of securiy, MUST use double hash

    // FIXME: m_signature_type is aplyable for each uSet in a m of n shema, wherase for merkle root we can apply only one signature_type.
    // for now we just use the signature_type of fist uSet.
    // BTW for now all signature_types in an address are same
    let first_unlocker: &str = tmp_unlockers.keys().map(|&k| k).collect::<Vec<&str>>()[0];
    if tmp_unlockers.get(first_unlocker).unwrap().m_signature_type == constants::signature_types::Mix23
    {
        merkle_root = ccrypto::sha256_dbl(&merkle_root);  // Extra securiy level
    }

    let mut unlock_document: UnlockDocument = UnlockDocument {
        m_unlock_sets: &vec![],
        m_merkle_root: &merkle_root,
        m_account_address: &ccrypto::bech32_encode(&merkle_root),
        m_merkle_version: &*mVersion,
        m_private_keys: Default::default(),
    };

    // asign merkle proofs to sign_set itself
    for &key in tmp_unlockers.keys()
    {
        let tmp: UnlockSet = UnlockSet {
            m_signature_type: tmp_unlockers[key].m_signature_type,
            m_signature_ver: tmp_unlockers[key].m_signature_ver,
            m_signature_sets: tmp_unlockers[key].m_signature_sets,
            m_merkle_proof: &mVerifies.get(key).unwrap().m_merkle_proof.iter().map(|&x| &*x).collect::<Vec<&str>>(),
            m_left_hash: &mVerifies.get(key).unwrap().m_left_hash,
            m_salt: tmp_unlockers[key].m_salt,
        };

        unlock_document.m_unlock_sets.push(&tmp);
    }
    return unlock_document;
}


pub fn calcUnlockHash(unlock_set: &UnlockSet, hash_algorithm: &str) -> String
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


    let mut to_be_hashed = unlock_set.m_signature_type.to_owned()
        + ":" + unlock_set.m_signature_ver
        + ":" + &customStringifySignatureSets(unlock_set.m_signature_sets)
        + ":" + unlock_set.m_salt;//  hash_algorithm(${sType}:${sVer}:${JSON.stringify(sSet)}:${salt})
    dlog(
        &format!("Custom stringyfied unlock_struct: {}", to_be_hashed),
        constants::Modules::App,
        constants::SecLevel::Trace);
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

UnlockSet convertJsonUSetToStruct(const QJsonObject& unlockSet)
{
  QVector<IndividualSignature> sSets;
  for(QJsonValueRef an_s_set: unlockSet.value("sSets").toArray())
  {
    QJsonObject an_s_setJ = an_s_set.toObject();
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
    cutils::convertJSonArrayToQStringList(unlockSet.value("mProof").toArray())
  };

  return out;
}

QString stringifyInputs(const QJsonArray inputs)
{
  QStringList inputs_list = {};
  for(auto an_input: inputs)
    inputs_list.append("[\"" + an_input[0].to_string() + "\"," + QString::number(an_input[1].toDouble()) + "]");
  QString inputs_string = "[" + inputs_list.join(",") + "]";
  return inputs_string;
}

QString stringifyInputs(const std::vector<TInput*> inputs)
{
  QStringList inputs_list = {};
  for(TInput* an_input: inputs)
    inputs_list.append("[\"" + an_input->m_transaction_hash+ "\"," + QString::number(an_input->m_output_index) + "]");
  QString inputs_string = "[" + inputs_list.join(",") + "]";
  return inputs_string;
}

QString stringifyOutputs(const QJsonArray outputs)
{
  QStringList outputs_list = {};
  for(auto an_input: outputs)
    outputs_list.append("[\"" + an_input[0].to_string() + "\"," + QString::number(an_input[1].toDouble()) + "]");
  QString outputs_string = "[" + outputs_list.join(",") + "]";
  return outputs_string;
}

QString stringifyOutputs(const std::vector<TOutput*> outputs)
{
  QStringList outputs_list = {};
  for(TOutput* an_output: outputs)
    outputs_list.append("[\"" + an_output->m_address + "\"," + QString::number(an_output->m_amount) + "]");
  QString outputs_string = "[" + outputs_list.join(",") + "]";
  return outputs_string;
}

QJsonObject compactUnlocker(const QJsonObject& u_set)
{
  QStringList optional_attributes_string = {"pPledge", "pDelegate"};
  QStringList optional_attributes_int = {"iTLockSt", "iTLock", "oTLock"};

  QJsonArray new_sign_sets {};

  for (auto a_sign_set_: u_set["sSets"].toArray())
  {
    QJsonObject a_sign_set = a_sign_set_.toObject();
    QStringList a_sign_set_keys = a_sign_set.keys();
    QJsonObject a_new_sign_set {
      {"sKey", a_sign_set["sKey"]}
    };

    for (QString a_key: optional_attributes_string)
      if (a_sign_set_keys.contains(a_key) && (a_sign_set[a_key] != ""))
        a_new_sign_set[a_key] = a_sign_set[a_key];

    for (QString a_key: optional_attributes_int)
      if (a_sign_set_keys.contains(a_key) && (a_sign_set[a_key] != 0))
        a_new_sign_set[a_key] = a_sign_set[a_key];

    new_sign_sets.push(a_new_sign_set);
  }
  QJsonObject new_u_set {
    {"lHash", u_set["lHash"]},
    {"mProof", u_set["mProof"]},
    {"sSets", new_sign_sets},
    {"sType", u_set["sType"]},
    {"sVer", u_set["sVer"]},
    {"salt", u_set["salt"]}};

  return new_u_set;
}

QJsonArray compactUnlockersArray(const QJsonArray& dExtInfo)
{
  QJsonArray new_doc_ext_info {};
  for (auto an_ext: dExtInfo)
  {
    QJsonObject unlock_doc = an_ext.toObject();
    QJsonObject new_unlock_doc {};
    for (QString a_key: unlock_doc.keys())
    {
      if (a_key == "uSet")
      {
        QJsonObject u_set = unlock_doc["uSet"].toObject();
        QJsonObject new_u_set = compactUnlocker(u_set);
        new_unlock_doc[a_key] = new_u_set;
      }else{
        new_unlock_doc[a_key] = unlock_doc[a_key];
      }
    }
    new_doc_ext_info.push(new_unlock_doc);
  }
  return new_doc_ext_info;
}

QString safeStringifySigntureSets(const QJsonArray& signture_sets)
{
  QStringList sets_str;
  for(QJsonValue an_s_set: signture_sets)
  {
    QJsonObject an_s_setJ = an_s_set.toObject();

    QString a_set = "{";
    a_set += "\"sKey\":\"" + an_s_setJ.value("sKey").to_string() + "\"";

    if (an_s_setJ.keys().contains("iTLock") && (an_s_setJ["iTLock"].toDouble() > 0))
      a_set += ",\"iTLock\":" + QString::number(an_s_setJ.value("iTLock").toDouble());

    if (an_s_setJ.keys().contains("iTLockSt") && (an_s_setJ["iTLockSt"].toDouble() > 0))
      a_set += ",\"iTLockSt\":" + QString::number(an_s_setJ.value("iTLockSt").toDouble()) ;

    if (an_s_setJ.keys().contains("oTLock") && (an_s_setJ["oTLock"].toDouble() > 0))
      a_set += ",\"oTLock\":" + QString::number(an_s_setJ.value("oTLock").toDouble());

    if(an_s_setJ.value("pDelegate").to_string() != "")
      a_set += ",\"pDelegate\":\"" + an_s_setJ.value("pDelegate").to_string() + "\"";

    if (an_s_setJ.value("pPledge").to_string() != "")
      a_set += ",\"pPledge\":\"" + an_s_setJ.value("pPledge").to_string() + "\"";

    a_set += "}";

    sets_str.append(a_set);
  }
  QString out = "[" + sets_str.join(",") + "]";
  return out;
}

QString safeStringifyUnlockSet(const QJsonObject& unlockSet)
{
  QString out = "{";
  if (unlockSet.value("lHash").to_string() == "")
  {
    out += "\"lHash\":\"\",";
  }else{
    out += "\"lHash\":\"" + unlockSet.value("lHash").to_string() + "\",";
  }
  if (unlockSet.value("mProof").toArray().size() > 0)
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
pub fn validateSigStruct(
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
    if unlock_set.m_signature_type == constants::signature_types::Strict {
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
    let leaveHash = calcUnlockHash(&unlock_set, "keccak256");

    let merkle_proof = &unlock_set.m_merkle_proof.iter().map(|&x| x.to_string()).collect::<Vec<String>>();
    let mut merkle_root = get_root_by_a_prove(
        &leaveHash,
        merkle_proof,
        &unlock_set.m_left_hash.to_string(),
        &input_type,
        &hash_algorithm);
    merkle_root = ccrypto::keccak256_dbl(&merkle_root);  // because of securiy, MUST use double hash

    if (vec![constants::HU_DNA_SHARE_ADDRESS, constants::HU_INAME_OWNER_ADDRESS].contains(&address)) &&
        (unlock_set.m_signature_type == constants::signature_types::Mix23) {
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
//         leaveHash = hash_algorithm(${unlock_set.sType}:${unlock_set.sVer}:${JSON.stringify(premu)}:${unlock_set.salt});
//         merkle_root = crypto.merkleGetRootByAProve(leaveHash, unlock_set.proofs, unlock_set.lHash, input_type, hash_algorithm);
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

pub fn customStringifySignatureSets(signature_sets: &Vec<&IndividualSignature>) -> String
{
    let mut sSets_serial: Vec<String> = vec![];
    for &a_sig in signature_sets
    {
        let mut tmp = "{\"sKey\":\"".to_owned() + a_sig.m_signature_key + "\"";

        if a_sig.m_permitted_to_pledge != ""
        {
            tmp += &(",\"pPledge\":\"".to_owned() + a_sig.m_permitted_to_pledge + "\"");
        }

        if a_sig.m_permitted_to_delegate != ""
        {
            tmp += &(",\"pDelegate\":\"".to_owned() + a_sig.m_permitted_to_delegate + "\"");
        }

        tmp += "}";

        sSets_serial.push(tmp);
    }
    let custom_stringify = "[".to_owned() + &sSets_serial.join(",") + "]";  //  JSON.stringify(sSet)
    return custom_stringify;
}

//old_name_was validateStructureStrictions
pub fn validate_structure_restrictions(
    unlock_set: &UnlockSet,
    options: &HashMap<&str, &str>) -> bool
{
    // console.log(validate StructureStrictions.args: ${utils.stringify(args)});
    let hash_algorithm = my_get(&options, "hash_algorithm", "keccak256").to_string();

    if unlock_set.m_signature_type == constants::signature_types::Strict
    {
        /**
         * this strict type of signature MUST have and ONLY have these 3 features
         * sKey: can be a public key(and later can be also another bech32 address, after implementing nested signature feature)
         * pPledge: means the signer Permitted to Pleadge this account
         * pDelegate: means the signer Permited to Delegate some rights (binded to this address) to others
         */

        if cutils::hash16c(&ccrypto::keccak256(&customStringifySignatureSets(&unlock_set.m_signature_sets))) != unlock_set.m_salt
        {
            dlog(
                &format!("invalid strict structure of signature of salt({}) ", unlock_set.m_salt),
                constants::Modules::App,
                constants::SecLevel::Info);
            return false;
        }

        for aSignSet in unlock_set.m_signature_sets
        {
            if (aSignSet.m_signature_key == "") || (aSignSet.m_permitted_to_pledge == "") ||
                (aSignSet.m_permitted_to_delegate == "")

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
