use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::{constants};
use crate::lib::custom_types::CAddressT;
use crate::lib::transactions::basic_transactions::signature_structure_handler::unlock_set::UnlockSet;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UnlockDocument {
    pub m_unlock_sets: Vec<UnlockSet>,
    pub m_merkle_root: String,
    pub m_account_address: CAddressT,
    pub m_merkle_version: String,
    pub m_private_keys: HashMap<String, Vec<String>>,//HashMap<String, StringList>
}

impl UnlockDocument {
    pub fn new() -> Self {
        return UnlockDocument {
            m_unlock_sets: vec![],
            m_merkle_root: "".to_string(),
            m_account_address: "".to_string(),
            m_merkle_version: "0.0.0".to_string(),
            m_private_keys: Default::default(),
        };
    }

    pub fn dump(&self) -> String {
        let prefix_tabs = constants::TAB;
        let mut out_str: String = constants::NL.to_owned().to_owned() + &prefix_tabs + "merkle_root: " + &self.m_merkle_root + "(" + &self.m_merkle_version + ")";
        out_str += &(constants::NL.to_owned() + &prefix_tabs + &"account_address: " + &self.m_account_address + &constants::NL.to_owned() + &"unlock_sets");
        return constants::NL.to_owned() + &prefix_tabs + &out_str + &dump_vec_of_unlock_sets(&self.m_unlock_sets);
    }
}

pub fn dump_vec_of_unlock_sets(custom_data: &Vec<UnlockSet>) -> String {
    let prefix_tabs = "\t ";
    let mut out: Vec<String> = vec![];
    for a_set in custom_data
    {
        let dumped_row = prefix_tabs.to_owned() + &a_set.dump();
        out.push(dumped_row);
    }

    let joined = out.iter().map(|x| x.clone()).collect::<Vec<String>>().join(&constants::NL);
    return constants::NL.to_owned() + prefix_tabs + &joined;
}

/*
  std::tuple<bool, JSonObject> exportUnlockSet(const uint32_t unlock_index) const;  // uSet
  JSonObject exportJson() const;
  void importJson(const JSonObject &obj);
};

void UnlockDocument::importJson(const JSonObject &obj)
{
  m_merkle_root = obj.value("m_merkle_root").to_string();
  m_account_address = obj.value("m_account_address").to_string();
  m_merkle_version = obj.value("m_merkle_version").to_string();
  m_unlock_sets = {};
  for (auto a_u_set: obj.value("uSets").toArray())
  {
    UnlockSet uO;
    uO.importJson(a_u_set.toObject());
    m_unlock_sets.push(uO);
  }
  m_private_keys = {};
  JSonObject private_keys = obj.value("the_private_keys").toObject();
  for (String a_salt: private_keys.keys())
  {
    StringList priv_keys{};
    for (QJsonValueRef a_ky: private_keys[a_salt].toArray())
    {
      priv_keys.push(a_ky.to_string());
    }
    m_private_keys.insert(a_salt, priv_keys);
  }
}

JSonObject UnlockDocument::exportJson() const
{
  JSonArray unlock_sets{};
  for (UnlockSet a_u_set: m_unlock_sets)
  {
    unlock_sets.push(a_u_set.exportToJson());
  }

  JSonObject private_keys;
  for (String a_salt: m_private_keys.keys())
  {
    JSonArray priv_keys{};
    for (String a_ky: m_private_keys[a_salt])
    {
      priv_keys.push(a_ky);
    }
    private_keys.insert(a_salt, priv_keys);
  }

  return JSonObject {
    {"m_merkle_root", m_merkle_root},
    {"m_account_address", m_account_address},
    {"m_merkle_version", m_merkle_version},
    {"uSets", unlock_sets},
    {"the_private_keys", private_keys},
  };
}

std::tuple<bool, JSonObject> UnlockDocument::exportUnlockSet(const uint32_t unlock_index) const
{
  if (unlock_index >= static_cast<uint32_t>(m_unlock_sets.len()))
    return {false, {}};
  auto unlock_set = m_unlock_sets[unlock_index];
  return {true, unlock_set.exportToJson()};
}

 */