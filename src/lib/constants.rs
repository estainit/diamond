#[allow(dead_code)]
pub enum Modules {
    App,
    CB,
    Trx,
    Sql,
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

pub(crate) const HD_FILES: &str = "/Users/silver/Documents/Diamond_files/";

#[allow(dead_code)]
pub(crate) const COINBASE_MATURATION_CYCLES: u8 = 2;

//bech32 part
pub(crate) const BECH32_ADDRESS_VER: &str = "0";
pub(crate) const TRUNCATE_FOR_BECH32_ADDRESS: u8 = 32;

pub(crate) const SIGN_MSG_LENGTH: u8 = 32;

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
  const QString SOCIETY_NAME = "im";
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

  const uint LAUNCH_YEAR = 2019; // every twelve years the coinbse divided to 2
  const uint HALVING_PERIOD = 20; // every twelve years the coinbse divided to 2

  const uint COIN_ISSUING_INIT_EXPONENT = 52; // the power of 2 for minting new coins
  const uint COINBASE_MATURATION_CYCLES = 2;

  /**
    * so every growth of shares(growth of contribute efforts) and pasing share ceil, causes to growth releaseable coins (by Fibonacci sequence).
    * until reaching entire world population having shares in imagine's network
    * it IS/MUST absolutely simple & clear rule, understandable, applyable & auditable for ALL.
    * implementation details in "calculateReleasableCoinsBasedOnContributesVolume"
    * */
  const QHash<uint8_t, uint8_t> MAP_CONTRIBUTE_AMOUNT_TO_MINTING_PERCENTAGE
  {
    {1 , 1},
    {2 , 1},
    {3 , 1},
    {5 , 2},
    {8 , 3},
    {13 , 5},
    {21 , 8},
    {34 , 13},
    {55 , 21},
    {89 , 34},
    {144 , 55},
    {233 , 89},
    {377 , 100}
  };
  const uint64_t WORLD_POPULATION = 8000000000;   //8,000,000,000
  const uint64_t ONE_MILLION = 1000000;


  const QString CLIENT_VERSION = "0.2.0";
*/
pub(crate) const DATABASAE_AGENT: &str = "psql";

// can be sqlite or psql
/*
  const uint DEFAULT_CONTRIBUTE_LEVEL = 6;

  // at least 4 cycle must be a gap between pay to treasury and dividing to shareholders
  const uint TREASURY_MATURATION_CYCLES = 4;
*/
pub const DUMPER_INDENT: &str = "  ";
/*

  const uint16_t GAP_EMAIL_POP = 300; // default is 5 minutes = 300 second

  const uint64_t STANDARD_CYCLE_BY_MINUTES = static_cast<uint64_t>(720);

  const bool DO_HARDCOPY_DAG_BACKUP = true;   // it creates a DAG backup in sense of blocks
  const bool DO_HARDCOPY_OUTPUT_EMAILS = true; // it creates a local copy of what your machine will send via email

  const QString JS_FAKSE_NULL = "__js__null__";
  // the maximum time in which local machine wait to attain most possible confidence coinbase block
  // best value is 3/4 of one coinbase cycle
  const double COINBASE_FLOOR_TIME_TO_RECORD_IN_DAG = 3 / 5;
  const double MINIMUM_SHARES_IF_IS_NOT_SHAREHOLDER = 0.0000000001;
  const uint SHARE_MATURITY_CYCLE = 2;    // 2 cycle after inserting a share in DB, coinbase will include the newly recorded share in it's dividend
  const uint CONTRIBUTION_APPRECIATING_PERIOD = 7; // every little help is appreciated for 7 years
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
    const QString ALL = "ALL";
    const QString NONE = "NONE";
    // these have conflict with BIP69, that's why we need custom SIGHASH
    // 'SINGLE': 'SINGLE',
    // 'ALL|ANYONECANPAY': 'ALL|ANYONECANPAY',
    // 'NONE|ANYONECANPAY': 'NONE|ANYONECANPAY',
    // 'SINGLE|ANYONECANPAY': 'SINGLE|ANYONECANPAY',
    const QString CUSTOM = "CUSTOM"; // TODO: implement it in order to have ability to sign some random inputs & outputs
  };

  const uint8_t MAX_TOLERATED_MISS_BLOCKS = 5;    // if machine missed more than this number, does not allowed to issue a coinbase block

  const bool SUPER_CONTROL_UTXO_DOUBLE_SPENDING = true; // can be disabled for the sake of performance
  const bool SUPER_CONTROL_COINS_BACK_TO_COINBASE_MINTING = true;  // can be disabled for the sake of performance
  const bool log_super_validate_debug = true;  // can be disabled for the sake of performance

  const bool DECODE_ALL_FREE_POST_FILES = true; // TODO: improve it to support different file types & security levels

  const uint8_t SIGN_MSG_LENGTH = 32;
  const uint16_t TRANSACTION_PADDING_LENGTH = 100;
  const uint16_t TRANSACTION_MINIMUM_LENGTH = 375;    // smallest transaction has 375 charachter length

  /**
   * this "MAX_BLOCK_LENGTH_BY_CHAR" is a single max size of single message after adding headers and encrypting
   * it means real payload of a block (or real GQL messegaes size) must be roughly around 80% - 90%
   */
  const BlockLenT MAX_BLOCK_LENGTH_BY_CHAR = 10000000 * 10;


  const DocLenT MAX_DOC_LENGTH_BY_CHAR = 10 * 1024 * 1024 * 9;   // 10 Mega Byte is actually block size
  const DocLenT MAX_DPCostPay_DOC_SIZE = 600;
  const uint64_t MAX_FullDAGDownloadResponse_LENGTH_BY_CHAR = 1500000;

  namespace STAGES
  {
    const QString Creating = "Creating";
    const QString Regenerating = "Regenerating";
    const QString Validating = "Validating";
  };
*/
pub mod thread_state
{
    pub(crate) const RUNNING: &str = "RUNNING";
    pub(crate) const SLEEPING: &str = "SLEEPING";
    pub(crate) const STOPPED: &str = "STOPPED";
}

/*
  namespace COIN_SELECTING_METHOD
  {
    const QString PRECISE = "PRECISE";
    const QString BIGGER_FIRST = "BIGGER_FIRST";
    const QString SMALLER_FIRST = "SMALLER_FIRST";
    const QString RANDOM = "RANDOM";
  };



  const QString HD_ROOT_PATHE = "";
  const QString HD_FILES = HD_ROOT_PATHE + "hd_files";

  namespace HD_PATHES
  {
    const QString GUI_ASSETS = "/tmp/comen_gui_asstes"; // FIXME: change it to standard path
  }
*/
pub(crate) mod psql_db
{
    pub(crate) const DB_HOST: &str = "localhost";
    pub(crate) const DB_NAME: &str = "diamond";
    pub(crate) const DB_USER: &str = "diamond";
    pub(crate) const DB_PASS: &str = "diamondpass";
}

pub const DEFAULT_LANG: &str = "eng";
pub const DEFAULT_VERSION: &str = "0.0.0";

pub const NL: &str = "\n";
pub const TAB: &str = "\t";

pub const ALL: &str = "All";
pub const DEFAULT: &str = "Default";
pub const DEFAULT_DOCUMENT_VERSION: &str = "0.0.0";
pub const DEFAULT_CONTENT_VERSION: &str = "0.0.0";
pub const PUBLIC: &str = "Public";
pub const PRIVATE: &str = "Private";
pub const GENERAL: &str = "General";
/*

  const QString COMPLETE =  "Complete";
  const QString SHORT =  "Short";

*/
pub const YES: &str = "Y";
pub const NO: &str = "N";
/*
  const QString ABSTAIN =  "A";
  const QString OPEN =  "O";
  const QString CLOSE =  "C";
  const QString VALID =  "V";
  const QString REVOKED =  "R";
  const QString UNREAD =  "UN";
  const QString READ =  "RD";
  const QString FROM =  "FM";
  const QString TO =  "TO";
  const QString SENT =  "ST";
  const QString RECEIVED =  "RC";
  const QString GQL =  "GQL";
  const QString TO_BUFFER =  "ToBuffer";
  const QString TO_NETWORK =  "ToNetwork";

  const QHash<QString, QString> STATUS_TO_LABEL {
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

  const QString NEW =  "NEW";
  const QString SIGNED =  "SIGNED";
  const QString PushedToDocBuffer =  "PushedToDocBuffer";

  const QString receivedPLR = "receivedPLR";  // received Pledge Loan Request
  const QString BundlePPT = "BundlePPT";  // Bundle of Proposal+Pledge+Transactions

  const float MINIMUM_SUS_VOTES_TO_ALLOW_CONSIDERING_SUS_BLOCK = 51.0;
  const uint8_t BACKER_PERCENT_OF_BLOCK_FEE = 71; // 71 % of block incomes goes to backer

  const QStringList TREASURY_PAYMENTS =
  {
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
  };

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

  namespace BLOCK_TYPES
  {
    const QString Genesis = "Genesis";
    const QString Coinbase = "Coinbase";
    const QString FSign = "FSign"; // floating signature
    const QString Normal = "Normal";
    const QString POW = "POW";
    const QString SusBlock = "SusBlock"; // suspicious/suspended block
    const QString FVote = "FVote";
    const QString RpBlock = "RpBlock"; // repayment blocks which are used to record repayments in DAG immediately after Coinbase block isuance
    /**
     * release Reserved blocks which are used to record issuance of reserved PAIs in DAG
     * the reserves can be released after passing a certain time of creation the Originated Coinbase block and by a certain percentage of vote of shareholders of that coinbase block
     */
    const QString  RlBlock = "RlBlock"; // release reserved coins block
  }

  namespace DOC_TYPES
  {
    const QString Coinbase = "Coinbase";
    const QString RpDoc = "RpDoc"; // Repayment doc
    const QString RlDoc = "RlDoc"; // Release Reserved coins
    const QString BasicTx = "BasicTx";    // simple "m of n" trx

    // mimblewimble transactions
    const QString MWIngress = "MWIngress";    // move PAIs from basic transaction to a mimble transaction
    const QString MWTx = "MWIngress";    // a mimble transaction
    const QString MWEgress = "MWEgress";    // move back PAIs from a mimble transaction to a basic transaction

    // ZKP zk-snarks transactions
    const QString ZKIngress = "MWIngress";    // move PAIs from basic transaction to a Zero Knowladge Proof transaction
    const QString ZKTx = "MWIngress";    // a Zero Knowladge Proof transaction
    const QString ZKEgress = "MWEgress";    // move back PAIs from a Zero Knowladge Proof transaction to a basic transaction

    // confidential trx Monero-like
    const QString MNIngress = "MWIngress";    // move PAIs from basic transaction to a Zero Knowladge Proof transaction
    const QString MNTx = "MWIngress";    // a Zero Knowladge Proof transaction
    const QString MNEgress = "MWEgress";    // move back PAIs from a Zero Knowladge Proof transaction to a basic transaction

    // RGB (colored coins) Transactions
    const QString RGBIngress = "MWIngress";    // move PAIs from basic transaction to a Zero Knowladge Proof transaction
    const QString RGBTx = "MWIngress";    // a Zero Knowladge Proof transaction
    const QString RGBEgress = "MWEgress";    // move back PAIs from a Zero Knowladge Proof transaction to a basic transaction


    const QString DPCostPay = "DPCostPay";

    const QString DNAProposal = "DNAProposal";

    const QString AdmPolling = "AdmPolling";
    const QString ReqForRelRes = "ReqForRelRes";    // remove it to AdmPolling


    const QString Pledge = "Pledge";
    const QString ClosePledge = "ClosePledge";

    const QString Polling = "Polling";
    const QString Ballot = "Ballot";

    const QString FPost = "FPost"; // Custom Posts (files, Agora posts, wiki pages...)

    const QString INameReg = "INameReg"; // Flens: imagine flexible & largly extensible name service
    const QString INameBind = "INameBind"; // binds a iPGP(later GNU GPG) to an iName
    const QString INameMsgTo = "INameMsgTo"; // general message to a registered iName


  };

  // message parsing settings
  const uint MAX_PARSE_ATTEMPS_COUNT = 5; // parse message tentative


  const QString NO_EXT_HASH = "NEH";

  // floating blocks like floating signature, floating votes, collision logs...
  namespace FLOAT_BLOCKS_CATEGORIES
  {
    const QString Trx = "Trx";     // collision on spending a coin in some transactions
    const QString FleNS = "FleNS"; // collision on registring an iName
  }

  namespace DOCUMENT_TYPES {
    const QString Coinbase = "Coinbase";
    const QString RpDoc = "RpDoc"; // Repayment doc
    const QString RlDoc = "RlDoc"; // Release Reserved coins
    const QString BasicTx = "BasicTx";    // simple "m of n" trx

    // mimblewimble transactions
    const QString MWIngress = "MWIngress";    // move PAIs from basic transaction to a mimble transaction
    const QString MWTx = "MWIngress";    // a mimble transaction
    const QString MWEgress = "MWEgress";    // move back PAIs from a mimble transaction to a basic transaction

    // ZKP zk-snarks transactions
    const QString ZKIngress = "MWIngress";    // move PAIs from basic transaction to a Zero Knowladge Proof transaction
    const QString ZKTx = "MWIngress";    // a Zero Knowladge Proof transaction
    const QString ZKEgress = "MWEgress";    // move back PAIs from a Zero Knowladge Proof transaction to a basic transaction

    // confidential trx Monero-like
    const QString MNIngress = "MWIngress";    // move PAIs from basic transaction to a Zero Knowladge Proof transaction
    const QString MNTx = "MWIngress";    // a Zero Knowladge Proof transaction
    const QString MNEgress = "MWEgress";    // move back PAIs from a Zero Knowladge Proof transaction to a basic transaction

    // RGB (colored coins) Transactions
    const QString RGBIngress = "MWIngress";    // move PAIs from basic transaction to a Zero Knowladge Proof transaction
    const QString RGBTx = "MWIngress";    // a Zero Knowladge Proof transaction
    const QString RGBEgress = "MWEgress";    // move back PAIs from a Zero Knowladge Proof transaction to a basic transaction


    const QString DPCostPay = "DPCostPay";

    const QString DNAProposal = "DNAProposal";

    const QString AdmPolling = "AdmPolling";
    // ReqForRelRes = "ReqForRelRes";    remove it to AdmPolling


    const QString Pledge = "Pledge";
    const QString ClosePledge = "ClosePledge";

    const QString Polling = "Polling";
    const QString Ballot = "Ballot";

    const QString FPost = "FPost"; // Custom Posts (files; Agora posts; wiki pages...)

    const QString INameReg = "INameReg"; // Flens = imagine flexible & largly extensible name service
    const QString INameBind = "INameBind"; // binds a iPGP(later GNU GPG) to an iName
    const QString INameMsgTo = "INameMsgTo"; // general message to a registered iName

    const QString customDoc = "customDoc"; // custom usages
  };

  namespace FPOST_CLASSES
  {
    const QString File = "File";

    const QString DMS_RegAgora = "DMS_RegAgora";   // Demos register an Agora
    const QString DMS_Post = "DMS_Post";           // Demos Post

    const QString WK_CreatePage = "WK_CreatePage";
    const QString WK_EditPage = "WK_EditPage";
  };

  namespace TRX_CLASSES
  {
    const QString SimpleTx = "SimpleTx";    // simple "m of n" trx
    const QString P4P = "P4P"; // pay for payment
    const QString Bitcoin = "Bitcoin"; // Bitcoinish transaction to supporting swap transactions
  };

  namespace PLEDGE_CLOSE_CLASESS
  {
    const QString General = "General";    //General imagine DNA proposals
  };
  namespace PROPOSAL_CLASESS
  {
    const QString General = "General";    //General imagine DNA proposals
  };
  namespace BALLOT_CLASSES
  {
    const QString Basic = "Basic";    //Basic Ballot type
  };
  namespace GENERAL_CLASESS
  {
    const QString Basic = "Basic";    //Basic message
  };
  namespace INAME_CLASESS
  {
    const QString NoDomain = "NoDomain";       // NoDomains: initially imagine do not accept domain-like strings as an iName.
    const QString YesDomain = "YesDomain";      // TODO: after implementing a method to autorizing real clessical domain names owner(e.g. google.com) will be activate YesDomain too. TODO: add supporting unicode domains too
  };



  namespace MESSAGE_TAGS
  {
    const QString senderStartTag = "---START_SENDER_TAG---";
    const QString senderEndTag = "---END_SENDER_TAG---";
    const QString receiverStartTag = "---START_RECEIVER_TAG---";
    const QString receiverEndTag = "---END_RECEIVER_TAG---";
    const QString hashStartTag = "---START_HASH_TAG---";
    const QString hashEndTag = "---END_HASH_TAG---";

    const QString customEnvelopeTag = "CUSTOM ENVELOPE";
    const QString customStartEnvelope = "-----START_CUSTOM_ENVELOPE-----";
    const QString customEndEnvelope = "-----END_CUSTOM_ENVELOPE-----";
    const QString NO_ENCRYPTION = "NO-ENCRYPTION";

    const QString iPGPStartEnvelope = "-----START_iPGP_ENVELOPE-----";
    const QString iPGPEndEnvelope = "-----END_iPGP_ENVELOPE-----";

    const QString iPGPStartLineBreak = "(";
    const QString iPGPEndLineBreak = ")\n\r<br>";
  };

  namespace POLLING_REF_TYPE
  {
    const QString Proposal = "Proposal";
    const QString ReqForRelRes = "ReqForRelRes"; // Request For release Reserved
    const QString AdmPolling = "AdmPolling";
  };

  namespace PLEDGE_CLASSES
  {
    // self authorized pledge
    // in this class the pledger signs a contract to cut regularely from mentioned address. (usually the shareholder address by which received incomes from share dvidend)
    // the diference is in this class all 3 parties(pledger, pledgee, arbiter) can close contract whenever they want.
    // this service is usefull for all type of subscription services e.g. monthly payments for T.V. streaming or even better DAILY payment for using services
    // by activating field "applyableAfter" the contract can be realy closed after certain minute of concluding it.
    // e.g. customer must use atleast 3 month of service(in order to get discount)
    // TODO: implement it since it cost couple of hours to implementing
    const QString PledgeSA = "PledgeSA";

    // pledge class P is designated to pledge an account in order to getting loan to apply proposal for voting.
    // in return if proposal accepted by community part of incomes will repayed to loaner untile get paid all loan+interest
    // so, this class of pledge needs 2 signature in certain order, to be valid
    // 1. pledger, signs pledge request and repayment conditions
    // 2. pledgee, sign and accepts pledging and publish all proposal, pledge-contract and payment-transaction at once in one block
    // 3. arbiter, in case of existance, the arbiters sign pledge-contract and they publish ALL 3(proposal, pledge-contract and payment-transaction),
    //    surely by arbiters signature(arbiterSignsPledge) the pledge hash will be changed.
    const QString PledgeP = "PledgeP";

    // Zero-Knowledge Proof implementation of PledgeP TODO: implement it
    const QString PledgePZKP = "PledgePZKP";


    // gneric pledge in which pledger can pledge an address(either has sufficent income from DNA or not) and get a loan
    // and by time payback the loan to same account
    // of course there is no garanty to payback PAIs excep reputation of pledger or her bussiness account which is pledged
    // TODO: implement it ASAP
    const QString PledgeG = "PledgeG";


    // lending PAIs by pledging an account with sufficent income
    // TODO: implement it ASAP
    const QString PledgeL = "PledgeL";

  };


  const uint8_t PLDEGE_ACTIVATE_OR_DEACTIVATE_MATURATION_CYCLE_COUNT = 2;

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
    const QString ByPledgee = "ByPledgee";
    const QString ByArbiter = "ByArbiter";
    const QString ByPledger = "ByPledger";
    const QString ByNetwork = "ByNetwork";
  };

  namespace BINDING_TYPES
  {
      const QString IPGP = "IPGP";
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
  const QString DEFAULT_SIGNATURE_MOD = "2/3"; // needs 2 signature of 3

  namespace CARD_TYPES
  {
      const QString ProposalLoanRequest = "ProposalLoanRequest";
      const QString FullDAGDownloadRequest = "FullDAGDownloadRequest";
      const QString FullDAGDownloadResponse = "FullDAGDownloadResponse";
      const QString BallotsReceiveDates = "BallotsReceiveDates";
      const QString NodeStatusScreenshot = "NodeStatusScreenshot";
      const QString NodeStatusSnapshot = "NodeStatusSnapshot";

      const QString pleaseRemoveMeFromYourNeighbors = "pleaseRemoveMeFromYourNeighbors";
      const QString directMsgToNeighbor = "directMsgToNeighbor";
  };

  namespace MESSAGE_TYPES
  {
    const QString HANDSHAKE = "handshake";
    const QString NICETOMEETYOU = "niceToMeetYou";
    const QString HEREISNEWNEIGHBOR = "hereIsNewNeighbor";

    // TODO: move these commands to GQL format
    const QString DAG_INVOKE_LEAVES = "dagInvokeLeaves";
    const QString DAG_LEAVES_INFO = "dagLeavesInfo";
    const QString DAG_INVOKE_BLOCK = "dagInvokeBlock";
    const QString DAG_INVOKE_DESCENDENTS = "dagInvokeDescendents";
  };

  const QString OUTPUT_DPCOST = "OUTPUT_DPCOST";
  const QString OUTPUT_NORMAL = "OUTPUT_NORMAL";
  const QString OUTPUT_TREASURY = "OUTPUT_TREASURY";
  const QString OUTPUT_CHANGEBACK = "OUTPUT_CHANGEBACK";

  /**
   * @brief CURRENT_AES_VERSION
   * ver 0.0.0 to connect to Javascript clients
   * ver 0.2.0 to connect to C++ clients
   */
  const QString CURRENT_AES_VERSION = "0.2.0";

  const QString FAKE_RIGHT_HASH_PREFIX = "_";

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
    GenRes(const bool &st, const QString &ms="");
    bool status = false;
    QString msg = "";
};

#endif // CONTANTS_H

 */