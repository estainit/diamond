use serde::{Serialize, Deserialize};
use serde_json::json;
use crate::cutils::remove_quotes;

use crate::lib::constants;
use crate::lib::custom_types::{JSonObject, VString};
use crate::lib::transactions::basic_transactions::signature_structure_handler::individual_signature::{dump_vec_of_ind_sig, IndividualSignature};
use crate::lib::utils::dumper::dump_vec_of_str;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UnlockSet
{
    pub m_signature_type: String,
    pub m_signature_ver: String,
    pub m_signature_sets: Vec<IndividualSignature>,
    pub m_merkle_proof: VString,
    pub m_left_hash: String,
    pub m_salt: String,
}

impl UnlockSet {
    pub fn new() -> Self {
        return UnlockSet {
            m_signature_type: constants::signature_types::BASIC.to_string(),
            m_signature_ver: "0.0.0".to_string(),
            m_signature_sets: vec![],
            m_merkle_proof: vec![],
            m_left_hash: "".to_string(),
            m_salt: "".to_string(),
        };
    }

    pub fn extract_signers_pub_keys(&self) -> VString
    {
        let mut pub_keys: VString = vec![];
        for a_signature_set in &self.m_signature_sets
        {
            pub_keys.push(a_signature_set.m_signature_key.clone());
        }
        pub_keys
    }

    pub fn export_to_json(&self) -> JSonObject
    {
        let mut signature_sets: Vec<JSonObject> = vec![];
        for a_sig in &self.m_signature_sets
        {
            signature_sets.push(a_sig.export_to_json());
        }
        let out = json!({
           "sType": self.m_signature_type,
           "sVer": self.m_signature_ver,
           "sSets": signature_sets,
           "mProof": self.m_merkle_proof,
           "lHash": self.m_left_hash,
           "salt": self.m_salt,
        });
        return out;
    }

    pub fn load_from_json(j_obj: &JSonObject) -> (bool, Self)
    {
        let mut signature_sets: Vec<IndividualSignature> = vec![];
        for a_set in j_obj["sSets"].as_array().unwrap()
        {
            let (status, inv_set) = IndividualSignature::load_from_json(a_set);
            if !status
            {
                return (false, Self::new());
            }
            signature_sets.push(inv_set);
        }

        let mut merkle_proof: VString = vec![];
        for an_item in j_obj["mProof"].as_array().unwrap()
        {
            merkle_proof.push(remove_quotes(an_item));
        }
        let out = Self {
            m_signature_type: remove_quotes(&j_obj["sType"]),
            m_signature_ver: remove_quotes(&j_obj["sVer"]),
            m_signature_sets: signature_sets,
            m_merkle_proof: merkle_proof,
            m_left_hash: remove_quotes(&j_obj["lHash"]),
            m_salt: remove_quotes(&j_obj["salt"]),
        };
        return (true, out);
    }

    pub fn dump(&self) -> String {
        let prefix_tabs = "\t ";
        let mut out_str = constants::NL.to_owned() + &prefix_tabs + "Signature: " + &self.m_signature_type + "(" + &self.m_signature_ver + ")";
        out_str += &(constants::NL.to_owned() + &prefix_tabs + "salt: " + &self.m_salt + "(lHash " + &self.m_left_hash + ")");
        out_str += &(constants::NL.to_owned() + &prefix_tabs + "Proofs: " + &prefix_tabs + constants::DUMPER_INDENT + &dump_vec_of_str(&self.m_merkle_proof));
        out_str += &(constants::NL.to_owned() + &prefix_tabs + "Signature sets: " + &dump_vec_of_ind_sig(&self.m_signature_sets));
        return out_str;
    }
}

/*


void UnlockSet::importJson(const JSonObject& obj)
{
  m_signature_type = obj.value("sType").to_string();
  m_signature_ver = obj.value("sVer").to_string();
  m_merkle_proof = cutils::convertJSonArrayToStringVector(obj.value("mProof").toArray());
  m_left_hash = obj.value("lHash").to_string();
  m_salt = obj.value("salt").to_string();
  m_signature_sets = {};
  for (auto a_signature_set: obj.value("sSets").toArray())
  {
    IndividualSignature sig_obj;
    sig_obj.importJson(a_signature_set.toObject());
    m_signature_sets.push(sig_obj);
  }

}

 */