use crate::{application, constants, cutils, dlog, machine};
use crate::lib::block::block_types::block_coinbase::coinbase_coins_handler::calc_coinbased_output_maturation_date;
use crate::lib::block::block_types::block_factory::load_block;
use crate::lib::block_utils::unwrap_safed_content_for_db;
use crate::lib::custom_types::{CMPAISValueT, COutputIndexT, QVDicT, VString};
use crate::lib::transactions::basic_transactions::signature_structure_handler::general_structure::TOutput;
use crate::lib::wallet::wallet_coins::{delete_from_funds, insert_a_coin_in_wallet};
use crate::lib::wallet::wallet_signer::exclude_locally_used_coins;

//old_name_was updateFundsFromNewBlock
pub fn update_funds_from_new_block(
    block_record: &QVDicT,
    wallet_addresses: &VString) -> bool
{
    // since all validation controls (relatd to transaction input/output) are already done before recording block_record in DAG,
    //here we trust the block_record information and just extract the funds from transactions

    let mp_code = machine().get_selected_m_profile();
    println!("xxxxx xxxxx 333 {}", mp_code);
    let (_status, _sf_ver, content) =
        unwrap_safed_content_for_db(&block_record["b_body"]);
    let (_status, j_block) = cutils::controlled_str_to_json(&content);
    dlog(
        &format!(
            "Update wallet funds for Block: {}",
            j_block),
        constants::Modules::Trx,
        constants::SecLevel::TmpDebug);

    let (_status, block) = load_block(&j_block);
    dlog(
        &format!(
            "Update wallet funds for Block {}",
            &block.get_block_identifier()),
        constants::Modules::Trx,
        constants::SecLevel::TmpDebug);

    // extract unmatured outputs
    println!("xxxxx xxxxx 444 {} documents",
             block.m_block_documents.len());

    for a_doc in &block.m_block_documents
    {
        if a_doc.m_doc_type == constants::document_types::COINBASE.to_string()
        {
            let outputs: &Vec<TOutput> = a_doc.get_outputs();
            let mut output_index: COutputIndexT = 0;
            while output_index < outputs.len() as COutputIndexT
            {
                println!("xxxxx xxxxx 555 {} outputs {} {}",
                         outputs.len(), a_doc.m_doc_type, output_index);
                let an_output = &outputs[output_index as usize];
                if !wallet_addresses.contains(&an_output.m_address)
                { continue; }

                let maturate_date =
                    calc_coinbased_output_maturation_date(&block.m_block_creation_date);
                insert_a_coin_in_wallet(
                    &block.m_block_hash,
                    &a_doc.m_doc_hash,
                    output_index as COutputIndexT,
                    &an_output.m_address,
                    an_output.m_amount as CMPAISValueT,
                    &a_doc.m_doc_type,
                    &block.m_block_creation_date,
                    &maturate_date,
                    &mp_code,
                );
                output_index += 1;
            }
        } else if a_doc.m_doc_type == constants::document_types::DATA_AND_PROCESS_COST_PAYMENT.to_string()
        {
            let outputs: &Vec<TOutput> = a_doc.get_outputs();
            let mut output_index: COutputIndexT = 0;
            while output_index < outputs.len() as COutputIndexT
            {
                let an_output = &outputs[output_index as usize];
                // import only wallet controlled funds, implicitely removes the "TP_DP" outputs too.
                if !wallet_addresses.contains(&an_output.m_address)
                { continue; }

                let cycle_by_minutes = application().get_cycle_by_minutes();
                let maturate_date = application().minutes_after(
                    cycle_by_minutes,
                    &block.m_block_creation_date);
                insert_a_coin_in_wallet(
                    &block.m_block_hash,
                    &a_doc.m_doc_hash,
                    output_index as COutputIndexT,
                    &an_output.m_address,
                    an_output.m_amount as CMPAISValueT,
                    &a_doc.m_doc_type,
                    &block.m_block_creation_date,
                    &maturate_date,
                    &mp_code,
                );
                output_index += 1;
            }
        } else if a_doc.m_doc_type == constants::document_types::BASIC_TX
        {
            let outputs: &Vec<TOutput> = a_doc.get_outputs();
            let mut output_index: COutputIndexT = 0;
            while output_index < outputs.len() as COutputIndexT
            {
                let an_output = &outputs[output_index as usize];

                // import only wallet controlled funds, implicitely removes the outputs dedicated to treasury payments too
                if !wallet_addresses.contains(&an_output.m_address)
                { continue; }

                // do not import DPCost payment outputs, because they are already spent in DPCostPay doc
                if a_doc.get_dpis().contains(&output_index)
                { continue; }

                let cycle_by_minutes = application().get_cycle_by_minutes();
                let maturate_date = application().minutes_after(
                    cycle_by_minutes,
                    &block.m_block_creation_date);
                insert_a_coin_in_wallet(
                    &block.m_block_hash,
                    &a_doc.m_doc_hash,
                    output_index,
                    &an_output.m_address,
                    an_output.m_amount as CMPAISValueT,
                    &a_doc.m_doc_type,
                    &block.m_block_creation_date,
                    &maturate_date,
                    &mp_code,
                );
                output_index += 1;
            }

            // removing spent UTXOs in block too
            for an_input in a_doc.get_inputs()
            {
                delete_from_funds(
                    &an_input.m_transaction_hash,
                    an_input.m_output_index,
                    &mp_code,
                );
            }
        } else if a_doc.m_doc_type == constants::document_types::REPAYMENT_DOCUMENT
        {
            let outputs: &Vec<TOutput> = a_doc.get_outputs();
            let mut output_index: COutputIndexT = 0;
            while output_index < outputs.len() as COutputIndexT
            {
                let an_output = &outputs[output_index as usize];

                // import only wallet controlled funds, implicitly removes the outputs dedicated to treasury payments too
                if a_doc.get_dpis().contains(&output_index)
                { continue; }

                // import only wallet controlled funds, implicitely removes the outputs dedicated to treasury payments too
                if !wallet_addresses.contains(&an_output.m_address)
                { continue; }

                let cycle_by_minutes = application().get_cycle_by_minutes();
                let maturate_date = application().minutes_after(
                    cycle_by_minutes,
                    &block.m_block_creation_date);
                insert_a_coin_in_wallet(
                    &block.m_block_hash,
                    &a_doc.m_doc_hash,
                    output_index as COutputIndexT,
                    &an_output.m_address,
                    an_output.m_amount as CMPAISValueT,
                    &a_doc.m_doc_type,
                    &block.m_block_creation_date,
                    &maturate_date,
                    &mp_code,
                );
            }

            // removing spent UTXOs in block too
            for an_input in a_doc.get_inputs()
            {
                delete_from_funds(
                    &an_input.m_transaction_hash,
                    an_input.m_output_index,
                    &mp_code);
            }
        }
    }

    // finally remove locally used coins from wallet funds
    exclude_locally_used_coins();

    return true;
}


