use crate::lib::custom_types::{BlockLenT, DocLenT};

#[allow(dead_code)]
pub enum Modules {
    App,
    CB,
    Trx,
    Sql,
    Sec,
}

#[allow(dead_code)]
pub enum SecLevel {
    Debug,
    Trace,
    Info,
    Warning,
    Error,
    Fatal,
}


#[allow(dead_code)]
pub(crate) const SOCIETY_NAME: &str = "im";

// in live environment time gain must be 1, in develop mode it is equal one cycle by minutes e.g. 5
pub(crate) const TIME_GAIN: u8 = 1;
pub(crate) const STANDARD_CYCLE_BY_MINUTES: u32 = 720;

//bech32 part
pub(crate) const BECH32_ADDRESS_VER: &str = "0";
pub(crate) const TRUNCATE_FOR_BECH32_ADDRESS: u8 = 32;


/*

const CMPAISValueT MAX_COIN_VALUE = 9007199254740991; // to be compatible with JS clients: Number.MAX_SAFE_INTEGER

//bech32 part
const std::string BECH32_ADDRESS_VER = "0"; // version must be one char
const int TRUNCATE_FOR_BECH32_ADDRESS = 32; //


namespace CConsts
{

  /**
  * @brief SOCIETY_NAME
  * the first society is imagine.
  * people simply by change this constant to ahatever they want can stablish a new community (having new money and rules, etc...) in a matter of minute
  */
  pub const SOCIETY_NAME = "im";
  */
pub(crate) const LAUNCH_DATE: &str = "2020-02-02 00:00:00";
/*
  const uint8_t TIME_GAIN = 10;  // in live environment time gain must be 1, in develop mode it is equal one cycle by minutes e.g. 5
   */
pub(crate) const EMAIL_IS_ACTIVE: bool = false;
/*
  const bool BROADCAST_TO_NEIGHBOR_OF_NEIGHBOR = true;    // if machine allowed to publish email of neightbors of neighbors to hei neighbors?
  const bool SUPPORTS_CLONED_TRANSACTION = false;    // after writing a tons of unittests can activate this feature
  const bool SUPPORTS_P4P_TRANSACTION = false;    // after writing a tons of unittests can activate this feature
*/
pub const LAUNCH_YEAR: u16 = 2022;
// every twelve years the coinbse divided to 2
pub const HALVING_PERIOD: u16 = 1;
// every twelve years the coinbse divided to 2

pub const COIN_ISSUING_INIT_EXPONENT: u8 = 11;
// the power of 2 for minting new coins
pub const COINBASE_MATURATION_CYCLES: u8 = 2;

pub const ONE_MILLION: u64 = 1_000_000;
pub const ONE_BILLION: u64 = 1_000_000_000;

pub const CLIENT_VERSION: &str = "0.2.0";
pub(crate) const DATABASAE_AGENT: &str = "psql";

// can be sqlite or psql
/*
  const uint DEFAULT_CONTRIBUTE_LEVEL = 6;

  // at least 4 cycle must be a gap between pay to treasury and dividing to shareholders
*/
pub const TREASURY_MATURATION_CYCLES: u8 = 4;
pub const DUMPER_INDENT: &str = "  ";
pub const DO_HARDCOPY_DAG_BACKUP: bool = true;
// it creates a DAG backup in sense of blocks
pub const DO_HARDCOPY_OUTPUT_EMAILS: bool = true;

// it creates a local copy of what your machine will send via email
/*

  const uint16_t GAP_EMAIL_POP = 300; // default is 5 minutes = 300 second

  const uint64_t STANDARD_CYCLE_BY_MINUTES = static_cast<uint64_t>(720);


  pub const JS_FAKSE_NULL = "__js__null__";
  // the maximum time in which local machine wait to attain most possible confidence coinbase block
  // best value is 3/4 of one coinbase cycle
  */

pub const COINBASE_FLOOR_TIME_TO_RECORD_IN_DAG: f64 = 3.0 / 5.0;
pub const MINIMUM_SHARES_IF_IS_NOT_SHAREHOLDER: f64 = 0.0000000001;
pub const SHARE_MATURITY_CYCLE: u8 = 2;

pub const CONTRIBUTION_APPRECIATING_PERIOD: u16 = 700;

// every little help is appreciated for 7 years
// 2 cycle after inserting a share in DB, coinbase will include the newly recorded share in it's dividend
/*
  // to avoid spam proposals, there is a cost equal to 30 days of potential income of the proposal
  // this cost goes to treasury and divide between shareholders

  // in order to put a proposal on voting process, requester has to pay one month income * 3 to treasury
  const uint PROPOSAL_APPLY_COST_SCALE = 3;

  /**
   * hopefully after 10 cycle(5 days) all DAG branches are merged and
   * ALL transactions from 5 days ago are visible by leaves,
   * so we do not need to mantain spend coins in DB
   */
  const uint KEEP_SPENT_COINS_BY_CYCLE = 10;

  const uint32_t ONE_MINUTE_BY_MILISECOND = 60000;
  namespace SIGHASH
  {
    pub const ALL = "ALL";
    pub const NONE = "NONE";
    // these have conflict with BIP69, that's why we need custom SIGHASH
    // 'SINGLE': 'SINGLE',
    // 'ALL|ANYONECANPAY': 'ALL|ANYONECANPAY',
    // 'NONE|ANYONECANPAY': 'NONE|ANYONECANPAY',
    // 'SINGLE|ANYONECANPAY': 'SINGLE|ANYONECANPAY',
    pub const CUSTOM = "CUSTOM"; // TODO: implement it in order to have ability to sign some random inputs & outputs
  };
*/
pub const MAX_TOLERATED_MISS_BLOCKS: u8 = 5;
// if machine missed more than this number, does not allowed to issue a coinbase block
/*

  const bool SUPER_CONTROL_UTXO_DOUBLE_SPENDING = true; // can be disabled for the sake of performance
  const bool SUPER_CONTROL_COINS_BACK_TO_COINBASE_MINTING = true;  // can be disabled for the sake of performance
  const bool log_super_validate_debug = true;  // can be disabled for the sake of performance

  const bool DECODE_ALL_FREE_POST_FILES = true; // TODO: improve it to support different file types & security levels
*/
pub(crate) const SIGN_MSG_LENGTH: u8 = 32;
pub(crate) const FLOAT_LENGTH: u8 = 11;
pub(crate) const LEN_PROP_LENGTH: u8 = 7;
pub(crate) const LEN_PROP_PLACEHOLDER: &str = "0000000";
pub(crate) const HASH_PROP_PLACEHOLDER: &str = "0000000000000000000000000000000000000000000000000000000000000000";
/*

  const uint16_t TRANSACTION_PADDING_LENGTH = 100;
  const uint16_t TRANSACTION_MINIMUM_LENGTH = 375;    // smallest transaction has 375 charachter length
*/

/**
 * this "MAX_BLOCK_LENGTH_BY_CHAR" is a single max size of single message after adding headers and encrypting
 * it means real payload of a block (or real GQL messegaes size) must be roughly around 80% - 90%
 */
pub const MAX_BLOCK_LENGTH_BY_CHAR: BlockLenT = 10000000 * 10;


// 10 Mega Byte is actually block size
pub const MAX_DOC_LENGTH_BY_CHAR: DocLenT = 10 * 1024 * 1024 * 9;
pub const MAX_DPCostPay_DOC_SIZE: DocLenT = 600;
pub const MAX_FullDAGDownloadResponse_LENGTH_BY_CHAR: u64 = 1500000;

pub mod stages
{
    pub const Creating: &str = "Creating";
    pub const Regenerating: &str = "Regenerating";
    pub const Validating: &str = "Validating";
}

pub mod thread_state
{
    pub(crate) const RUNNING: &str = "RUNNING";
    pub(crate) const SLEEPING: &str = "SLEEPING";
    pub(crate) const STOPPED: &str = "STOPPED";
}

/*
  namespace COIN_SELECTING_METHOD
  {
    pub const PRECISE = "PRECISE";
    pub const BIGGER_FIRST = "BIGGER_FIRST";
    pub const SMALLER_FIRST = "SMALLER_FIRST";
    pub const RANDOM = "RANDOM";
  };

*/

pub(crate) const HD_ROOT_FILES: &str = "/Users/silver/Documents/Diamond_files";

pub(crate) mod psql_db
{
    pub(crate) const DB_HOST: &str = "localhost";
    pub(crate) const DB_NAME: &str = "diamond";
    pub(crate) const DB_USER: &str = "diamond";
    pub(crate) const DB_PASS: &str = "diamondpass";
}

pub const DEFAULT_LANG: &str = "eng";
pub const DEFAULT_VERSION: &str = "0.0.0";
pub const WRAP_SAFE_VERION: &str = "0.0.0";

pub const NL: &str = "\n";
pub const TAB: &str = "\t";

pub const ALL: &str = "All";
pub const DEFAULT: &str = "Default";
pub const DEFAULT_BLOCK_VERSION: &str = "0.0.0";
pub const DEFAULT_DOCUMENT_VERSION: &str = "0.0.0";
pub const DEFAULT_CONTENT_VERSION: &str = "0.0.0";
pub const PUBLIC: &str = "Public";
pub const PRIVATE: &str = "Private";
pub const GENERAL: &str = "General";
/*

  pub const COMPLETE =  "Complete";
  pub const SHORT =  "Short";

*/
pub const YES: &str = "Y";
pub const NO: &str = "N";
pub const ABSTAIN: &str = "A";
pub const OPEN: &str = "O";
pub const CLOSE: &str = "C";
pub const VALID: &str = "V";
pub const REVOKED: &str = "R";
pub const UNREAD: &str = "UN";
pub const READ: &str = "RD";
pub const FROM: &str = "FM";
pub const TO: &str = "TO";
pub const SENT: &str = "ST";
pub const RECEIVED: &str = "RC";
pub const GQL: &str = "GQL";
pub const TO_BUFFER: &str = "ToBuffer";
pub const TO_NETWORK: &str = "ToNetwork";
/*

  const QHash<String, String> STATUS_TO_LABEL {
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

  const float MINIMUM_SUS_VOTES_TO_ALLOW_CONSIDERING_SUS_BLOCK = 51.0;
  const uint8_t BACKER_PERCENT_OF_BLOCK_FEE = 71; // 71 % of block incomes goes to backer
*/
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
        // 'TP_REQRELRES',     // Request for Release Reserved coins
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
    pub const Genesis: &str = "Genesis";
    pub const Coinbase: &str = "Coinbase";
    // floating signature
    pub const FSign: &str = "FSign";
    pub const Normal: &str = "Normal";
    pub const POW: &str = "POW";
    // suspicious/suspended block
    pub const SusBlock: &str = "SusBlock";
    pub const FVote: &str = "FVote";
    // repayment blocks which are used to record repayments in DAG immediately after Coinbase block isuance
    pub const RpBlock: &str = "RpBlock";
}

pub mod doc_types
{
    pub const Coinbase: &str = "Coinbase";
    pub const RpDoc: &str = "RpDoc";
    // Repayment doc
    pub const RlDoc: &str = "RlDoc";
    // Release Reserved coins
    pub const BasicTx: &str = "BasicTx";    // simple "m of n" trx

    // mimblewimble transactions
    pub const MWIngress: &str = "MWIngress";
    // move PAIs from basic transaction to a mimble transaction
    pub const MWTx: &str = "MWIngress";
    // a mimble transaction
    pub const MWEgress: &str = "MWEgress";    // move back PAIs from a mimble transaction to a basic transaction

    // ZKP zk-snarks transactions
    pub const ZKIngress: &str = "MWIngress";
    // move PAIs from basic transaction to a Zero Knowladge Proof transaction
    pub const ZKTx: &str = "MWIngress";
    // a Zero Knowladge Proof transaction
    pub const ZKEgress: &str = "MWEgress";    // move back PAIs from a Zero Knowladge Proof transaction to a basic transaction

    // confidential trx Monero-like
    pub const MNIngress: &str = "MWIngress";
    // move PAIs from basic transaction to a Zero Knowladge Proof transaction
    pub const MNTx: &str = "MWIngress";
    // a Zero Knowladge Proof transaction
    pub const MNEgress: &str = "MWEgress";    // move back PAIs from a Zero Knowladge Proof transaction to a basic transaction

    // RGB (colored coins) Transactions
    pub const RGBIngress: &str = "MWIngress";
    // move PAIs from basic transaction to a Zero Knowladge Proof transaction
    pub const RGBTx: &str = "MWIngress";
    // a Zero Knowladge Proof transaction
    pub const RGBEgress: &str = "MWEgress";    // move back PAIs from a Zero Knowladge Proof transaction to a basic transaction


    pub const DPCostPay: &str = "DPCostPay";

    pub const DNAProposal: &str = "DNAProposal";

    pub const AdmPolling: &str = "AdmPolling";
    pub const ReqForRelRes: &str = "ReqForRelRes";    // remove it to AdmPolling


    pub const Pledge: &str = "Pledge";
    pub const ClosePledge: &str = "ClosePledge";

    pub const Polling: &str = "Polling";
    pub const Ballot: &str = "Ballot";

    pub const FPost: &str = "FPost"; // Custom Posts (files, Agora posts, wiki pages...)

    pub const INameReg: &str = "INameReg";
    // Flens: imagine flexible & largly extensible name service
    pub const INameBind: &str = "INameBind";
    // binds a iPGP(later GNU GPG) to an iName
    pub const INameMsgTo: &str = "INameMsgTo"; // general message to a registered iName
}
/*

// message parsing settings
const uint MAX_PARSE_ATTEMPS_COUNT = 5; // parse message tentative


pub const NO_EXT_HASH = "NEH";

// floating blocks like floating signature, floating votes, collision logs...
namespace FLOAT_BLOCKS_CATEGORIES
{
pub const Trx = "Trx";     // collision on spending a coin in some transactions
pub const FleNS = "FleNS"; // collision on registring an iName
}
*/
pub mod document_types {
    pub const Coinbase: &str = "Coinbase";
    pub const RpDoc: &str = "RpDoc";
    // Repayment doc
    pub const BasicTx: &str = "BasicTx";    // simple "m of n" trx

    pub const DPCostPay: &str = "DPCostPay";

    pub const DNAProposal: &str = "DNAProposal";

    pub const AdmPolling: &str = "AdmPolling";
// ReqForRelRes : &str = "ReqForRelRes";    remove it to AdmPolling


    pub const Pledge: &str = "Pledge";
    pub const ClosePledge: &str = "ClosePledge";

    pub const Polling: &str = "Polling";
    pub const Ballot: &str = "Ballot";

    pub const INameReg: &str = "INameReg";
    // Flens: imagine flexible & largly extensible name service
    pub const INameBind: &str = "INameBind";
    // binds a iPGP(later GNU GPG) to an iName
    pub const INameMsgTo: &str = "INameMsgTo"; // general message to a registered iName


    pub const CallOption: &str = "CallOption";
    pub const PutOptions: &str = "PutOptions";


    pub const customDoc: &str = "customDoc"; // custom usages
}

/*

namespace FPOST_CLASSES
{
pub const File = "File";

pub const DMS_RegAgora = "DMS_RegAgora";   // Demos register an Agora
pub const DMS_Post = "DMS_Post";           // Demos Post

pub const WK_CreatePage = "WK_CreatePage";
pub const WK_EditPage = "WK_EditPage";
};

namespace TRX_CLASSES
{
pub const SimpleTx = "SimpleTx";    // simple "m of n" trx
pub const P4P = "P4P"; // pay for payment
pub const Bitcoin = "Bitcoin"; // Bitcoinish transaction to supporting swap transactions
};

namespace PLEDGE_CLOSE_CLASESS
{
pub const General = "General";    //General imagine DNA proposals
};
namespace PROPOSAL_CLASESS
{
pub const General = "General";    //General imagine DNA proposals
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



namespace MESSAGE_TAGS
{
pub const senderStartTag = "---START_SENDER_TAG---";
pub const senderEndTag = "---END_SENDER_TAG---";
pub const receiverStartTag = "---START_RECEIVER_TAG---";
pub const receiverEndTag = "---END_RECEIVER_TAG---";
pub const hashStartTag = "---START_HASH_TAG---";
pub const hashEndTag = "---END_HASH_TAG---";

pub const customEnvelopeTag = "CUSTOM ENVELOPE";
pub const customStartEnvelope = "-----START_CUSTOM_ENVELOPE-----";
pub const customEndEnvelope = "-----END_CUSTOM_ENVELOPE-----";
pub const NO_ENCRYPTION = "NO-ENCRYPTION";

pub const iPGPStartEnvelope = "-----START_iPGP_ENVELOPE-----";
pub const iPGPEndEnvelope = "-----END_iPGP_ENVELOPE-----";

pub const iPGPStartLineBreak = "(";
pub const iPGPEndLineBreak = ")\n\r<br>";
};

*/
pub mod polling_ref_types
{
    pub const Proposal: &str = "Proposal";
    pub const ReqForRelRes: &str = "ReqForRelRes";
    // Request For release Reserved
    pub const AdmPolling: &str = "AdmPolling";
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
    pub const PledgeSA: &str = "PledgeSA";

    // pledge class P is designated to pledge an account in order to getting loan to apply proposal for voting.
// in return if proposal accepted by community part of incomes will repayed to loaner untile get paid all loan+interest
// so, this class of pledge needs 2 signature in certain order, to be valid
// 1. pledger, signs pledge request and repayment conditions
// 2. pledgee, sign and accepts pledging and publish all proposal, pledge-contract and payment-transaction at once in one block
// 3. arbiter, in case of existance, the arbiters sign pledge-contract and they publish ALL 3(proposal, pledge-contract and payment-transaction),
//    surely by arbiters signature(arbiterSignsPledge) the pledge hash will be changed.
    pub const PledgeP: &str = "PledgeP";

    // Zero-Knowledge Proof implementation of PledgeP TODO: implement it
    pub const PledgePZKP: &str = "PledgePZKP";


    // gneric pledge in which pledger can pledge an address(either has sufficent income from DNA or not) and get a loan
// and by time payback the loan to same account
// of course there is no garanty to payback PAIs excep reputation of pledger or her bussiness account which is pledged
// TODO: implement it ASAP
    pub const PledgeG: &str = "PledgeG";


    // lending PAIs by pledging an account with sufficent income
// TODO: implement it ASAP
    pub const PledgeL: &str = "PledgeL";
}


pub const PLDEGE_ACTIVATE_OR_DEACTIVATE_MATURATION_CYCLE_COUNT: u8 = 2;

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
    pub const MIX23: &str = "Mix23";
    // Mixed of sha256 and keccak256 signature. TODO: implement Mix32 recording on blockgraph
    pub const BASIC: &str = "Basic";
    // Basic signature
    pub const BITCOIN: &str = "Bitcoin";
    // Bitcoin like sha256 hash address
    pub const IOT: &str = "IOT";
    // simple light signature for Internet Of Things
    pub const STRICT: &str = "Strict";
    // Strict signature by which some signer are allowed to pledge/unpledge account or delegate it
    pub const STRICTITL: &str = "STRICTITL"; // Strict Input-time-lock signature in which all inputs can not be spendable before passing a certain time.
}
/*
  pub const DEFAULT_SIGNATURE_MOD = "2/3"; // needs 2 signature of 3

  namespace CARD_TYPES
  {
      pub const ProposalLoanRequest = "ProposalLoanRequest";
      pub const FullDAGDownloadRequest = "FullDAGDownloadRequest";
      pub const FullDAGDownloadResponse = "FullDAGDownloadResponse";
      pub const BallotsReceiveDates = "BallotsReceiveDates";
      pub const NodeStatusScreenshot = "NodeStatusScreenshot";
      pub const NodeStatusSnapshot = "NodeStatusSnapshot";

      pub const pleaseRemoveMeFromYourNeighbors = "pleaseRemoveMeFromYourNeighbors";
      pub const directMsgToNeighbor = "directMsgToNeighbor";
  };

  namespace MESSAGE_TYPES
  {
    pub const HANDSHAKE = "handshake";
    pub const NICETOMEETYOU = "niceToMeetYou";
    pub const HEREISNEWNEIGHBOR = "hereIsNewNeighbor";

    // TODO: move these commands to GQL format
    pub const DAG_INVOKE_LEAVES = "dagInvokeLeaves";
    pub const DAG_LEAVES_INFO = "dagLeavesInfo";
    pub const DAG_INVOKE_BLOCK = "dagInvokeBlock";
    pub const DAG_INVOKE_DESCENDENTS = "dagInvokeDescendents";
  };

*/
pub const OUTPUT_DPCOST: &str = "OUTPUT_DPCOST";
pub const OUTPUT_NORMAL: &str = "OUTPUT_NORMAL";
pub const OUTPUT_TREASURY: &str = "OUTPUT_TREASURY";
pub const OUTPUT_CHANGEBACK: &str = "OUTPUT_CHANGEBACK";

/*
  /**
   * @brief CURRENT_AES_VERSION
   * ver 0.0.0 to connect to Javascript clients
   * ver 0.2.0 to connect to C++ clients
   */
  pub const CURRENT_AES_VERSION = "0.2.0";

  pub const FAKE_RIGHT_HASH_PREFIX = "_";

  // GUI settings
  const uint16_t WATCHING_BLOCKS_COUNT = 300; // default is 5 minutes = 300 second

*/
// hu part
//im1xq6rwefjxgcnxwfc8qcxxd35xd3rqvt9vy6r2wr9xa3nwvenv3ssnm4w8c
pub const HU_DNA_SHARE_ADDRESS: &str = "im1xqexzdn9x5mrgcfcv5cnswrrxu6nzvpk8yuxzdpkvcunqwpnv3jq7rps4d";
pub const HU_INAME_OWNER_ADDRESS: &str = "im1xq6rwefjxgcnxwfc8qcxxd35xd3rqvt9vy6r2wr9xa3nwvenv3ssnm4w8c";
/*
}

class GenRes
{
public:
    GenRes();
    GenRes(const bool &st, pub const &ms="");
    bool status = false;
    String msg = "";
};

#endif // CONTANTS_H

 */