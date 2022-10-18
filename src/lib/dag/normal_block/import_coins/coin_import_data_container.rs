use std::collections::HashMap;
use crate::constants;
use crate::lib::block::document_types::basic_tx_document::basic_tx_document::BasicTxDocument;
use crate::lib::block::document_types::document::Document;
use crate::lib::custom_types::{CAddressT, CBlockHashT, CCoinCodeT, CDateT, CDocHashT, CMPAIValueT, QVDicT, SharesPercentT};
use crate::lib::transactions::basic_transactions::coins::coins_handler::{CoinInfo, CoinDetails};
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockDPCostTreasury
{
    pub m_cat: String,
    pub m_title: String,
    pub m_descriptions: String,
    pub m_coin: CCoinCodeT,
    pub m_value: CMPAIValueT,

// void reset();
}

impl BlockDPCostTreasury {
    pub fn new() -> Self
    {
        Self {
            m_cat: "".to_string(),
            m_title: "".to_string(),
            m_descriptions: "".to_string(),
            m_coin: "".to_string(),
            m_value: 0,
        }
    }

    #[allow(unused, dead_code)]
    pub fn reset(&mut self)
    {
        self.m_cat = "".to_string();
        self.m_title = "".to_string();
        self.m_descriptions = "".to_string();
        self.m_coin = "".to_string();
        self.m_value = 0;
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockDPCostBacker
{
    pub m_coin: CCoinCodeT,
    pub m_address: CAddressT,
    pub m_value: CMPAIValueT,
}

impl BlockDPCostBacker
{
    pub fn new() -> Self
    {
        Self {
            m_coin: "".to_string(),
            m_address: "".to_string(),
            m_value: 0,
        }
    }

    #[allow(dead_code, unused)]
    pub fn reset(&mut self)
    {
        self.m_coin = "".to_string();
        self.m_address = "".to_string();
        self.m_value = 0;
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockTreasuryLog
{
    pub m_title: String,
    pub m_cat: String,
    pub m_descriptions: String,
    pub m_coin: CCoinCodeT,
    pub m_value: CMPAIValueT,
    pub m_donate_coins_blocks: Vec<CoinDetails>,
}

impl BlockTreasuryLog {
    #[allow(dead_code, unused)]
    pub fn new() -> Self
    {
        Self {
            m_title: "".to_string(),
            m_cat: "".to_string(),
            m_descriptions: "".to_string(),
            m_coin: "".to_string(),
            m_value: 0,
            m_donate_coins_blocks: vec![],
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BlockAlterTreasuryIncome
{
    pub m_trx_hash: CDocHashT,
    pub m_coin: CCoinCodeT,
    pub m_value: CMPAIValueT,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SingleTrxDPCost
{
    pub m_coin: CCoinCodeT,
    pub m_address: CAddressT,
    pub m_value: CMPAIValueT,
    pub m_ref_creation_date: CDateT,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimeLockedDoc
{
    pub m_block_hash: CBlockHashT,
    pub m_doc_hash: CDocHashT,
    pub m_doc_pure_hash: CDocHashT,
    pub m_coin: CCoinCodeT,
    pub m_doc: BasicTxDocument,
    pub m_redeem_time: CDateT,
    pub m_clone_code: String,
    pub m_ref_creation_date: CDateT,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RawVote
{
    // are extracted from table trx_suspect_transactions
    pub m_voter: CAddressT,
    pub m_vote_date: CDateT,
    pub m_voting_coin: CCoinCodeT,
    pub m_logger_block: CBlockHashT,
    pub m_spender_block: CBlockHashT,
    pub m_spender_doc: CDocHashT,
    pub m_receive_order: i32,
    pub m_spend_date: CDateT,

    pub m_voter_percentage: SharesPercentT, // which is calculated
}

impl RawVote
{
    pub fn new() -> Self
    {
        Self {
            m_voter: "".to_string(),
            m_vote_date: "".to_string(),
            m_voting_coin: "".to_string(),
            m_logger_block: "".to_string(),
            m_spender_block: "".to_string(),
            m_spender_doc: "".to_string(),
            m_receive_order: 0,
            m_spend_date: "".to_string(),
            m_voter_percentage: 0.0,
        }
    }

    pub fn load_from_record(a_record: &QVDicT) -> Self
    {
        let mut out = Self::new();
        out.m_voting_coin = a_record["the_coin"].clone();
        out.m_voter = a_record["st_voter"].clone();
        out.m_vote_date = a_record["st_vote_date"].clone();
        out.m_logger_block = a_record["st_logger_block"].clone();
        out.m_spender_block = a_record["st_spender_block"].clone();
        out.m_spender_doc = a_record["st_spender_doc"].clone();
        out.m_receive_order = a_record["st_receive_order"].parse::<i32>().unwrap();
        out.m_spend_date = a_record["st_spend_date"].clone();
        if a_record.contains_key("voterPercentage")
        {
            out.m_voter_percentage = a_record["voterPercentage"].parse::<SharesPercentT>().unwrap();
        }
        out
    }
}

//old_name_was SBSCDS
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpenderBlockStatedCreationDate // Spender Block's Stated Creation Date Structure
{
    pub m_spend_date: CDateT,
    pub m_spend_doc: CDocHashT,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CoinVoterDocInfo
{
    pub m_coin_spend_date: CDateT,
    pub m_spend_receive_order: i32,
    // order of receiving spends for same refLoc
    pub m_voter_percentage: SharesPercentT,
    pub m_vote_date: CDateT,
}

// will be used to recognize if the 2 usage of coins have less than 6 hours different or not
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FirstSpenderInfo
{
    pub m_spend_doc: CDocHashT,
    pub m_spend_time: CDateT,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CoinVoterInfo
{
    //spend Orders Info By Spender Block's Stated Creation Date
    pub m_spend_orders_info_by_spender_block_stated_creation_date: HashMap<String, SpenderBlockStatedCreationDate>,
    pub m_docs_info: HashMap<CDocHashT, CoinVoterDocInfo>,
    pub m_receive_orders: HashMap<String, CDocHashT>,
    //  StringList m_spendTimes {};
//  FirstSpenderInfo m_firstSpenderInfo {}; // will be used to recognize if the 2 usage of coins have less than 6 hours different or not
    pub m_spends_less_than_6_hour_new: bool,
    pub m_spends_less_than_6_hours: bool,
    pub m_can_control_less_6_condition: bool,
}

impl CoinVoterInfo {
    pub fn new() -> Self
    {
        Self {
            m_spend_orders_info_by_spender_block_stated_creation_date: Default::default(),
            m_docs_info: Default::default(),
            m_receive_orders: Default::default(),
            m_spends_less_than_6_hour_new: false,
            m_spends_less_than_6_hours: false,
            m_can_control_less_6_condition: false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VoteData
{
    pub m_doc_hash: CDocHashT,
    pub m_inside_6_voters_count: i64,
    pub m_inside_6_votes_gain: f64,
    pub m_outside_6_voters_count: i64,
    pub m_outside_6_votes_gain: f64,
    pub m_vote_gain: f64,
    pub m_details: HashMap<CAddressT, CoinVoterDocInfo>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CoinOrderedSpender
{
    pub m_vote_data: VoteData,
    pub m_docs: Vec<CoinVoterDocInfo>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InOutside6hElm
{
    pub m_doc_hash: CDocHashT,
    pub m_votes: f64,
    pub m_voters: i64,  // voters count
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CoinAndPosition
{
    pub m_inside_6_hours: Vec<InOutside6hElm>,
    pub m_outside_6_hours: Vec<InOutside6hElm>,
    pub m_inside_total: f64,
    pub m_outside_total: f64,
}

impl CoinAndPosition
{
    pub fn new() -> Self
    {
        Self {
            m_inside_6_hours: vec![],
            m_outside_6_hours: vec![],
            m_inside_total: 0.0,
            m_outside_total: 0.0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SusVote
{
    pub m_valid: bool,
    pub m_action: String,
    pub m_voters: i64,
    pub m_votes: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidityCheck
{
    pub m_votes_dict: Vec<RawVote>,
    pub m_coins_and_voters_dict: HashMap<CCoinCodeT, HashMap<CAddressT, CoinVoterInfo>>,
    pub m_coins_and_ordered_spenders_dict: HashMap<CCoinCodeT, HashMap<u32, HashMap<CDocHashT, CoinOrderedSpender>>>,
    pub m_coins_and_positions_dict: HashMap<CCoinCodeT, HashMap<u32, CoinAndPosition>>,
    pub m_sus_vote_res: HashMap<CCoinCodeT, SusVote>,
    pub m_valid: bool,
    pub m_cloned: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SusInputDetection
{
    pub m_coin: CCoinCodeT,
    pub m_detection: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TheVote
{
    pub m_voter: CAddressT,
    pub m_shares_percent: SharesPercentT,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TheVotes
{
    pub m_votes: Vec<TheVote>,
    pub m_sum_percent: SharesPercentT,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CostPaymentStatus
{
    pub m_message: String,
    pub m_is_payed: bool,// = true;
}

impl CostPaymentStatus {
    #[allow(dead_code, unused)]
    pub fn new() -> Self
    {
        Self {
            m_message: "".to_string(),
            m_is_payed: true,
        }
    }
}

//old_name_was  UTXOImportDataContainer
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CoinImportDataContainer
{
    pub m_block_is_sus_case: bool,
    pub m_can_import_normally: bool,
    pub m_does_enough_sus_votes_exist: String,
    pub m_current_votes_percentage: SharesPercentT,
    pub m_raw_votes: Vec<RawVote>,
    pub m_to_cut_from_backer_fee: CMPAIValueT,
    pub m_block_dp_cost_backer_final: CMPAIValueT,
    pub m_to_cut_from_treasury_fee: CMPAIValueT,
    pub m_block_dp_cost_treasury_final: CMPAIValueT,
    pub m_block_has_income: bool,
    pub m_votes_dict: HashMap<CBlockHashT, TheVotes>,
    pub m_minimum_floating_vote: SharesPercentT, // = constants::MINIMUM_SUS_VOTES_TO_ALLOW_CONSIDERING_SUS_BLOCK;

    pub m_importable_coins: Vec<CoinInfo>,
    pub m_cut_ceased_trx_from_coins: Vec<CCoinCodeT>,
    pub m_supported_p4p: Vec<CDocHashT>,
    pub m_block_dp_cost_treasury: BlockDPCostTreasury,
    pub m_block_dp_cost_backer: BlockDPCostBacker,
    pub m_block_treasury_logs: Vec<BlockTreasuryLog>,

    pub m_p4p_docs: Vec<Document>,

    pub m_block_alter_treasury_incomes: HashMap<CAddressT, Vec<BlockAlterTreasuryIncome>>,

    pub m_trx_u_dict: HashMap<CDocHashT, Document>,
    pub m_map_u_trx_ref_to_trx_hash: HashMap<CDocHashT, CDocHashT>,
    pub m_map_u_trx_hash_to_trx_ref: HashMap<CDocHashT, CDocHashT>,

    pub m_a_single_trx_dp_cost: HashMap<CDocHashT, SingleTrxDPCost>,
    pub m_dp_cost_coin_codes: Vec<CCoinCodeT>,
    pub m_to_be_restored_coins: Vec<CoinDetails>,
    pub m_time_locked_docs: Vec<TimeLockedDoc>,

    // because trx is rejected or donated for double-spending
    pub m_must_not_import_trx_outputs: Vec<CDocHashT>,
    pub m_transactions_detection: HashMap<CDocHashT, String>,

    pub m_transactions_validity_check: HashMap<CDocHashT, ValidityCheck>,

    pub m_sus_inputs_detection: Vec<SusInputDetection>,
    pub m_rejected_transactions: HashMap<CDocHashT, Vec<CCoinCodeT>>,   // for each input refLoc must be inserted on record(even in one same transaction)

    pub m_map_u_referencer_to_referenced: HashMap<CDocHashT, CDocHashT>,
    pub m_map_u_referenced_to_referencer: HashMap<CDocHashT, CDocHashT>,

    pub m_output_time_locked_related_docs: HashMap<CDocHashT, bool>,    // TODO: implemet it ASAP

    // typedef String document_type;
    pub m_cost_payment_status: HashMap<String, HashMap<CDocHashT, CostPaymentStatus>>,

}

impl CoinImportDataContainer
{
    pub fn new() -> Self
    {
        Self {
            m_block_is_sus_case: false,
            m_can_import_normally: false,
            m_does_enough_sus_votes_exist: "".to_string(),
            m_current_votes_percentage: 0.0,
            m_raw_votes: vec![],
            m_to_cut_from_backer_fee: 0,
            m_block_dp_cost_backer_final: 0,
            m_to_cut_from_treasury_fee: 0,
            m_block_dp_cost_treasury_final: 0,
            m_block_has_income: false,
            m_votes_dict: Default::default(),
            m_minimum_floating_vote: constants::MINIMUM_SUS_VOTES_TO_ALLOW_CONSIDERING_SUS_BLOCK,
            m_importable_coins: vec![],
            m_cut_ceased_trx_from_coins: vec![],
            m_supported_p4p: vec![],
            m_block_dp_cost_treasury: BlockDPCostTreasury::new(),
            m_block_dp_cost_backer: BlockDPCostBacker::new(),
            m_block_treasury_logs: vec![],
            m_p4p_docs: vec![],
            m_block_alter_treasury_incomes: Default::default(),
            m_trx_u_dict: Default::default(),
            m_map_u_trx_ref_to_trx_hash: Default::default(),
            m_map_u_trx_hash_to_trx_ref: Default::default(),
            m_a_single_trx_dp_cost: Default::default(),
            m_dp_cost_coin_codes: vec![],
            m_to_be_restored_coins: vec![],
            m_time_locked_docs: vec![],
            m_must_not_import_trx_outputs: vec![],
            m_transactions_detection: Default::default(),
            m_transactions_validity_check: Default::default(),
            m_sus_inputs_detection: vec![],
            m_rejected_transactions: Default::default(),
            m_map_u_referencer_to_referenced: Default::default(),
            m_map_u_referenced_to_referencer: Default::default(),
            m_output_time_locked_related_docs: Default::default(),
            m_cost_payment_status: Default::default(),
        }
    }

    pub fn reset(&mut self)
    {
        self.m_block_is_sus_case = false;
        self.m_can_import_normally = false;
        self.m_does_enough_sus_votes_exist = "".to_string();
        self.m_current_votes_percentage = 0.0;
        self.m_raw_votes = vec![];
        self.m_to_cut_from_backer_fee = 0;
        self.m_block_dp_cost_backer_final = 0;
        self.m_to_cut_from_treasury_fee = 0;
        self.m_block_dp_cost_treasury_final = 0;
        self.m_block_has_income = false;
        self.m_votes_dict = Default::default();
        self.m_minimum_floating_vote = constants::MINIMUM_SUS_VOTES_TO_ALLOW_CONSIDERING_SUS_BLOCK;
        self.m_importable_coins = vec![];
        self.m_cut_ceased_trx_from_coins = vec![];
        self.m_supported_p4p = vec![];
        self.m_block_dp_cost_treasury = BlockDPCostTreasury::new();
        self.m_block_dp_cost_backer = BlockDPCostBacker::new();
        self.m_block_treasury_logs = vec![];
        self.m_p4p_docs = vec![];
        self.m_block_alter_treasury_incomes = Default::default();
        self.m_trx_u_dict = Default::default();
        self.m_map_u_trx_ref_to_trx_hash = Default::default();
        self.m_map_u_trx_hash_to_trx_ref = Default::default();
        self.m_a_single_trx_dp_cost = Default::default();
        self.m_dp_cost_coin_codes = vec![];
        self.m_to_be_restored_coins = vec![];
        self.m_time_locked_docs = vec![];
        self.m_must_not_import_trx_outputs = vec![];
        self.m_transactions_detection = Default::default();
        self.m_transactions_validity_check = Default::default();
        self.m_sus_inputs_detection = vec![];
        self.m_rejected_transactions = Default::default();
        self.m_map_u_referencer_to_referenced = Default::default();
        self.m_map_u_referenced_to_referencer = Default::default();
        self.m_output_time_locked_related_docs = Default::default();
        self.m_cost_payment_status = Default::default();
    }
}