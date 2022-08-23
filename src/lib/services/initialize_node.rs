use crate::{CMachine, constants};
use crate::lib::block::block_types::block_genesis::genesis_block::b_genesis::init_genesis_block;
use crate::lib::database::abs_psql::{OrderModifier, q_select, simple_eq_clause};
use crate::lib::database::tables::C_BLOCKS;
use crate::lib::k_v_handler::set_value;
use crate::lib::services::polling::polling_handler::init_polling_profiles;
use crate::lib::services::society_rules::society_rules::init_administrative_configurations_history;

//old_name_was maybeInitDAG
pub fn maybe_init_dag(machine: &mut CMachine) -> bool
{

    // check if genesis block is created?
    let (_status, records) = q_select(
        C_BLOCKS,
        vec!["b_id", "b_hash"],     // fields
        vec![
            simple_eq_clause( "b_type", &constants::block_types::GENESIS.to_string())],
        vec![
            &OrderModifier { m_field: "b_creation_date", m_order: "ASC" },
            &OrderModifier { m_field: "b_id", m_order: "ASC" },
        ],
        1,   // limit
        true,
    );
    if records.len() > 0
    {
        return true;
    }

    // create Genisis Block
    init_genesis_block(machine);

    // init Administrative Configurations History
    init_administrative_configurations_history(machine);

    // init Polling Profiles
    init_polling_profiles();

    // Initialize Agoras content
    /*
    // initialize registerd iNames
        status = INameHandler::initINames();
        if (!status)
        cutils::exiter("INameHandler initINames failed!", 908);
    */

    // initialize wiki pages

    return does_safely_initialized(machine);
}


pub fn does_safely_initialized(_machine: &mut CMachine) -> bool
{
    // TODO implement it to controll if all intial document are inserted properly?

    // long list of controlls

    set_value("machine_and_dag_are_safely_initialized", constants::YES, false);
    return true;
}
