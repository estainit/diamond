use crate::lib::custom_types::{BlockLenT, CMPAIValueT, DocLenT, SharesPercentT};

#[allow(dead_code, unused)]
pub enum Modules {
    App,
    CB,
    Trx,
    Sql,
    Sec,
}

#[allow(dead_code, unused)]
pub enum SecLevel {
    Debug,
    Trace,
    TmpDebug,
    Info,
    Warning,
    Error,
    Fatal,
}

pub const SOCIETY_NAME: &str = "im";

//bech32 part
pub const BECH32_ADDRESS_VER: &str = "0";
pub const TRUNCATE_FOR_BECH32_ADDRESS: u8 = 32;

// to be compatible with JS clients: Number.MAX_SAFE_INTEGER
pub const MAX_COINS_AMOUNT: CMPAIValueT = 24_000_000 * 1_000_000_000;

pub const STANDARD_CYCLE_BY_MINUTES: u32 = 720;


/*
  const bool BROADCAST_TO_NEIGHBOR_OF_NEIGHBOR = true;    // if machine allowed to publish email of neightbors of neighbors to hei neighbors?
*/
pub const SUPPORTS_CLONED_TRANSACTION: bool = false;    // after writing a tons of unittests can activate this feature

// after writing a tons of unittests can activate this feature
pub const SUPPORTS_P4P_TRANSACTION: bool = false;
pub const LAUNCH_YEAR: u16 = 2022;
// every twelve years the coinbse divided to 2
pub const HALVING_PERIOD: u16 = 1;
// every twelve years the coinbse divided to 2

pub const COIN_ISSUING_INIT_EXPONENT: u8 = 11;
// the power of 2 for minting new coins
pub const COINBASE_MATURATION_CYCLES: u8 = 2;

#[allow(unused, dead_code)]
pub const ONE_MILLION: u64 = 1_000_000;
pub const ONE_BILLION: u64 = 1_000_000_000;
pub const MONEY_MAX_DIVISION: u64 = ONE_BILLION;

pub const CLIENT_VERSION: &str = "0.2.0";
// can be sqlite or psql
pub const DATA_BASAE_AGENT: &str = "psql";

#[allow(unused, dead_code)]
pub const DEFAULT_CONTRIBUTE_LEVEL: u8 = 6;

// at least 4 cycle must be a gap between pay to treasury and dividing to shareholders
pub const TREASURY_MATURATION_CYCLES: u8 = 4;
pub const DUMPER_INDENT: &str = "  ";
pub const DO_HARD_COPY_DAG_BACKUP: bool = true;
// it creates a DAG backup in sense of blocks

// it creates a local copy of what your machine will send via email
/*

  const uint16_t GAP_EMAIL_POP = 300; // default is 5 minutes = 300 second

  const uint64_t STANDARD_CYCLE_BY_MINUTES = static_cast<uint64_t>(720);


  pub const JS_FAKSE_NULL = "__js__null__";
  // the maximum time in which local machine wait to attain most possible confidence coinbase block
  // best value is 3/4 of one coinbase cycle
  */

#[allow(unused, dead_code)]
pub const COINBASE_FLOOR_TIME_TO_RECORD_IN_DAG: f64 = 3.0 / 5.0;
#[allow(unused, dead_code)]
pub const MINIMUM_SHARES_IF_IS_NOT_SHAREHOLDER: f64 = 0.0000000001;
#[allow(unused, dead_code)]
pub const SHARE_MATURITY_CYCLE: u8 = 2;

#[allow(unused, dead_code)]
pub const CONTRIBUTION_APPRECIATING_PERIOD: u16 = 100;

// 2 cycle after inserting a share in DB, coinbase will include the newly recorded share in it's dividend
/*
  // to avoid spam proposals, there is a cost equal to 30 days of potential income of the proposal
  // this cost goes to treasury and divide between shareholders

  // in order to put a proposal on voting process, requester has to pay one month income * 3 to treasury
  const uint PROPOSAL_APPLY_COST_SCALE = 3;

  const uint32_t ONE_MINUTE_BY_MILISECOND = 60000;

*/

//  * hopefully after 10 cycle(5 days) all DAG branches are merged and
//  * ALL transactions from 5 days ago are visible by leaves,
//  * so we do not need to mantain spend coins in DB
pub const KEEP_SPENT_COINS_BY_CYCLE: u8 = 10;

pub mod sig_hashes
{
    pub const ALL: &str = "ALL";
    pub const NONE: &str = "NONE";
    // these have conflict with BIP69, that's why we need custom SIGHASH
    // 'SINGLE': 'SINGLE',
    // 'ALL|ANYONECANPAY': 'ALL|ANYONECANPAY',
    // 'NONE|ANYONECANPAY': 'NONE|ANYONECANPAY',
    // 'SINGLE|ANYONECANPAY': 'SINGLE|ANYONECANPAY',
    #[allow(unused, dead_code)]
    pub const CUSTOM: &str = "CUSTOM"; // TODO: implement it in order to have ability to sign some random inputs & outputs
}

// if machine missed more than this number, does not allowed to issue a coinbase block
pub const MAX_TOLERATED_MISS_BLOCKS: u8 = 5;
// can be disabled for the sake of performance
pub const SUPER_CONTROL_COINS_DOUBLE_SPENDING: bool = true;
// can be disabled for the sake of performance
pub const SUPER_CONTROL_COINS_BACK_TO_COINBASE_MINTING: bool = true;
pub const SUPER_CONTROL_SHOULD_CONTROL_SIGNATURES_AS_WELL: bool = true;
// can be disabled for the sake of performance
pub const LOG_SUPER_VALIDATE_DEBUG: bool = true;
/*


  const bool DECODE_ALL_FREE_POST_FILES = true; // TODO: improve it to support different file types & security levels
*/
#[allow(dead_code, unused)]
pub const SIGN_MSG_LENGTH: u8 = 32;
#[allow(dead_code, unused)]
pub const FLOAT_LENGTH: u8 = 11;
#[allow(dead_code, unused)]
pub const LEN_PROP_LENGTH: u8 = 7;
#[allow(dead_code, unused)]
pub const LEN_PROP_PLACEHOLDER: &str = "0000000";
#[allow(dead_code, unused)]
pub const HASH_ZEROS_PLACEHOLDER: &str = "0000000000000000000000000000000000000000000000000000000000000000";

#[allow(unused, dead_code)]
pub const TRANSACTION_PADDING_LENGTH: DocLenT = 100;
pub const TRANSACTION_MINIMUM_LENGTH: DocLenT = 375;    // smallest transaction has 375 charachter length

/**
 * this "MAX_BLOCK_LENGTH_BY_CHAR" is a single max size of single message after adding headers and encrypting
 * it means real payload of a block (or real GQL messegaes size) must be roughly around 80% - 90%
 */
pub const MAX_BLOCK_LENGTH_BY_CHAR: BlockLenT = 10_000_000 * 10;


// 10 Mega Byte is actually block size
pub const MAX_DOC_LENGTH_BY_CHAR: DocLenT = 10 * 1024 * 1024 * 9;
#[allow(unused, dead_code)]
pub const MAX_DP_COST_PAY_DOCUMENT_SIZE: DocLenT = 600;
#[allow(unused, dead_code)]
pub const MAX_FULL_DAG_DOWNLOAD_RESPONSE_LENGTH_BY_CHAR: u64 = 1500000;

pub mod stages
{
    pub const CREATING: &str = "Creating";
    #[allow(unused, dead_code)]
    pub const REGENERATING: &str = "Regenerating";
    #[allow(unused, dead_code)]
    pub const VALIDATING: &str = "Validating";
}

#[allow(dead_code, unused)]
pub mod thread_state
{
    #[allow(dead_code, unused)]
    pub const RUNNING: &str = "RUNNING";
    #[allow(dead_code, unused)]
    pub const SLEEPING: &str = "SLEEPING";
    #[allow(dead_code, unused)]
    pub const STOPPED: &str = "STOPPED";
}

pub mod coin_selecting_method
{
    #[allow(unused, dead_code)]
    pub const PRECISE: &str = "PRECISE";
    #[allow(unused, dead_code)]
    pub const BIGGER_FIRST: &str = "BIGGER_FIRST";
    #[allow(unused, dead_code)]
    pub const SMALLER_FIRST: &str = "SMALLER_FIRST";
    #[allow(unused, dead_code)]
    pub const RANDOM: &str = "RANDOM";
}


#[allow(unused, dead_code)]
pub const DEFAULT_LANG: &str = "eng";
#[allow(unused, dead_code)]
pub const DEFAULT_VERSION: &str = "0.0.0";
pub const WRAP_SAFE_CONTENT_VERSION: &str = "0.0.0";

pub const NL: &str = "\n";
pub const TAB: &str = "\t";

#[allow(unused, dead_code)]
pub const ALL: &str = "All";
#[allow(unused, dead_code)]
pub const DEFAULT: &str = "Default";
#[allow(unused, dead_code)]
pub const DEFAULT_BLOCK_VERSION: &str = "0.0.0";
pub const DEFAULT_DOCUMENT_VERSION: &str = "0.0.0";
#[allow(unused, dead_code)]
pub const DEFAULT_RSA_KEY_LENGTH: usize = 2048;
#[allow(unused, dead_code)]
pub const PUBLIC: &str = "Public";
#[allow(unused, dead_code)]
pub const PRIVATE: &str = "Private";
#[allow(unused, dead_code)]
pub const GENERAL: &str = "General";
#[allow(unused, dead_code)]
pub const COMPLETE: &str = "Complete";
pub const SHORT: &str = "Short";
#[allow(unused, dead_code)]
pub const YES: &str = "Y";
#[allow(unused, dead_code)]
pub const NO: &str = "N";
#[allow(unused, dead_code)]
pub const ABSTAIN: &str = "A";
#[allow(unused, dead_code)]
pub const OPEN: &str = "O";
#[allow(unused, dead_code)]
pub const CLOSE: &str = "C";
#[allow(unused, dead_code)]
pub const VALID: &str = "V";
#[allow(unused, dead_code)]
pub const REVOKED: &str = "R";
#[allow(unused, dead_code)]
pub const UNREAD: &str = "UN";
#[allow(unused, dead_code)]
pub const READ: &str = "RD";
#[allow(unused, dead_code)]
pub const FROM: &str = "FM";
#[allow(unused, dead_code)]
pub const TO: &str = "TO";
#[allow(unused, dead_code)]
pub const SENT: &str = "ST";
#[allow(unused, dead_code)]
pub const RECEIVED: &str = "RC";
#[allow(unused, dead_code)]
pub const GQL: &str = "GQL";
#[allow(unused, dead_code)]
pub const TO_BUFFER: &str = "ToBuffer";
#[allow(unused, dead_code)]
pub const TO_NETWORK: &str = "ToNetwork";
/*

  const HashMap<String, String> STATUS_TO_LABEL {
    {"Y", "Yes"},
    {"N", "No"},
    {"O", "Open"},
    {"C", "Close"},
    {"V", "Valid"},
    {"R", "Revoked"},
    {"UN", "Unread"},
    {"RD", "Read"},
    {"FM", "From"},
    {"TO", "To"},
    {"ST", "Sent"},
    {"RC", "Received"}
  };

  pub const NEW =  "NEW";
  pub const SIGNED =  "SIGNED";
  pub const PushedToDocBuffer =  "PushedToDocBuffer";

  pub const receivedPLR = "receivedPLR";  // received Pledge Loan Request
  pub const BundlePPT = "BundlePPT";  // Bundle of Proposal+Pledge+Transactions

*/
pub const MINIMUM_SUS_VOTES_TO_ALLOW_CONSIDERING_SUS_BLOCK: f64 = 51.0;

pub const OUTPUT_TIME_LOCK_IS_ENABLED: bool = false;
// 71 % of block incomes goes to backer
pub const BACKER_PERCENT_OF_BLOCK_FEE: SharesPercentT = 71.0;
pub const TREASURY_PAYMENTS: [&str; 13] =
    [
        "TP_DP",            // Data & Process Costs
        "TP_DONATE_DOUBLE_SPEND",
        "TP_PROPOSAL",      // the costs of proposal register & voting process
        "TP_PLEDGE",        // the pledging costs
        "TP_PLEDGE_CLOSE",  // closing pledge cost. it culd be different based of the concluder
        "TP_POLLING",       // any type of polling(except proposal polling in which the polling doc is created automatically in each node) the payment labled this tag
        "TP_ADM_POLLING",     // Request for network administrative parameters polling
        "TP_BALLOT",        // the Ballot costs
        "TP_INAME_REG",     // iName registerig costs
        "TP_INAME_BIND",     // binding to iName costs
        "TP_INAME_MSG",      // send message to iName costs
        "TP_FDOC",           // custom posts
        "TP_REP_FAILED",      // Reputation failed
    ];
/*
  // (FLENS) for every 180 shares the share holder has permitted to register one iName.
  // iNames are kind of username/nickname/domain-name/aplication-account/email-address/website-address/social-network account
  // even bank account ALL-IN-ONE handler
  const uint64_t SHARES_PER_INAME = 5 * 5 * 6;   // (5 days * 5 hours which will be normal working hours for human) * average level
  const uint64_t INAME_UNIT_PRICE_NO_EMAIL = 1000000 * 710; // initialy it is intentionaly super hi value to avoid spammingor greedy name registering(Domain name speculation), TODO: implement voting for INAME_UNIT_PRICE to reduce it time by time
  const uint64_t INAME_UNIT_PRICE_EMAIL = 1000000 * 71; // initialy it is intentionaly super hi value to avoid spammingor greedy name registering(Domain name speculation), TODO: implement voting for INAME_UNIT_PRICE to reduce it time by time
  const uint64_t INAME_OFFSET = 3;
  const uint64_t INAME_THRESHOLD_LENGTH = 15; // iNames longer than 15 characters cost almost nothing

  const uint32_t DEFAULT_REPAYMENT_SCHEDULE = 2 * 365; // 2 cycle per day * 365 days
  const uint32_t DEFAULT_MAX_REPAYMENTS = 2 * 365 * 2; // 2 cycle per day * 365 days * 2 years
*/
pub mod block_types
{
    pub const GENESIS: &str = "Genesis";
    pub const COINBASE: &str = "Coinbase";
    // floating signature
    pub const FLOATING_SIGNATURE: &str = "FSign";
    pub const NORMAL: &str = "Normal";
    pub const POW: &str = "POW";
    // suspicious/suspended block
    #[allow(unused, dead_code)]
    pub const SUS_BLOCK: &str = "SusBlock";
    pub const FLOATING_VOTE: &str = "FVote";
    // repayment blocks which are used to record repayments in DAG immediately after Coinbase block isuance
    pub const REPAYMENT_BLOCK: &str = "RpBlock";
}

pub const THE_BLOCK_TYPES: [&str; 6] = [
    block_types::NORMAL,
    block_types::COINBASE,
    block_types::FLOATING_SIGNATURE,
    block_types::FLOATING_VOTE,
    block_types::POW,
    block_types::REPAYMENT_BLOCK];

pub mod document_types
{
    pub const COINBASE: &str = "Coinbase";

    // Repayment doc
    pub const REPAYMENT_DOCUMENT: &str = "RpDoc";

    // simple "m of n" trx
    pub const BASIC_TX: &str = "BasicTx";

    pub const DATA_AND_PROCESS_COST_PAYMENT: &str = "DPCostPay";

    // mimblewimble transactions
    #[allow(dead_code, unused)]
    pub const MW_INGRESS: &str = "MWIngress";

    // move PAIs from basic transaction to a mimble transaction
    #[allow(dead_code, unused)]
    pub const MW_TX: &str = "MWTx";

    // a mimble transaction
    #[allow(dead_code, unused)]
    pub const MW_EGRESS: &str = "MWEgress";    // move back PAIs from a mimble transaction to a basic transaction

    // ZKP zk-snarks transactions
    #[allow(dead_code, unused)]
    pub const ZK_INGRESS: &str = "ZKIngress";

    // move PAIs from basic transaction to a Zero Knowledge Proof transaction
    #[allow(dead_code, unused)]
    pub const ZK_TX: &str = "ZKTx";

    // a Zero Knowledge Proof transaction
    #[allow(dead_code, unused)]
    pub const ZK_EGRESS: &str = "ZKEgress";    // move back PAIs from a Zero Knowledge Proof transaction to a basic transaction

    // confidential trx Monero-like
    #[allow(dead_code, unused)]
    pub const MN_INGRESS: &str = "MNIngress";

    // move PAIs from basic transaction to a Zero Knowledge Proof transaction
    #[allow(dead_code, unused)]
    pub const MN_TX: &str = "MNTx";

    // a Zero Knowledge Proof transaction
    #[allow(dead_code, unused)]
    pub const MN_EGRESS: &str = "MNEgress";    // move back PAIs from a Zero Knowledge Proof transaction to a basic transaction

    // RGB (colored coins) Transactions
    // move PAIs from basic transaction to a Zero Knowledge Proof transaction
    #[allow(dead_code, unused)]
    pub const RGB_INGRESS: &str = "RGBIngress";

    #[allow(dead_code, unused)]
    pub const RGB_TX: &str = "RGBTx";

    // a Zero Knowledge Proof transaction
    #[allow(dead_code, unused)]
    pub const RGB_EGRESS: &str = "RGBEgress";    // move back PAIs from a Zero Knowledge Proof transaction to a basic transaction

    pub const PROPOSAL: &str = "Proposal";

    pub const ADMINISTRATIVE_POLLING: &str = "AdmPolling";
    // pub const ReqForRelRes: &str = "ReqForRelRes";    // remove it to AdmPolling

    pub const PLEDGE: &str = "Pledge";
    pub const CLOSE_PLEDGE: &str = "ClosePledge";

    pub const POLLING: &str = "Polling";
    pub const BALLOT: &str = "Ballot";

    #[allow(unused, dead_code)]
    pub const FREE_POST: &str = "FPost"; // Custom Posts (files, Agora posts, wiki pages...)

    // Flens: imagine flexible & largly extensible name service
    #[allow(dead_code, unused)]
    pub const I_NAME_REGISTER: &str = "INameReg";

    // binds a iPGP(later GNU GPG) to an iName
    #[allow(dead_code, unused)]
    pub const I_NAME_BIND: &str = "INameBind";

    #[allow(dead_code, unused)]
    pub const I_NAME_MESSAGE_TO: &str = "INameMsgTo"; // general message to a registered iName

    #[allow(unused, dead_code)]
    pub const CALL_OPTION: &str = "CallOption";
    #[allow(unused, dead_code)]
    pub const PUT_OPTION: &str = "PutOptions";

    #[allow(unused, dead_code)]
    pub const CUSTOM_DOCUMENT: &str = "customDoc"; // custom usages
}

// message parsing settings
pub const MAX_PARSE_ATTEMPTS_COUNT: i32 = 5; // parse message tentative


// floating blocks like floating signature, floating votes, collision logs...
pub mod float_blocks_categories
{
    // collision on spending a coin in some transactions
    pub const TRANSACTION: &str = "Trx";

    // collision on registering an iName
    #[allow(unused, dead_code)]
    pub const FLEXIBLE_NAME_SERVICE: &str = "FleNS";
}


pub mod free_post_classes
{
    #[allow(unused, dead_code)]
    pub const FILE: &str = "File";

    // Demos register an Agora
    #[allow(unused, dead_code)]
    pub const DMS_REGISTER_AGORA: &str = "DMS_RegAgora";
    pub const DMS_POST: &str = "DMS_Post";           // Demos Post

    #[allow(unused, dead_code)]
    pub const WK_CREATE_PAGE: &str = "WK_CreatePage";
    #[allow(unused, dead_code)]
    pub const WK_EDIT_PAGE: &str = "WK_EditPage";
}

pub mod trx_classes
{
    pub const SIMPLE_TX: &str = "SimpleTx";
    // simple "m of n" trx
    #[allow(unused, dead_code)]
    pub const P4P: &str = "P4P";
    // pay for payment
    #[allow(unused, dead_code)]
    pub const BITCOIN: &str = "Bitcoin"; // Bitcoinish transaction to supporting swap transactions
}

/*

namespace PLEDGE_CLOSE_CLASESS
{
pub const General = "General";    //General imagine proposals
};
namespace PROPOSAL_CLASESS
{
pub const General = "General";    //General imagine proposals
};
namespace BALLOT_CLASSES
{
pub const Basic = "Basic";    //Basic Ballot type
};
namespace GENERAL_CLASESS
{
pub const Basic = "Basic";    //Basic message
};
namespace INAME_CLASESS
{
pub const NoDomain = "NoDomain";       // NoDomains: initially imagine do not accept domain-like strings as an iName.
pub const YesDomain = "YesDomain";      // TODO: after implementing a method to autorizing real clessical domain names owner(e.g. google.com) will be activate YesDomain too. TODO: add supporting unicode domains too
};



*/
pub mod message_tags
{
    pub const SENDER_START_TAG: &str = "---SENDER_START_TAG---";
    pub const SENDER_END_TAG: &str = "---SENDER_END_TAG---";
    pub const RECEIVE_START_TAG: &str = "---RECEIVER_START_TAG---";
    pub const RECEIVE_END_TAG: &str = "---RECEIVER_END_TAG---";
    pub const HASH_START_TAG: &str = "---HASH_START_TAG---";
    pub const HASH_END_TAG: &str = "---HASH_END_TAG---";

    #[allow(dead_code, unused)]
    pub const ENVELOPE_CUSTOM_TAG: &str = "CUSTOM ENVELOPE";
    pub const ENVELOPE_CUSTOM_START: &str = "-----ENVELOPE_CUSTOM_START-----";
    pub const ENVELOPE_CUSTOM_END: &str = "-----ENVELOPE_CUSTOM_END-----";
    pub const NO_ENCRYPTION: &str = "NO-ENCRYPTION";

    pub const ENVELOPE_I_PGP_START: &str = "-----ENVELOPE_I_PGP_START-----";
    pub const ENVELOPE_I_PGP_END: &str = "-----ENVELOPE_I_PGP_END-----";

    pub const I_PGP_START_LINEBREAK: &str = "(";
    pub const I_PGP_END_LINEBREAK: &str = ")\n\r<br>";
}

pub mod polling_ref_types
{
    pub const PROPOSAL: &str = "Proposal";
    #[allow(dead_code, unused)]
    pub const ADMINISTRATIVE_POOLING: &str = "AdmPolling";
}

pub mod pledge_classes
{
    // self authorized pledge
    // in this class the pledger signs a contract to cut regularely from mentioned address. (usually the shareholder address by which received incomes from share dvidend)
    // the diference is in this class all 3 parties(pledger, pledgee, arbiter) can close contract whenever they want.
    // this service is usefull for all type of subscription services e.g. monthly payments for T.V. streaming or even better DAILY payment for using services
    // by activating field "applyableAfter" the contract can be realy closed after certain minute of concluding it.
    // e.g. customer must use atleast 3 month of service(in order to get discount)
    // TODO: implement it since it cost couple of hours to implementing
    #[allow(dead_code, unused)]
    pub const PLEDGE_SA: &str = "PledgeSA";

    // pledge class P is designated to pledge an account in order to getting loan to apply proposal for voting.
    // in return if proposal accepted by community part of incomes will repayed to loaner untile get paid all loan+interest
    // so, this class of pledge needs 2 signature in certain order, to be valid
    // 1. pledger, signs pledge request and repayment conditions
    // 2. pledgee, sign and accepts pledging and publish all proposal, pledge-contract and payment-transaction at once in one block
    // 3. arbiter, in case of existance, the arbiters sign pledge-contract and they publish ALL 3(proposal, pledge-contract and payment-transaction),
    //    surely by arbiters signature(arbiterSignsPledge) the pledge hash will be changed.
    pub const PLEDGE_P: &str = "PledgeP";

    // Zero-Knowledge Proof implementation of PledgeP TODO: implement it
    #[allow(dead_code, unused)]
    pub const PLEDGE_PZKP: &str = "PledgePZKP";


    // gneric pledge in which pledger can pledge an address(either has sufficent income from or not) and get a loan
    // and by time payback the loan to same account
    // of course there is no garanty to payback PAIs excep reputation of pledger or her bussiness account which is pledged
    // TODO: implement it ASAP
    #[allow(dead_code, unused)]
    pub const PLEDGE_G: &str = "PledgeG";


    // lending PAIs by pledging an account with sufficent income
    // TODO: implement it ASAP
    #[allow(dead_code, unused)]
    pub const PLEDGE_L: &str = "PledgeL";
}


#[allow(unused, dead_code)]
pub const PLEDGE_ACTIVATE_OR_DEACTIVATE_MATURATION_CYCLE_COUNT: u8 = 2;

/*
// there are 3 ways to unpledge an account
// 1. ByPledgee: The pledgee redeems account (pledgee Closes Pledge). it is most common way supposed to close pledge and redeem account.
//    this is most efficient way to avoid entire network engaging unnecessary calculation.
// 2. ByArbiter: in redeem contract can be add a BECH32 address as a person(or a group of persons) as arbiter. the contract can be closed by arbiter signatures too.
//    the arbitersinetercept in 2 cases. in case of denial of closing contract by pledgee or for the sake of useless process.
//    this method is also cost-less(in sence of network process load) and is a GOOD SAMPLE of trusting and not over-processing by entirenetwork
//    in other word, by giving rights to someone to be arbiter, and givin small wage, and involving them in a contract,
//    the arbiters MUST run a full node (and maybe a full-stack-virtual-machine to evaluate a contract) and the rest of networks doing nothing about this particulare contract
//    in this implementation even an infinit loop problem doesnot collapse entire network, and more over the contracts can be run as an OPTIONAL-PLUGIN on SOME machines
//    and for the rest of network, what is important is the final signature of arbiters, adn they only validate signatures in order to confirm the PAIs transformation
// 3. ByPledger: in case of pledgee refuse to redeem account, pledger requests network to validate redeem cluses by calculate repayments and...
//    and finally vote to redeem (or not redeem) the account. this proces has cost, and must be paid by pledger.
//    althout this has cost for pledger, also has negative effect on pledgee reputation(if network redeem the account).
// 4. ByNetwork, every time the coinbase dividend toke place, every machines calculate automatically. this method is most costly till implementing an efficient way
// either pledgee or arbiter can close contract before head and remit repayments
namespace PLEDGE_CONCLUDER_TYPES
{
pub const ByPledgee = "ByPledgee";
pub const ByArbiter = "ByArbiter";
pub const ByPledger = "ByPledger";
pub const ByNetwork = "ByNetwork";
};

namespace BINDING_TYPES
{
pub const IPGP = "IPGP";
};
*/
pub mod signature_types
{
    // Mixed of sha256 and keccak256 signature. TODO: implement Mix32 recording on blockgraph
    pub const MIX23: &str = "Mix23";

    // Basic signature
    pub const BASIC: &str = "Basic";

    // Bitcoin like sha256 hash address
    #[allow(dead_code, unused)]
    pub const BITCOIN: &str = "Bitcoin";

    // simple light signature for Internet Of Things
    #[allow(dead_code, unused)]
    pub const IOT: &str = "IOT";

    // Strict signature by which some signer are allowed to pledge/unpledge account or delegate it
    pub const STRICT: &str = "Strict";

    // Strict Input-time-lock signature in which all inputs can not be spendable before passing a certain time.
    #[allow(dead_code, unused)]
    pub const STRICTITL: &str = "STRICTITL";
}

#[allow(unused, dead_code)]
pub const DEFAULT_SIGNATURE_MOD: &str = "2/3"; // needs 2 signature of 3


pub const DEFAULT_PACKET_VERSION: &str = "0.0.3";
pub const DEFAULT_PACKET_TYPE: &str = "Standard";
pub const DEFAULT_CARD_VERSION: &str = "0.0.1";
pub const DEFAULT_SAFE_VERSION: &str = "0.0.0";

pub mod card_types
{
    // simple cards
    pub const HANDSHAKE: &str = "handshake";
    pub const NICE_TO_MEET_YOU: &str = "niceToMeetYou";
    pub const HERE_IS_NEW_NEIGHBOR: &str = "hereIsNewNeighbor";


    // TODO: move these commands to GQL format
    pub const DAG_INVOKE_LEAVES: &str = "dagInvokeLeaves";
    pub const DAG_LEAVES_INFO: &str = "dagLeavesInfo";
    pub const DAG_INVOKE_BLOCK: &str = "dagInvokeBlock";
    pub const DAG_INVOKE_DESCENDENTS: &str = "dagInvokeDescendents";

    // complicated cards
    pub const PROPOSAL_LOAN_REQUEST: &str = "ProposalLoanRequest";
    pub const FULL_DAG_DOWNLOAD_REQUEST: &str = "FullDAGDownloadRequest";
    pub const FULL_DAG_DOWNLOAD_RESPONSE: &str = "FullDAGDownloadResponse";
    pub const BALLOTS_RECEIVE_DATES: &str = "BallotsReceiveDates";
    pub const NODE_STATUS_SCREENSHOT: &str = "NodeStatusScreenshot";
    pub const NODE_STATUS_SNAPSHOT: &str = "NodeStatusSnapshot";

    #[allow(unused, dead_code)]
    pub const PLEASE_REMOVE_ME_FROM_YOUR_NEIGHBORS: &str = "pleaseRemoveMeFromYourNeighbors";
    pub const DIRECT_MESSAGE_TO_NEIGHBOR: &str = "directMsgToNeighbor";
}

pub const THE_CARD_TYPES: [&str; 15] = [
    card_types::DAG_INVOKE_BLOCK,
    card_types::DAG_INVOKE_DESCENDENTS,
    card_types::DAG_INVOKE_LEAVES,
    card_types::DAG_LEAVES_INFO,
    card_types::HANDSHAKE,
    card_types::NICE_TO_MEET_YOU,
    card_types::HERE_IS_NEW_NEIGHBOR,
    card_types::PROPOSAL_LOAN_REQUEST,
    card_types::FULL_DAG_DOWNLOAD_REQUEST,
    card_types::PLEASE_REMOVE_ME_FROM_YOUR_NEIGHBORS,
    card_types::FULL_DAG_DOWNLOAD_RESPONSE,
    card_types::BALLOTS_RECEIVE_DATES,
    card_types::NODE_STATUS_SNAPSHOT,
    card_types::NODE_STATUS_SCREENSHOT,
    card_types::DIRECT_MESSAGE_TO_NEIGHBOR];


pub const OUTPUT_DP_COST: &str = "OUTPUT_DP_COST";
pub const OUTPUT_NORMAL: &str = "OUTPUT_NORMAL";
pub const TEMP_DP_COST_AMOUNT: CMPAIValueT = 111_111_111;
#[allow(unused, dead_code)]
pub const OUTPUT_TREASURY: &str = "OUTPUT_TREASURY";
pub const OUTPUT_CHANGE_BACK: &str = "OUTPUT_CHANGE_BACK";

pub mod change_back_mods
{
    pub const EXACT_ADDRESS: &str = "exactAddress";
    pub const BACKER_ADDRESS: &str = "backerAddress";
    pub const A_NEW_ADDRESS: &str = "aNewAddress";
}

pub mod transaction_fee_calculate_methods
{
    pub const EXACT_FEE: &str = "exactFee";
    #[allow(unused, dead_code)]
    pub const MIN_FEE: &str = "minFee";
}

pub const CURRENT_SIGNATURE_VERSION: &str = "0.0.1";
pub const CURRENT_AES_VERSION: &str = "0.0.0";
pub const CURRENT_PGP_VERSION: &str = "0.0.0";

// GUI settings
#[allow(unused, dead_code)]
pub const WATCHING_BLOCKS_COUNT: u16 = 300; // default is 5 minutes = 300 second

// hu part
#[allow(unused, dead_code)]
pub const HU_SHARE_ADDRESS: &str = "im1xqenscfjvscnjdfs89jngce5vd3rwd33vy6kxdtpxvukgwryxscs6xpe6r";
#[allow(unused, dead_code)]
pub const HU_INAME_OWNER_ADDRESS: &str = "im1xqenscfjvscnjdfs89jngce5vd3rwd33vy6kxdtpxvukgwryxscs6xpe6r";
pub const INITIAL_SHARES: i32 = 10_000_000;

