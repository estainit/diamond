use crate::lib::constants;
use crate::lib::database::abs_psql::{q_select, simple_eq_clause};
use crate::lib::machine::machine_neighbor::{NeighborPresentation};
use crate::lib::transactions::basic_transactions::signature_structure_handler::unlock_document::UnlockDocument;
use serde::{Serialize, Deserialize};
use crate::dlog;
use crate::lib::database::tables::C_MACHINE_PROFILES;


/**
* the machine can have 1 or more diffrent profile(s)
* each profile has it's own public/private email&  public/private iPGP key pairs&  it's neightbors set&
* and it's wallet addresses and
* machine_onchain_contracts
* and maybe kvalue
*
* adding profile field to all tables
* machine_tmp_documents, machine_buffer_documents
* machine_neighbors, machine_wallet_addresses, machine_wallet_funds
* machine_draft_proposals, machine_used_utxos, machine_ballots, machine_draft_pledges
*
*   // the status be booting, synching, ready
 // booting: when nodes starts to connecting to network for first time
 // synching: when latest confirmed blocks are created before 12 hours ago
 // ready: the node has some confirmed blocks created in last 12 hour in his locl DB
 // status: constants.NODE_IS_BOOTING,
 // lastConfirmedBlockDate: IMAGINE LAUNCH DATE,

 // machine email setting
 // each node has 2 email addreess, public&  private to resist against the spamming...
 // TODO: maybe machine have to have ability to have more than one email to comunicate to prevent against any censorship

 */
#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct MachineProfile
{
    pub(crate) m_mp_code: String,
    pub(crate) m_mp_name: String,
    pub(crate) m_mp_last_modified: String,
    pub(crate) m_mp_settings: MPSetting,
}

impl MachineProfile {
    pub fn get_profile_from_db(mp_code: &str) -> (bool, MachineProfile)
    {
        let (_status, records) = q_select(
            C_MACHINE_PROFILES,
            vec!["mp_code", "mp_name", "mp_settings"],
            vec![
                simple_eq_clause("mp_code", mp_code)],
            vec![],   // order
            1,
            true,
        );
        if records.len() == 1
        {
            let mp_prof:MachineProfile = serde_json::from_str(&records[0]["mp_settings"].clone()).unwrap();

            // let (status, mp_settings) = match serde_json::from_str(&records[0]["mp_settings"].clone()) {
            //     Ok(s) => (true, s),
            //     Err(e) => {
            //         dlog(
            //             &format!("Failed in deserializing machine profile! {} {}",
            //                      e, records[0]["mp_settings"]),
            //             constants::Modules::App,
            //             constants::SecLevel::Error);
            //         panic!("zzzz z z z z zz z z z: {} {}",e,&records[0]["mp_settings"].clone());
            //         (false, MPSetting::new())
            //     }
            // };
            let machine_profile = MachineProfile {
                m_mp_code: mp_prof.m_mp_code,
                m_mp_name: mp_prof.m_mp_name,
                m_mp_last_modified: mp_prof.m_mp_last_modified,
                m_mp_settings: mp_prof.m_mp_settings,
            };
            return (true, machine_profile);
        }
        (false, MachineProfile::new())
    }

    pub fn new() -> MachineProfile {
        return MachineProfile {
            m_mp_code: "".to_string(),
            m_mp_name: "".to_string(),
            m_mp_last_modified: "".to_string(),
            m_mp_settings: MPSetting::new(),
        };
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MPSetting
{
    pub(crate) m_public_email: EmailSettings,
    pub(crate) m_private_email: EmailSettings,

    pub(crate) m_machine_alias: String,
    pub(crate) m_backer_detail: UnlockDocument,
    pub(crate) m_language: String,
    pub(crate) m_term_of_services: String,
    pub(crate) m_already_presented_neighbors: Vec<NeighborPresentation>,

}

impl MPSetting {
    pub fn new() -> MPSetting {
        return MPSetting {
            m_public_email: EmailSettings::new(),
            m_private_email: EmailSettings::new(),
            m_machine_alias: "Diamond_node".to_string(),
            m_backer_detail: UnlockDocument::new(),
            m_language: constants::DEFAULT_LANG.to_string(),
            m_term_of_services: constants::NO.to_string(),
            m_already_presented_neighbors: vec![],
        };
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct EmailSettings {
    pub(crate) m_address: String,
    pub(crate) m_password: String,
    pub(crate) m_income_imap: String,
    pub(crate) m_income_pop3: String,
    pub(crate) m_incoming_mail_server: String,
    pub(crate) m_outgoing_mail_server: String,
    pub(crate) m_outgoing_smtp: String,
    // it depends on smtp server, but less than 5 minute is useless
    pub(crate) m_fetching_interval_by_minute: String,
    pub(crate) m_pgp_private_key: String,
    pub(crate) m_pgp_public_key: String,
}

impl EmailSettings {
    pub fn new() -> EmailSettings {
        return EmailSettings {
            m_address: "abc@def.gh".to_string(),
            m_password: "".to_string(),
            m_income_imap: "993".to_string(),
            m_income_pop3: "995".to_string(),
            m_incoming_mail_server: "".to_string(),
            m_outgoing_mail_server: "".to_string(),
            m_outgoing_smtp: "465".to_string(),
            m_fetching_interval_by_minute: "5".to_string(),  // it depends on smtp server, but less than 5 minute is useles,
            m_pgp_private_key: "".to_string(),
            m_pgp_public_key: "".to_string(),
        };
    }
}

