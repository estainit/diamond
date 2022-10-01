use serde::{Serialize, Deserialize};
use crate::lib::custom_types::JSonObject;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FloatingVoteDocument
{
    pub m_some_field: String,
}

impl FloatingVoteDocument
{
    pub fn new() -> Self
    {
        Self {
            m_some_field: "".to_string()
        }
    }

    pub fn load_floating_votes(floating_votes: &JSonObject) -> Vec<FloatingVoteDocument>
    {
        let mut out: Vec<FloatingVoteDocument> = vec![];
        for an_f_vote in floating_votes.as_array().unwrap()
        {
            let mut an_f: FloatingVoteDocument = FloatingVoteDocument::new();
            if !an_f_vote["Something..."].is_null()
            {
                an_f.m_some_field = "Something...".to_string()
            }
            out.push(an_f);
        }
        out
    }
}