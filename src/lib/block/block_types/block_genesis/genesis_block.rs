pub mod b_genesis {
    use std::collections::HashMap;
    use postgres::types::ToSql;
    use crate::{application, ccrypto, constants, dlog};
    use crate::lib::block::block_types::block::Block;
    use crate::lib::block::document_types::proposal_document::{ProposalDocument};
    use crate::lib::block::document_types::document::Document;
    use crate::lib::custom_types::{CBlockHashT, CDateT, JSonObject};
    use crate::lib::database::abs_psql::{simple_eq_clause};
    use crate::lib::services::dna::dna_handler::insert_a_share;
    use crate::lib::services::polling::polling_handler::update_polling;

    //old_name_was initGenesisBlock
    pub fn init_genesis_block() -> (bool, String)
    {
        let mut block: Block = Block::new();
        block.m_block_ancestors = vec![",".to_string()];
        block.m_block_type = constants::block_types::GENESIS.to_string();
        block.m_block_creation_date = application().launch_date();

        let mut doc = Document::new();
        let back_in_time = (application().get_cycle_by_minutes() * 2) as u64;
        let launch_date = application().launch_date();
        let proposal_creation_date: CDateT = application().minutes_before(
            back_in_time,
            &launch_date);
        doc.m_doc_type = constants::document_types::PROPOSAL.to_string();
        doc.m_doc_class = "Classic".to_string();
        doc.m_doc_title = "fair effort, fair gain, win win win".to_string();
        doc.m_doc_creation_date = proposal_creation_date.clone();
        doc.m_doc_tags = "initialize, contributions, shares".to_string();
        doc.m_doc_comment = "Imagine all the people living life in peace".to_string();

        let mut proposal = ProposalDocument::new();
        proposal.m_project_hash = ccrypto::convert_title_to_hash(&"imagine".to_string());
        proposal.m_help_hours = constants::INITIAL_SHARES;
        proposal.m_help_level = 1;
        proposal.m_shares = constants::INITIAL_SHARES as i64;
        proposal.m_contributor_account = constants::HU_SHARE_ADDRESS.to_string();
        proposal.m_approval_date = proposal_creation_date;
        proposal.m_polling_profile = "Basic".to_string();
        proposal.m_voting_timeframe = 24.0;
        proposal.m_votes_yes = constants::INITIAL_SHARES as i64;
        proposal.m_votes_abstain = 0;
        proposal.m_votes_no = 0;

        doc.m_if_proposal_doc = proposal;
        doc.m_doc_hash = doc.m_if_proposal_doc.calc_doc_hash(&doc);

        block.m_block_documents_root_hash = doc.get_doc_hash(); // since the genesis block has only 1 document // "fb20e4323d695db7728eabcf3a44a1c0516d23362622fa3093e7cf887ef88396";
        block.m_block_documents.push(doc);
        block.m_block_hash = block.calc_block_hash();//"7a2e58190452d3764afd690ffd13a1360193fdf30f932fc1b2572e834b72c291";
        block.m_block_backer = constants::HU_SHARE_ADDRESS.to_string();
        block.m_block_confidence = 99.99;

        let (status, msg) = block.add_block_to_dag();
        if !status
        {
            let msg = format!("Failed in add genesis block to DAG. {}", msg);
            dlog(
                &msg,
                constants::Modules::App,
                constants::SecLevel::Error);

            return (false, msg);
        }

        // set initial shares
        return init_shares(&block);
    }

    /*
    GenesisBlock::GenesisBlock()
    {
        m_block_descriptions = "Imagine all the people sharing all the world";
        m_block_confidence = 100.0;
    }

*/
    pub fn genesis_set_by_json_obj(block: &mut Block, _obj: &JSonObject) -> bool
    {
        // custom settings for Genesis block
        block.m_block_type = constants::block_types::GENESIS.to_string();
        return true;
    }

    //old_name_was initShares
    pub fn init_shares(block: &Block) -> (bool, String)
    {
        let back_in_time = (5 * application().get_cycle_by_minutes()) as u64;
        let conclude_date = application().launch_date();
        let start_voting_date: String = application().minutes_before(
            back_in_time,
            &conclude_date);

        let initial_proposal: &Document = &block.m_block_documents[0];
        let proposal_hash: String = initial_proposal.get_doc_hash();

        // update proposal status in DB
        let pr_approved = constants::YES.to_string();
        let update_values: HashMap<&str, &(dyn ToSql + Sync)> = HashMap::from([
            ("pr_start_voting_date", &start_voting_date as &(dyn ToSql + Sync)),
            ("pr_conclude_date", &conclude_date as &(dyn ToSql + Sync)),
            ("pr_approved", &pr_approved as &(dyn ToSql + Sync)),
        ]);

        let c1 = simple_eq_clause("pr_hash", &proposal_hash);
        ProposalDocument::update_proposal(
            &update_values,
            vec![c1],
            false);

        // conclude the polling
        let pll_end_date = application().minutes_after(
            36 * 60,
            &start_voting_date.clone());
        let polling_upd_values: HashMap<&str, &(dyn ToSql + Sync)> = HashMap::from([
            ("pll_start_date", &start_voting_date as &(dyn ToSql + Sync)),
            ("pll_end_date", &pll_end_date as &(dyn ToSql + Sync)),
            ("pll_status", &constants::CLOSE as &(dyn ToSql + Sync)),
            ("pll_ct_done", &constants::YES as &(dyn ToSql + Sync)),
        ]);
        let c1 = simple_eq_clause("pll_ref", &proposal_hash);
        update_polling(
            &polling_upd_values,
            vec![c1],
            false);

        // also insert in db DNA initiate shares
        return insert_a_share(initial_proposal);
    }

    /*
    JSonObject GenesisBlock::export_block_to_json(const bool ext_info_in_document) const
    {
      JSonObject block = Block::export_block_to_json(ext_info_in_document);
      return block;
        }

    String GenesisBlock::safe_stringify_block(const bool ext_info_in_document) const
    {
      JSonObject block = export_block_to_json(ext_info_in_document);

      // recaluculate block final length
      String tmp_stringified = cutils::serializeJson(block);
      block["bLen"] = cutils::padding_length_value(tmp_stringified.len());

      String out = cutils::serializeJson(block);
      CLog::log("Safe sringified block(Genesis) Block(" + cutils::hash8c(m_block_hash) + ") length(" + String::number(out.len()) + ") the block: " + out, "app", "trace");

      return out;
    }
    */

    //old_name_was getBlockHashableString
    pub fn genesis_get_block_hashable_string(block: &Block) -> String
    {
        // in order to have almost same hash! we sort the attributes alphabetically
        let hashable_block: String = format!(
            "bAncestors:[],bCDate:{},bDocsRootHash:{},bLen:{},bNet:{},bType:{},bVer:{}",
            block.m_block_creation_date,
            block.m_block_length,
            block.m_block_net,
            block.m_block_type,
            block.m_block_version,
            block.m_block_documents_root_hash, // note that we do not put the docsHash directly in block hash, instead using docsHash-merkle-root-hash
        );
        return hashable_block;
    }

    //old_name_was calcBlockHash
    pub fn genesis_calc_block_hash(block: &Block) -> CBlockHashT
    {
        let hashable_block: String = genesis_get_block_hashable_string(block);

        // clonedTransactionsRootHash: block.clonedTransactionsRootHash,
        // note that we do not put the clonedTransactions directly in block hash, instead using clonedTransactions-merkle-root-hash

        let block_hash: CBlockHashT = ccrypto::keccak256(&hashable_block);
        dlog(
            &format!("The Genesis! block({block_hash}) hashable: {hashable_block} \n"),
            constants::Modules::App,
            constants::SecLevel::Info);

        return block_hash;
    }
}
