use crate::lib::constants;
use crate::lib::database::abs_psql::{ModelClause, q_select};
use crate::lib::machine::machine_neighbor::{ NeighborInfo};
use crate::lib::transactions::basic_transactions::signature_structure_handler::unlock_document::UnlockDocument;
use serde::{Serialize, Deserialize};
use crate::lib::database::tables::STBL_MACHINE_PROFILES;


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


#[derive(Serialize, Deserialize)]
pub(crate) struct MachineProfile<'m>
{
    m_dummy_m_lifetime_user: &'m str,
    pub(crate) m_mp_code: String,
    pub(crate) m_mp_name: String,
    pub(crate) m_mp_last_modified: String,
    pub(crate) m_mp_settings: MPSetting<'m>,
}

impl <'m>MachineProfile<'m> {
    pub fn get_profile(mp_code: &str) -> (bool, MachineProfile)
    {
        let (status, records) = q_select(
            STBL_MACHINE_PROFILES,
            &vec!["mp_code", "mp_name", "mp_settings"],
            &vec![
                &ModelClause {
                    m_field_name: "mp_code",
                    m_field_single_str_value: mp_code,
                    m_clause_operand: "=",
                    m_field_multi_values: vec![],
                }],
            &vec![],   // order
            1,
            true,
        );
        if records.len() == 1
        {
            println!("XXXXX: {:?}", records);
            // let mp: MachineProfile = serde_json::from_str(records[0]["mp_settings"]).unwrap();
            // (true, mp)
            return (false, MachineProfile::get_null());
        }
        (false, MachineProfile::get_null())
    }

    pub fn get_null() -> MachineProfile<'m> {
        return MachineProfile {
            m_dummy_m_lifetime_user: "",
            m_mp_code: "".to_string(),
            m_mp_name: "".to_string(),
            m_mp_last_modified: "".to_string(),
            m_mp_settings: MPSetting {
                m_dummy_m_lifetime_user: "",
                m_public_email: EmailSettings::get_null(),
                m_private_email: EmailSettings::get_null(),
                m_machine_alias: "".to_string(),
                m_backer_detail: UnlockDocument::get_null(),
                m_language: "".to_string(),
                m_term_of_services: "".to_string(),
                m_already_presented_neighbors: vec![],
            },
        };
    }
}

#[derive(Serialize, Deserialize)]
pub struct MPSetting<'m>
{
    m_dummy_m_lifetime_user: &'m str,
    pub(crate) m_public_email: EmailSettings,
    pub(crate) m_private_email: EmailSettings,

    pub(crate) m_machine_alias: String,
    pub(crate) m_backer_detail: UnlockDocument,
    pub(crate) m_language: String,
    pub(crate) m_term_of_services: String,
    pub(crate) m_already_presented_neighbors: Vec<NeighborInfo>,

}

impl <'m>MPSetting<'m> {
    pub fn new() -> MPSetting <'m>{
        return MPSetting {
            m_dummy_m_lifetime_user: "",
            m_public_email: EmailSettings::new(),
            m_private_email: EmailSettings::new(),
            m_machine_alias: "Diamond_node".to_string(),
            m_backer_detail: UnlockDocument::get_null(),
            m_language: constants::DEFAULT_LANG.to_string(),
            m_term_of_services: constants::NO.to_string(),
            m_already_presented_neighbors: vec![],
        };
    }
}

#[derive(Serialize, Deserialize)]
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

    pub fn get_null() -> EmailSettings {
        return EmailSettings {
            m_address: "".to_string(),
            m_password: "".to_string(),
            m_income_imap: "".to_string(),
            m_income_pop3: "".to_string(),
            m_incoming_mail_server: "".to_string(),
            m_outgoing_mail_server: "".to_string(),
            m_outgoing_smtp: "".to_string(),
            m_fetching_interval_by_minute: "".to_string(),
            m_pgp_private_key: "".to_string(),
            m_pgp_public_key: "".to_string(),
        };
    }
}

