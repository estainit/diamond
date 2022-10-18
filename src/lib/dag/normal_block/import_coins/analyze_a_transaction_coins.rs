use crate::{constants, cutils, dlog};
use crate::lib::block::block_types::block::Block;
use crate::lib::block::document_types::document::Document;
use crate::lib::custom_types::VString;
use crate::lib::dag::normal_block::import_coins::coin_analyzer::{do_sus_treatments, extract_doc_importable_coins};
use crate::lib::dag::normal_block::import_coins::coin_import_data_container::{CoinImportDataContainer, SingleTrxDPCost};
use crate::lib::transactions::basic_transactions::coins::suspect_trx_handler::{check_doc_validity, get_sus_info_by_doc_hash, retrieve_voter_percentages};

//old_name_was analyzeATransactionCoins
pub fn analyze_a_transaction_coins(
    block_inspect_container: &mut CoinImportDataContainer,
    block: &Block,
    doc: &Document)
{
    let doc_hash = doc.get_doc_hash();


    // * find DPCost payments, to not importing in UTXOs
    // * and also in case of doublespending removing these incomes from blockTotalTrxFee & treauryTotalTrxFeeIncome

    for inx in doc.get_dpis()
    {
        let an_out = &doc.get_outputs()[*inx as usize];
        if (doc.m_doc_type != constants::document_types::DATA_AND_PROCESS_COST_PAYMENT)
            && (an_out.m_address == block.get_block_backer())
        {
            block_inspect_container.m_a_single_trx_dp_cost.insert(doc_hash.clone(), SingleTrxDPCost {
                m_coin: cutils::pack_coin_code(&doc_hash, *inx),
                m_address: an_out.m_address.clone(),
                m_value: an_out.m_amount,
                m_ref_creation_date: block.m_block_creation_date.clone(),
            });
        }
    }

    block_inspect_container.m_transactions_detection.insert(doc_hash.clone(), "Normal".to_string());


    if constants::OUTPUT_TIME_LOCK_IS_ENABLED
    {

        //    // retrieve latest redeemTimes(if exist)
        //    let { docInputs, docMaxRedeem } = outputTimeLockHandler.getDocInputsAndMaxRedeem({
        //        doc,
        //        docExtInfo
        //    });
        //    let isOutputTimeLockedRelatedDoc = outputTimeLockHandler.isTimeLockRelated(docInputs);
        //    if (isOutputTimeLockedRelatedDoc)
        //        block_inspect_container.oTimeLockedRelatedDocs[doc.hash] = outputTimeLockHandler.isTimeLockRelated(docInputs);
        //    clog.trx.info(`doc(${utils.hash6c(doc.hash)}) is time Locked Related Doc(${isOutputTimeLockedRelatedDoc}) inputs(${docInputs.map(x => iutils.shortCoinRef(x))})`);
        //    console.log(`doc(${utils.hash6c(doc.hash)}) is time Locked Related Doc(${isOutputTimeLockedRelatedDoc}) inputs(${docInputs.map(x => iutils.shortCoinRef(x))})`);
        //    if ((0 < docMaxRedeem) || isOutputTimeLockedRelatedDoc) {
        //        // save redeem info to apply it on right time, and return
        //        let refLocBlocks = get_coins_generation_info_via_sql(docInputs);
        //        for (let refLoc of docInputs) {
        //            block_inspect_container.timeLockedDocs.push({
        //                blockHash: block.blockHash,
        //                docHash: doc.hash,
        //                pureHash: trxHashHandler.getPureHash(doc),
        //                refLoc,
        //                doc,
        //                redeemTime: utils.minutesAfter(docMaxRedeem + iConsts.getCycleByMinutes(), block.creation Date),
        //                docMaxRedeem,
        //                cloneCode: block.cycle,
        //                refCreation Date: refLocBlocks[refLoc].coinGenCreation Date
        //            });
        //        }
        //        let redeemTime = utils.minutesAfter(docMaxRedeem + iConsts.getCycleByMinutes(), block.creation Date);
        //        console.log(`doc(${utils.hash6c(doc.hash)}) created On(${block.creation Date}) MaxRedeem: ${utils.stringify(docMaxRedeem)} redeemTime(${redeemTime})`);
        //        clog.trx.info(`doc(${utils.hash6c(doc.hash)}) created On(${block.creation Date}) MaxRedeem: ${utils.stringify(docMaxRedeem)} redeemTime(${redeemTime})`);

        //        // do nothing more with this document
        //        block_inspect_container.transactionsDetection[doc.hash] = 'timeLocked';
        //        return;
        //    }
    }

    if !block_inspect_container.m_block_is_sus_case
    {
        block_inspect_container.m_can_import_normally = true;
        extract_doc_importable_coins(
            block_inspect_container,
            block,
            doc);

        return;
    }

    // control if there is atleast one susVote for this document?
    let (all_coins_are_valid, mut raw_votes) = get_sus_info_by_doc_hash(&doc_hash);
    if all_coins_are_valid
    {
        dlog(
            &format!("doc({}) of block{}, is not sus (even if block is blockIsSusCase) allCoinsOfDocAreValid",
                     cutils::hash8c(&doc_hash),
                     block.get_block_identifier(),
            ),
            constants::Modules::Trx,
            constants::SecLevel::Info);

        block_inspect_container.m_can_import_normally = true;
        block_inspect_container.m_transactions_detection.insert(doc_hash.clone(), "There is no sus vote for doc".to_string());
        extract_doc_importable_coins(
            block_inspect_container,
            block,
            doc);

        return;
    }

    let (_status, updated_raw_votes) = retrieve_voter_percentages(&mut raw_votes);
    block_inspect_container.m_raw_votes = updated_raw_votes;

    // analyze votes and decide about dobiuos transactions
    check_doc_validity(
        block_inspect_container,
        &doc_hash,
        true,
        true);
//  block_inspect_container.m_transactions_validity_check[doc_hash] = validity;

    block_inspect_container.m_can_import_normally = false;
    if block_inspect_container.m_transactions_validity_check[&doc_hash].m_cloned == "cloned".to_string()
    {
        // the document/transaction is cloned, so can import outputs regularly
        dlog(
            &format!(
                "Recognized a cloned trx{} in block{}",
                doc.get_doc_identifier(),
                block.get_block_identifier()
            ),
            constants::Modules::Trx,
            constants::SecLevel::TmpDebug);
        block_inspect_container.m_can_import_normally = true;
    }

    if (block_inspect_container
        .m_transactions_validity_check[&doc_hash]
        .m_sus_vote_res.keys().len() == 1)

        &&

        (block_inspect_container
            .m_transactions_validity_check[&doc_hash]
            .m_sus_vote_res[
            &block_inspect_container
                .m_transactions_validity_check[&doc_hash]
                .m_sus_vote_res.keys().cloned().collect::<VString>()[0]
            ].m_valid == true)
    {
        dlog(
            &format!(
                "Only one sus-input exist and it is valid trx({}) in block({}) ",
                cutils::hash8c(&doc_hash),
                block.get_block_identifier()
            ),
            constants::Modules::Trx,
            constants::SecLevel::Info);
        block_inspect_container.m_can_import_normally = true;
        block_inspect_container.m_transactions_detection.insert(
            doc_hash.clone(),
            "Only one sus-input exist and it is valid".to_string());
    }

    if block_inspect_container.m_can_import_normally
    {
        // so doc can be imported normally
        extract_doc_importable_coins(
            block_inspect_container,
            block,
            doc);
        return;
    }

    // transaction needs special treatment.
    // no out put of this transaction will be imported to UTXOs
    // depends on vote result, transaction inputs can be
    // 1. donated to treasury,
    // 2. restored in UTXOs,
    // 3. denied
    do_sus_treatments(
        block_inspect_container,
        block,
        doc);
    return;
}

