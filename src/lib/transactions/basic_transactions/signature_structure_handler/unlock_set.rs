use serde::{Serialize, Deserialize};

use crate::lib::constants;
use crate::lib::transactions::basic_transactions::signature_structure_handler::individual_signature::{dump_vec_of_ind_sig, IndividualSignature};
use crate::lib::utils::dumper::dump_vec_of_str;

#[derive(Serialize, Deserialize)]
pub struct UnlockSet<'a>
{
    pub(crate) m_signature_type: &'a str,
    pub(crate) m_signature_ver: &'a str,
    pub(crate) m_signature_sets: &'a Vec<&'a IndividualSignature<'a>>,
    pub(crate) m_merkle_proof: &'a Vec<&'a str>,
    pub(crate) m_left_hash: &'a str,
    pub(crate) m_salt: &'a str,
}

impl<'a> UnlockSet<'a> {
    pub fn new() -> UnlockSet<'a> {
        return UnlockSet {
            m_signature_type: constants::signature_types::Basic,
            m_signature_ver: "0.0.0",
            m_signature_sets: &vec![],
            m_merkle_proof: &vec![],
            m_left_hash: "",
            m_salt: "",
        };
    }

    pub fn dump(&self) -> String {
        let prefix_tabs = "\t ";
        let mut out_str = constants::NL.to_owned() + &prefix_tabs + "Signature: " + &self.m_signature_type + "(" + &self.m_signature_ver + ")";
        out_str += &(constants::NL.to_owned() + &prefix_tabs + "salt: " + &self.m_salt + "(lHash " + &self.m_left_hash + ")");
        out_str += &(constants::NL.to_owned() + &prefix_tabs + "Proofs: " + &prefix_tabs + constants::DUMPER_INDENT + &dump_vec_of_str(self.m_merkle_proof));
        out_str += &(constants::NL.to_owned() + &prefix_tabs + "Signature sets: " + &dump_vec_of_ind_sig(&self.m_signature_sets));
        return out_str;
    }
}

/*

QJsonObject UnlockSet::exportToJson()
{
  QJsonArray signature_sets{};
  for (IndividualSignature aSig: m_signature_sets)
  {
    signature_sets.push(aSig.exportJson());
  }
  return QJsonObject {
    {"sType", m_signature_type},
    {"sVer", m_signature_ver},
    {"mProof", CUtils::convertQStringListToJSonArray(m_merkle_proof)},
    {"sSets", signature_sets},
    {"lHash", m_left_hash},
    {"salt", m_salt}
  };
}

void UnlockSet::importJson(const QJsonObject& obj)
{
  m_signature_type = obj.value("sType").to_string();
  m_signature_ver = obj.value("sVer").to_string();
  m_merkle_proof = CUtils::convertJSonArrayToQStringList(obj.value("mProof").toArray());
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