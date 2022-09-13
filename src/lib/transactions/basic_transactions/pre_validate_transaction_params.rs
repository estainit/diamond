use crate::{constants, dlog};
use crate::lib::custom_types::CMPAIValueT;
use crate::lib::transactions::basic_transactions::basic_transaction_template::BasicTransactionTemplate;

//old_name_was preValidateTransactionParams
pub fn pre_validate_transaction_params(
    tpl: &BasicTransactionTemplate) -> (bool, String, CMPAIValueT, CMPAIValueT)
{
    let msg: String;

    if tpl.m_tpl_inputs.keys().len() == 0
    {
        msg = "Try to make transaction without inputs!".to_string();
        dlog(
            &msg,
            constants::Modules::Trx,
            constants::SecLevel::Error);
        return (false, msg, 0, 0);
    }

    let mut total_inputs_amount: CMPAIValueT = 0;
    for a_coin in tpl.m_tpl_inputs.keys()
    {
        total_inputs_amount += tpl.m_tpl_inputs[a_coin].m_amount;
    }

    if total_inputs_amount == 0
    {
        msg = "Try to make transaction with zero inputs!".to_string();
        dlog(
            &msg,
            constants::Modules::Trx,
            constants::SecLevel::Error);
        return (false, msg, 0, 0);
    }


    if tpl.m_tpl_outputs.len() == 0
    {
        msg = "Try to make transaction without output accounts!".to_string();
        dlog(
            &msg,
            constants::Modules::Trx,
            constants::SecLevel::Error);
        return (false, msg, 0, 0);
    }

    let mut total_outputs_amount_except_dp_costs: CMPAIValueT = 0;
    for an_out in &tpl.m_tpl_outputs
    {
        if !vec![constants::OUTPUT_CHANGE_BACK.to_string(), constants::OUTPUT_DP_COST.to_string()]
            .contains(&an_out.m_output_character)
        {
            total_outputs_amount_except_dp_costs += an_out.m_amount;
        }
    }

    if total_outputs_amount_except_dp_costs == 0
    {
        msg = "Try to make transaction without outputs!".to_string();
        dlog(
            &msg,
            constants::Modules::Trx,
            constants::SecLevel::Error);
        return (false, msg, 0, 0);
    }

    return (
        true,
        "".to_string(),
        total_inputs_amount,
        total_outputs_amount_except_dp_costs);
}
