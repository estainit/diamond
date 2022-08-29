use crate::{application};
use crate::lib::block::block_types::block_coinbase::cb1_maybe_create_coinbase_block::does_dag_has_more_confidence_cb;
use crate::lib::block::block_types::block_coinbase::cb2_random_ordering_neighbors::if_i_have_first_hashed_email;
use crate::lib::constants;
use crate::lib::custom_types::TimeBySecT;
use crate::lib::dag::leaves_handler::{has_fresh_leaves};
use crate::lib::dag::missed_blocks_handler::get_missed_blocks_to_invoke;
use crate::lib::dlog::dlog;
use crate::lib::messaging_protocol::dag_message_handler::set_maybe_ask_for_latest_blocks_flag;


//old_name_was controlCoinbaseIssuanceCriteria
pub fn control_coinbase_issuance_criteria() -> bool
{
    let now_ = application().now();
    let current_cycle_range = application().get_coinbase_range(&now_);
    dlog(
        &format!("Coinbase check Range (from {} to {} ", current_cycle_range.from, current_cycle_range.to),
        constants::Modules::CB,
        constants::SecLevel::Info);


    if !has_fresh_leaves()
    {
        // younger than 2 cycle
        dlog(
            &format!("Machine hasn't fresh leaves!"),
            constants::Modules::CB,
            constants::SecLevel::Info);
        set_maybe_ask_for_latest_blocks_flag(constants::YES);
        return false;
    }
    // control if already exist in DAG a more confidence Coinbase Block than what machine can create?
    let dag_has_more_confidence_cb: bool = does_dag_has_more_confidence_cb();
    if dag_has_more_confidence_cb {
        dlog(
            &format!("Machine already has more confidente Coinbase blocks in DAG"),
            constants::Modules::CB,
            constants::SecLevel::Info);
        return false;
    }


    // // some control to be sure the coinbase block for current 12 hour cycle didn't create still,
    // // or at least I haven't it in my local machine
    // let latestCoinbase = cbBufferHandler.getMostConfidenceFromBuffer(currntCoinbaseTimestamp);
    // if (latestCoinbase.cycle == currntCoinbaseTimestamp) {
    //     // check if after sending cb to other , the local machine has added any new block to DAG? and machine still doesn't receive confirmed cb block?
    //     // so must create nd send new coinbase block
    //     msg = `At least one CB exists on local machine: ${latestCoinbase} (${iutils.getCoinbaseRange().from.split(' ')[1]} - ${iutils.getCoinbaseRange().to.split(' ')[1]})`
    //     clog.cb.info(msg);
    //     res.msg = msg
    //     res.atLeastOneCBExists = true;
    // }

    // postpond coinbase-generating if machine missed some blocks
    let missed_blocks: Vec<String> = get_missed_blocks_to_invoke(0);

    if missed_blocks.len() > 0 {

        // // FIXME: if an adversory sends a bunch of blocks which have ancestors, in machine will finished with a long list of
        // // missed blocks. so machine(or entire network machines) can not issue new coinbase block!
        // if (missed_blocks.length > iConsts.MAX_TOLERATED_MISS_BLOCKS) {
        //     msg = `Machine missed ${missed_blocks.length} blocks and touched the limitation(${iConsts.MAX_TOLERATED_MISS_BLOCKS}), so can not issue a coinbase block`
        //     clog.cb.info(msg);
        //     res.canGenCB = false;
        //     res.msg = msg
        //     return res;
        // }

        let now_ = application().now();
        let latenancy_factor: f64 = ((missed_blocks.len() + 1) as f64 / (constants::MAX_TOLERATED_MISS_BLOCKS as f64)) * application().get_coinbase_age_by_seconds(&now_) as f64;
        let are_we_in_4_of_5: bool = application().get_coinbase_age_by_seconds(&now_) < (application().get_cycle_by_seconds() * 4 / 5);
        if are_we_in_4_of_5 && (application().get_coinbase_age_by_seconds(&now_) < latenancy_factor as TimeBySecT)
        {
            dlog(
                &format!("Because of {} missed blocks, machine can not create a CB before {} second age of cycle or atleast 4/5 of cycle age passed", missed_blocks.len(), latenancy_factor),
                constants::Modules::CB,
                constants::SecLevel::Info);
            return false;
        }
    }

    // a psudo random mechanisem
    let am_i_qualified_to_issue_coinbase: bool = if_i_have_first_hashed_email("asc");
    if !am_i_qualified_to_issue_coinbase {
        dlog(
            &format!("It is not machine turn To Create Coinbase Block!"),
            constants::Modules::CB,
            constants::SecLevel::TmpDebug);
        return false;
    }
    return true;
}