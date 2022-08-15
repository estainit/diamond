use crate::{CMachine, constants};
use crate::lib::block::block_types::block_genesis::genesis_block::b_genesis::initGenesisBlock;
use crate::lib::database::abs_psql::{OrderModifier, q_select, simple_eq_clause};
use crate::lib::database::tables::STBL_BLOCKS;
use crate::lib::k_v_handler::set_value;
use crate::lib::services::polling::polling_handler::initPollingProfiles;
use crate::lib::services::society_rules::society_rules::initAdministrativeConfigurationsHistory;

//old_name_was maybeInitDAG
pub fn maybe_init_dag(machine: &mut CMachine) -> bool
{

    // check if genesis block is created?
    let (status, records) = q_select(
        STBL_BLOCKS,
        &vec!["b_id", "b_hash"],     // fields
        &vec![
            simple_eq_clause( "b_type", constants::block_types::Genesis)],
        vec![
            &OrderModifier { m_field: "b_creation_date", m_order: "ASC" },
            &OrderModifier { m_field: "b_id", m_order: "ASC" },
        ],
        1,   // limit
        true,
    );
    if records.len() > 0
    { return true; }


    // create Genisis Block
    initGenesisBlock(machine);

    // init Administrative Configurations History
    initAdministrativeConfigurationsHistory(machine);

    // init Polling Profiles
    initPollingProfiles();

    // Initialize Agoras content
    /*
    // initialize registerd iNames
        status = INameHandler::initINames();
        if (!status)
        cutils::exiter("INameHandler initINames failed!", 908);
    */

    // initialize wiki pages

    return doesSafelyInitialized(machine);
}


pub fn doesSafelyInitialized(machine: &mut CMachine) -> bool
{
    // TODO implement it to controll if all intial document are inserted properly?
    machine.setDAGIsInitialized(true);

    // long list of controlls

    set_value("MACHINE_AND_DAG_ARE_SAFELY_INITIALIZED", constants::YES, false);

    return true;
}