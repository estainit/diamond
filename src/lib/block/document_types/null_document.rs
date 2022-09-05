use crate::lib::custom_types::COutputIndexT;
use crate::lib::transactions::basic_transactions::signature_structure_handler::general_structure::{TInput, TOutput};
use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct NullDocument
{
    pub m_inputs: Vec<TInput>,
    pub m_outputs: Vec<TOutput>,
    m_data_and_process_payment_indexes: Vec<COutputIndexT>,  // dPIs
}

impl NullDocument {
    pub fn new() -> NullDocument {
        NullDocument {
            m_inputs: vec![],
            m_outputs: vec![],
            m_data_and_process_payment_indexes: vec![],
        }
    }

    //old_name_was getInputs
    pub fn get_inputs(&self) -> &Vec<TInput>
    {
        return &self.m_inputs;
    }

    pub fn get_outputs(&self) -> &Vec<TOutput>
    {
        return &self.m_outputs;
    }

    pub fn get_dpis(&self) -> &Vec<COutputIndexT>
    {
        return &self.m_data_and_process_payment_indexes;
    }

}