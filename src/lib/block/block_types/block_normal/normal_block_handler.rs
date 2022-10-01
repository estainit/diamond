use serde_json::json;
use crate::{application, constants, cutils, dlog, machine};
use crate::lib::block::block_types::block::{Block, TransientBlockInfo};
use crate::lib::block::documents_in_related_block::transactions::transactions_in_related_block::{append_transactions, validate_transactions};
use crate::lib::block::node_signals_handler::get_machine_signals;
use crate::lib::block_utils::normalize_ancestors;
use crate::lib::custom_types::{CDateT, VString};
use crate::lib::dag::leaves_handler::{get_leave_blocks, has_fresh_leaves};
use crate::lib::machine::machine_buffer::retrieve_and_group_buffered_documents::retrieve_and_group_buffered_documents;
use crate::lib::messaging_protocol::dag_message_handler::set_maybe_ask_for_latest_blocks_flag;

//old_name_was createANormalBlock
pub fn create_a_normal_block(
    ancestors: &VString,
    creation_date: &CDateT,
    allowed_to_double_spend: bool /* test purpose */)
    -> (bool /* creating block status */, Block/* block */, bool/* should empty buffer */, String /* msg */)
//
{
    let msg: String;
    let mut should_reset_block_buffer: bool = true;
    println!("create A NormalBlock create A NormalBlock");

    let mut creation_date = creation_date.to_string();
    if creation_date == ""
    {
        creation_date = application().now();
    }

    // if !has_fresh_leaves()
    // {
    //     msg = format!("Machine hasn't fresh leaves, so it can not broadcast new block(Normal block)");
    //     dlog(
    //         &msg,
    //         constants::Modules::App,
    //         constants::SecLevel::Warning);
    //     set_maybe_ask_for_latest_blocks_flag(constants::YES);
    //     return (false, Block::new(), false, msg);
    // }

    let (status, mut block) = Block::load_block(&json!({
    "bType": constants::block_types::NORMAL,
    "bNet": constants::SOCIETY_NAME}));
    if !status {
        msg = format!("Failed in creation of Normal block!");
        dlog(
            &msg,
            constants::Modules::App,
            constants::SecLevel::Error);
        return (false, Block::new(), false, msg);
    }

    block.m_block_creation_date = creation_date;
    block.m_block_signals = get_machine_signals();
    block.m_block_documents = vec![];
    block.m_block_ext_root_hash = constants::HASH_ZEROS_PLACEHOLDER.to_string();
    block.m_block_ext_info = vec![];

    println!("status {}, block {:#?}", status, block);
    block.m_block_backer = machine().get_backer_address();
    println!("block.m_block_backer {:#?}", block.m_block_backer);

    // * the first step of creating a block is appending the transactions
    // * each block MUST have at least one transaction
    let mut transient_block_info: TransientBlockInfo = TransientBlockInfo::new();
    let (append_res, should_reset_block_buffer1, append_res_msg) =
        append_transactions(&mut block, &mut transient_block_info);
    should_reset_block_buffer &= should_reset_block_buffer1;
    if !append_res
    {
        return (append_res, block, should_reset_block_buffer, append_res_msg);
    }
    println!("kkkkk  k 5");

    let (grouping_res, should_reset_block_buffer2, grouping_res_msg) =
        retrieve_and_group_buffered_documents(&mut block, &mut transient_block_info);
    should_reset_block_buffer &= should_reset_block_buffer2;
    if !grouping_res
    {
        return (grouping_res, block, should_reset_block_buffer, grouping_res_msg);
    }
    println!("kkkkk  k 6");


    // control if each trx is referenced to only one Document?
    let mut tmp_trxs: VString = vec![];
    for a_trx_ref in &transient_block_info.m_map_trx_ref_to_trx_hash.keys().cloned().collect::<VString>()
    {
        tmp_trxs.push(transient_block_info.m_map_trx_ref_to_trx_hash[a_trx_ref].clone());
    }
    if tmp_trxs.len() != cutils::array_unique(&tmp_trxs).len()
    {
        msg = format!(
            "Creating new block, same transaction is used as a ref for different docs! {:?}",
            transient_block_info.m_map_trx_ref_to_trx_hash);
        dlog(
            &msg,
            constants::Modules::App,
            constants::SecLevel::Error);
        return (false, block, false, msg);
    }
    println!("kkkkk  k 7");

    // TODO: important! currently the order of adding documents to block is important(e.g. polling must be added before proposalsand pledges)
    // improve the code and remove this dependency

    // * add free Documents(if exist)
    // * since block size controlling is not implemented completaly, it is better to put this part at the begening of appending,
    // * just in order to be sure block has enough capacity to include entire docs in buffer
    // let(free_append_res, free_should_reset_block_buffer, free_append_res_msg) =
    // appendFreeDocsToBlock(
    // block,
    // transient_block_info);
    // should_reset_block_buffer &= free_should_reset_block_buffer;
    // if (!free_append_res)
    // return {false, block, should_reset_block_buffer, free_append_res_msg};

    //  * add vote-ballots(if exist)
    // auto[ballot_append_res, ballot_should_reset_block_buffer, ballot_append_res_msg] = BallotsInRelatedBlock::appendBallotsToBlock(
    //   block,
    //   transient_block_info);
    // should_reset_block_buffer &= ballot_should_reset_block_buffer;
    // if (!ballot_append_res)
    //   return {false, block, should_reset_block_buffer, ballot_append_res_msg};

    //  * add iName-reg-req(if exist)
    // auto[iname_append_res, iname_should_reset_block_buffer, iname_append_res_msg] = INamesInRelatedBlock::appendINamesToBlock(
    //   block,
    //   transient_block_info);
    // should_reset_block_buffer &= iname_should_reset_block_buffer;
    // if (!iname_append_res)
    //   return {false, block, should_reset_block_buffer, iname_append_res_msg};

    //  * add bind iName(if exist)
    // auto[iname_pgp_bind_append_res, iname_pgp_bind_should_reset_block_buffer, iname_pgp_bind_append_res_msg] = INamesBindsInRelatedBlock::appendINameBindsToBlock(
    //   block,
    //   transient_block_info);
    // should_reset_block_buffer &= iname_pgp_bind_should_reset_block_buffer;
    // if (!iname_pgp_bind_append_res)
    //   return {false, block, should_reset_block_buffer, iname_pgp_bind_append_res_msg};

    //   * add msg to iName(if exist)
    //  let addInameMsgRes = iNameMsgsInRelatedBlock.appendINameMsgsToBlock(appendArgs);
    //  clog.app.info(`addInameMsgRes: ${utils.stringify(addInameMsgRes)}`);
    //  if (addInameMsgRes.err != false) {
    //      clog.app.error(`addInameMsgRes ${addInameMsgRes.msg}`);
    //      return addInameMsgRes;
    //  }
    //  block = addInameMsgRes.block;
    //  transient_block_info.m_block_documents_hashes = addInameMsgRes.docsHashes;
    //  externalInfoHashes = addInameMsgRes.externalInfoHashes;
    //  if (addInameMsgRes.addedDocs > 0)
    //      console.log(`\n\nblockAfterAdding iName-register: ${utils.stringify(block)}`);

    //  * add admPolling(if exist)
    // auto[adm_polling_append_res, adm_polling_should_reset_block_buffer, adm_polling_append_res_msg] = AdministrativePollingsInRelatedBlock::appendAdmPollingsToBlock(
    //   block,
    //   transient_block_info);
    // should_reset_block_buffer &= adm_polling_should_reset_block_buffer;
    // if (!adm_polling_append_res)
    //   return {false, block, should_reset_block_buffer, adm_polling_append_res_msg};


    //  /**
    //   * add ReqForRelRes(if exist)
    //   * TODO: move it to appendAdmPollingsToBlock
    //   */
    //  let addRelCoinsRes = reqRelRessInRelatedBlock.appendReqRelResToBlock(appendArgs);
    //  if (addRelCoinsRes.err != false) {
    //      clog.app.error(`addRelCoinsRes ${addRelCoinsRes.msg}`);
    //      return addRelCoinsRes;
    //  }
    //  block = addRelCoinsRes.block;
    //  transient_block_info.m_block_documents_hashes = addRelCoinsRes.docsHashes;
    //  externalInfoHashes = addRelCoinsRes.externalInfoHashes;
    //  if (addRelCoinsRes.addedDocs > 0)
    //      console.log(`\n\nblockAfterAdding ReqRelRes: ${utils.stringify(block)}`);

    //  * add polling(if exist) except pollings for proposal which are generating automatically
    // auto[polling_append_res, polling_should_reset_block_buffer, polling_append_res_msg] = PollingsInRelatedBlock::appendPollingsToBlock(
    //   block,
    //   transient_block_info);
    //   should_reset_block_buffer &= polling_should_reset_block_buffer;
    //   if (!polling_append_res)
    //     return {false, block, should_reset_block_buffer, polling_append_res_msg};

    //  * add proposals(if exist)
    // auto[proposal_append_res, proposal_should_reset_block_buffer, proposal_append_res_msg] = ProposalsInRelatedBlock::appendProposalsToBlock(
    //   block,
    //   transient_block_info);
    // should_reset_block_buffer &= proposal_should_reset_block_buffer;
    // if (!proposal_append_res)
    //   return {false, block, should_reset_block_buffer, proposal_append_res_msg};

    //  * add pledges(if exist)
    // auto[pledge_append_res, pledge_should_reset_block_buffer, pledge_append_res_msg] = PledgesInRelatedBlock::appendPledgesToBlock(
    //   block,
    //   transient_block_info);
    // should_reset_block_buffer &= pledge_should_reset_block_buffer;
    // if (!pledge_append_res)
    //   return {false, block, should_reset_block_buffer, pledge_append_res_msg};

    //   * add redeem pledges(if exist)
    //  let addClosePledgesRes = closePledgeInRelatedBlock.appendClosePledgesToBlock(appendArgs);
    //  clog.app.info(`addClosePledgesRes: ${utils.stringify(addClosePledgesRes)}`);
    //  if (addClosePledgesRes.err != false) {
    //      console.log(`addClosePledgesRes ${addClosePledgesRes.msg}`);
    //      return addClosePledgesRes;
    //  }
    //  block = addClosePledgesRes.block;
    //  transient_block_info.m_block_documents_hashes = addClosePledgesRes.docsHashes;
    //  externalInfoHashes = addClosePledgesRes.externalInfoHashes;
    //  if (addClosePledgesRes.addedDocs > 0)
    //      console.log(`\n\nblockAfterAdding close-pledges: ${utils.stringify(block)}`);


    // retrieve wiki page
    // retrieve demos text
    // retrieve ...


    dlog(
        &format!("Creating the NORMAL block which has {} document(s)",
                 transient_block_info.m_block_documents_hashes.len()
        ),
        constants::Modules::App,
        constants::SecLevel::Warning);

    println!("kkkkk  k 8");

    let (doc_status, doc_root_hash) = block.calc_documents_root_hash();
    if !doc_status
    { return (false, block, false, "Failed in creation documents root hash".to_string()); }
    block.m_block_documents_root_hash = doc_root_hash;
    println!("kkkkk  k 9");

    let (ext_status, ext_root_hash) = block.calc_block_ext_root_hash();
    if !ext_status
    { return (false, block, false, "Failed in creation documents ext root hash".to_string()); }
    block.m_block_ext_root_hash = ext_root_hash;

    println!("kkkkk  k 10");

    if ancestors.len() > 0
    {
        block.m_block_ancestors = ancestors.clone();
    } else {
        block.m_block_ancestors = cutils::array_add(
            &get_leave_blocks(&"".to_string()).keys().cloned().collect::<VString>(),
            &block.get_ancestors());
    }
    println!("kkkkk  k 11");

    block.m_block_ancestors = normalize_ancestors(&block.get_ancestors());
    if transient_block_info.m_pre_requisites_ancestors.len() > 0
    {
        dlog(
            &format!(
                "The outgoing block has to has some ancestors because of related polling creation block(s): {}",
                transient_block_info.m_pre_requisites_ancestors.join(", ")
            ),
            constants::Modules::App,
            constants::SecLevel::Info);
        block.m_block_ancestors = cutils::array_unique(
            &cutils::array_add(&block.get_ancestors(),
                               &transient_block_info.m_pre_requisites_ancestors));
    }
    block.m_block_ancestors.sort();
    dlog(
        &format!(
            "The NORMAL block will be descendant of these ancestors: {:#?}", block.get_ancestors()
        ),
        constants::Modules::App,
        constants::SecLevel::Info);

    println!("kkkkk  k 12");

    // fill in the bloc.m_block_ext_info
    block.fill_in_block_ext_info();

    block.calc_and_set_block_length();
    block.set_block_hash(&block.calc_block_hash());

    let transient_block_info2: TransientBlockInfo = block.group_docs_of_block(&constants::stages::CREATING.to_string());
    if !transient_block_info2.m_status
    { return (false, block, false, "Failed in group Docs Of Block".to_string()); }

    msg = format!("Final block, before transactions validate: {}", block.safe_stringify_block(false));
    dlog(
        &msg,
        constants::Modules::App,
        constants::SecLevel::TmpDebug);
    println!("kkkkk  k 13");

    // re-validate block transactions
    if !allowed_to_double_spend
    {
        let (status, _is_sus_block, validate_msg, _double_spends) =
            validate_transactions(&block, &constants::stages::CREATING.to_string());
        if !status
        {
            return (false, block, false, format!("Failed in validate transactions. {}", validate_msg));
        }
    }

    println!("kkkkk  k 14");

    let b_id = block.get_block_identifier();
    println!("kkkkk  k 15");
    return (
        true,
        block,
        should_reset_block_buffer,
        format!("Normal block created. {}", b_id)
    );
}


