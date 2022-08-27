use crate::{constants, cutils, dlog};
use crate::lib::block::block_types::block::Block;
use crate::lib::block::block_types::block_ancestors_controls::ancestors_controls;
use crate::lib::custom_types::{ClausesT};
use crate::lib::dag::dag_walk_through::get_cached_blocks_hashes;
use crate::lib::database::abs_psql::{q_delete};
use crate::lib::database::tables::{C_PARSING_Q};
use crate::lib::parsing_q_handler::queue_pars::EntryParsingResult;

/*

void ParsingQHandler::loopSmartPullFromParsingQ()
{
  String thread_prefix = "pull_from_parsing_q_";
  String thread_code = String::number((quint64)QThread::currentThread(), 16);

  while (CMachine::shouldLoopThreads())
  {
    CMachine::reportThreadStatus(thread_prefix, thread_code, constants::THREAD_STATE::RUNNING);
    MissedBlocksHandler::refreshMissedBlock();

    smartPullQ();

    CLog::log("Smart Pull From Parsing Q, Every (" + String::number(CMachine::getParsingQGap()) + " seconds) ", "app", "trace");
    CMachine::reportThreadStatus(thread_prefix, thread_code, constants::THREAD_STATE::SLEEPING);
    std::this_thread::sleep_for(std::chrono::seconds(CMachine::getParsingQGap()));
  }

  CMachine::reportThreadStatus(thread_prefix, thread_code, constants::THREAD_STATE::STOPPED);
  CLog::log("Gracefully stopped thread(" + thread_prefix + thread_code + ") of loop Smart Pull From Parsing Q");
}
 */


//old_name_was parsePureBlock
pub fn parse_pure_block(
    _sender: &String,
    pq_type: &String,
    block: &Block,
    _connection_type: &String,
    _receive_date: &String,
) -> EntryParsingResult
{
    let err_msg: String;

    // DAG existance ancestors controlls
    let needed_blocks =
        cutils::array_diff(&block.m_block_ancestors, &get_cached_blocks_hashes());
    if needed_blocks.len() > 0
    {
        err_msg = format!("in order to parse 1 block({}) machine needs blocks({:?}) exist in DAG",
                          cutils::hash6c(&block.get_block_hash()), needed_blocks);
        dlog(
            &err_msg,
            constants::Modules::App,
            constants::SecLevel::TmpDebug);

        // TODO: maybe some reputation system to report diorder of neighbor
        return EntryParsingResult {
            m_status: false,
            m_should_purge_record: false,
            m_message: err_msg,
        };
    }

    // (b_status, b_should_purge_record) =
    let en_pa_res = block.block_general_controls();
    if !en_pa_res.m_status
    { return en_pa_res; }

    // general ancestors controlls
    let en_pa_res = ancestors_controls(pq_type, block);
    // (status, should_purge_record)
    if !en_pa_res.m_status
    { return en_pa_res; }

    return block.handle_received_block();

//  switch (pq_type) {

//    case iConsts.BLOCK_TYPES.FVote:
//        res = require('../../dag/floating-vote/floating-vote-handler').handleReceivedFVoteBlock({
//            sender,
//            block,
//            connection_type,
//            receive_date
//        });
//        break;
}

pub fn remove_from_parsing_q(clauses: ClausesT) -> bool
{
    return q_delete(
        C_PARSING_Q,
        clauses,
        false,
    );
}
