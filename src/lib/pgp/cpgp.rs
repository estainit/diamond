use crate::{ constants };
use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct CPGPMessage
{
    pub m_decryption_status: bool,
    pub m_pgp_versaion: String,
    pub m_secret_key: String,
    pub m_initialization_vector: String,
    pub m_decrypted_secret_key: String,
    pub m_decrypted_initialization_vector: String,
    pub m_message: String,
    pub m_aes_version: String,
    pub m_signature_version: String,
    pub m_is_signed: bool,
    pub m_is_compressed: bool,
    pub m_is_verified: bool,
    pub m_is_authenticated: bool,
    pub m_zip_version: String,
    pub m_zip_algorithm: String,
}

impl CPGPMessage {
    pub fn new() -> CPGPMessage {
        CPGPMessage {
            m_decryption_status: false,
            m_pgp_versaion: constants::CURRENT_PGP_VERSION.to_string(),
            m_secret_key: "".to_string(),
            m_initialization_vector: "".to_string(),
            m_decrypted_secret_key: "".to_string(),
            m_decrypted_initialization_vector: "".to_string(),
            m_message: "".to_string(),
            m_aes_version: "".to_string(),
            m_signature_version: "".to_string(),
            m_is_signed: false,
            m_is_compressed: false,
            m_is_verified: false,
            m_is_authenticated: false,
            m_zip_version: "".to_string(),
            m_zip_algorithm: "".to_string(),
        }
    }
}

//old_name_was stripPGPEnvelope
pub fn strip_pgp_envelope(content: &String) -> String
{
    let mut content: String = content.trim().to_string(); // remove extra spaces
    if content.contains(constants::message_tags::ENVELOPE_I_PGP_START)
    {
        content = content.split(constants::message_tags::ENVELOPE_I_PGP_START).collect::<Vec<&str>>()[1].to_string().clone();
    }
    if content.contains(constants::message_tags::ENVELOPE_I_PGP_END)
    {
        content = content.split(constants::message_tags::ENVELOPE_I_PGP_END).collect::<Vec<&str>>()[0].to_string().clone();
    }
    return content;
}

//old_name_was wrapPGPEnvelope
pub fn wrap_pgp_envelope(content: &String) -> String
{
    return constants::message_tags::ENVELOPE_I_PGP_START.to_owned() + content + constants::message_tags::ENVELOPE_I_PGP_END;
}
