use serde::{Serialize, Deserialize};
use crate::lib::constants;
use crate::lib::custom_types::TimeByHoursT;


#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct IndividualSignature
{
    pub m_signer_id: String,
    // a dummy handler id
    pub m_signature_key: String,
    // sKey
    pub m_permitted_to_pledge: String,
    // pPledge
    pub m_permitted_to_delegate: String,
    // pDelegate
    pub m_input_time_lock: TimeByHoursT,
    pub m_input_time_lock_strickt: TimeByHoursT,
    pub m_output_time_lock: TimeByHoursT,
}

impl IndividualSignature {
    #[allow(unused, dead_code)]
    pub fn new() -> Self {
        return IndividualSignature {
            m_signer_id: "".to_string(),
            m_signature_key: "".to_string(),
            m_permitted_to_pledge: "".to_string(),
            m_permitted_to_delegate: "".to_string(),
            m_input_time_lock: 0.0,
            m_input_time_lock_strickt: 0.0,
            m_output_time_lock: 0.0,
        };
    }

    #[allow(unused, dead_code)]
    pub fn dump(&self) -> String {
        let prefix_tabs = "\t ";
        let mut out_str = constants::NL.to_owned() + &prefix_tabs + "signature_key: " + &self.m_signature_key;
        if self.m_permitted_to_pledge != ""
        {
            out_str += &(constants::NL.to_owned() + &prefix_tabs + "permitted_to_pledge: " + &self.m_permitted_to_pledge);
        }
        if self.m_permitted_to_delegate != ""
        {
            out_str += &(constants::NL.to_owned() + &prefix_tabs + "permitted_to_delegate: " + &self.m_permitted_to_delegate);
        }
        return out_str;
    }
}

pub fn dump_vec_of_ind_sig(custom_data: &Vec<IndividualSignature>) -> String {
    let prefix_tabs = "\t ";
    let mut out_str = constants::NL.to_string();
    for an_ind in custom_data{
        out_str += &(constants::NL.to_owned() + &prefix_tabs + "an Individual Signature: " + &*an_ind.dump());
    }
    return out_str.to_string();
}

/*
  // a date for which, user can not spend this money before stated date
  TimeByHoursT m_input_time_lock_strict = 0;
//    note for implementation. there will be 2 differnt input lock time signatures.
//    free input lock time: either the user with lock time can escrow money, or other users without lock time can escrow money
//    strict input lock time: ONLY and ONLY one signature is valid and the owner ONLY can scrow money after time expiration.


  IndividualSignature(){};

  IndividualSignature(
    const String& signature_key,
    const String& permitted_to_pledge = "",
    const String& permitted_to_delegate = "",
    const TimeByHoursT input_time_lock_strict = 0);

  JSonObject exportJson() const
  {
    return JSonObject {
      {"sKey", m_signature_key},
      {"pPledge", m_permitted_to_pledge},
      {"pDelegate", m_permitted_to_delegate},
      {"iTLock", QVariant::fromValue(m_input_time_lock).toDouble()},
      {"oTLock", QVariant::fromValue(m_output_time_lock).toDouble()},
      {"iTLockSt", QVariant::fromValue(m_input_time_lock_strict).toDouble()}
    };
  }

  void importJson(const JSonObject& obj)
  {
    m_signature_key = obj.value("sKey").to_string();
    m_permitted_to_pledge = obj.value("pPledge").to_string();
    m_permitted_to_delegate = obj.value("pDelegate").to_string();
    m_input_time_lock = obj.value("iTLock").toDouble();
    m_output_time_lock = obj.value("oTLock").toDouble();
    m_input_time_lock_strict = obj.value("iTLockSt").toDouble();
  }
};


#endif // INDIVIDUALSIGNATURE_H


IndividualSignature::IndividualSignature(
  const String& signature_key,
  const String& permitted_to_pledge,
  const String& permitted_to_delegate,
  const TimeByHoursT input_time_lock_strict)
{
  m_signer_id = "";
  m_signature_key = signature_key;
  m_permitted_to_pledge = permitted_to_pledge;
  m_permitted_to_delegate = permitted_to_delegate;
  m_input_time_lock_strict = input_time_lock_strict;
}

*/