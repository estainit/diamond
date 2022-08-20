use std::collections::HashMap;
use postgres::Client;
use crate::{ccrypto, cutils, dbhandler};
use crate::lib::constants::NETWORK_LAUNCH_DATE;
use crate::lib::custom_types::{CAddressT, CDateT, JSonObject, QSDicT, QVDRecordsT, VString};
use crate::lib::database::db_handler::{empty_db, get_connection, maybe_initialize_db};
use postgres::types::ToSql;
use serde_json::json;
use crate::constants::HD_ROOT_FILES;
use crate::lib::address::address_handler::create_a_new_address;
use crate::lib::constants;
use crate::lib::dag::dag_walk_through::get_latest_block_record;
use crate::lib::database::abs_psql::{q_select, q_upsert, simple_eq_clause};
use crate::lib::database::tables::{C_KVALUE, C_MACHINE_PROFILES};
use crate::lib::dlog::dlog;
use crate::lib::file_handler::file_handler::{mkdir, path_exist};
use crate::lib::k_v_handler::{get_value, set_value, upsert_kvalue};
use crate::lib::machine::dev_neighbors::dev_neighbors::{ALICE_PRIVATE_KEY, ALICE_PUBLIC_EMAIL, ALICE_PUPLIC_KEY, BOB_PRIVATE_KEY, BOB_PUBLIC_EMAIL, BOB_PUPLIC_KEY, EVE_PRIVATE_KEY, EVE_PUBLIC_EMAIL, EVE_PUPLIC_KEY, HU_PRIVATE_KEY, HU_PUBLIC_EMAIL, HU_PUPLIC_KEY, USER_PRIVATE_KEY, USER_PUBLIC_EMAIL, USER_PUPLIC_KEY};
use crate::lib::machine::machine_neighbor::{add_a_new_neighbor, NeighborInfo};
use crate::lib::machine::machine_profile::{EmailSettings, MachineProfile};
use crate::lib::services::initialize_node::maybe_init_dag;
use crate::lib::transactions::basic_transactions::signature_structure_handler::unlock_document::UnlockDocument;
use crate::lib::wallet::wallet_address_handler::{insert_address, WalletAddress};

//  '  '  '  '  '  '  '  '  '  '  '  '  '  '  '  machine_handler.cpp file
// #[derive(Default)]
pub struct CMachine {
    m_clone_id: i8,
    m_should_loop_threads: bool,

    pub m_is_db_connected: bool,
    pub m_is_db_initialized: bool,
    pub m_is_in_sync_process: bool,
    m_last_sync_status_check: CDateT,

    m_threads_status: QSDicT,
    m_map_thread_code_to_prefix: QSDicT,
    m_is_develop_mod: bool,

    m_develop_launch_date: CDateT,

    /*


      bool m_machine_is_loaded = false;
*/
    m_selected_profile: String,
    /*
      bool s_DAG_is_initialized;
      bool m_should_loop_threads = true;
      bool m_can_start_lazy_loadings = false;


  Config* m_global_configs {};
  */
    pub(crate) m_recorded_blocks_in_db: u32,
    // TODO: remove this variable(mechanism) after fixing sqlite database lock problem
    /*
      StringList m_cache_coins_visibility = {}; // TODO: remove this variable(mechanism) after fixing sqlite database lock problem and bloom filter implementation
    QVDRecordsT m_cache_spendable_coins = {}; // TODO: remove this variable(mechanism) after fixing sqlite database lock problem
*/
    pub m_dag_cached_blocks: QVDRecordsT,
    // TODO: optimize it ASAP
    pub m_dag_cached_block_hashes: Vec<String>,
    // TODO: optimize it ASAP
    pub(crate) m_profile: MachineProfile,

}
/*
pub trait CMachineThreadGaps {
    fn get_coinbase_import_gap(&self) -> TimeBySecT;
    fn get_block_invoke_gap(&self) -> TimeBySecT;
    fn get_nb_coins_import_gap(&mut self) -> TimeBySecT;
}
 */


impl CMachine {
    pub(crate) fn new() -> CMachine {
        // let (_status, profile) = MachineProfile::get_profile_from_db(constants::DEFAULT);
        CMachine {
            m_clone_id: 0,
            m_should_loop_threads: true,

            m_is_in_sync_process: true,

            m_last_sync_status_check: NETWORK_LAUNCH_DATE.to_string(),

            m_is_develop_mod: false,

            m_threads_status: HashMap::new(),
            m_map_thread_code_to_prefix: HashMap::new(),

            m_develop_launch_date: "".to_string(),

            m_selected_profile: "".to_string(),
            m_is_db_connected: false,
            m_is_db_initialized: false,

            /*
          const static String stb_machine_block_buffer;
          const static StringList stb_machine_block_buffer_fields;

          static const String stbl_machine_onchain_contracts;
          static const StringList stbl_machine_onchain_contracts_fields;

          Config* m_global_configs {};
          */
            m_recorded_blocks_in_db: 0, // TODO: remove this variable(mechanism) after fixing sqlite database lock problem
            /*
              StringList m_cache_coins_visibility = {}; // TODO: remove this variable(mechanism) after fixing sqlite database lock problem and bloom filter implementation
            QVDRecordsT m_cache_spendable_coins = {}; // TODO: remove this variable(mechanism) after fixing sqlite database lock problem
            QVDRecordsT m_dag_cached_blocks; // TODO: optimize it ASAP
            StringList m_dag_cached_block_hashes = {}; // TODO: optimize it ASAP

            MachineProfile m_profile;

              */
            m_dag_cached_blocks: vec![],
            m_dag_cached_block_hashes: vec![],
            m_profile: MachineProfile::new(),
        }
    }

    pub fn initialize_machine(&mut self) -> bool
    {
        self.create_folders();

        if self.get_app_clone_id() > 0
        {
            // change database
            println!(" connnnnnnnecting db to {}", self.get_app_clone_id());
            dbhandler().m_db = get_connection(self.get_app_clone_id());
        }

        self.m_last_sync_status_check = self.get_launch_date();

        // control DataBase
        let (status, msg) = maybe_initialize_db(self);
        if !status
        {
            panic!("failed on maybe initialize db {}", msg)
        }

        maybe_init_dag(self);

        //launchThreads();

        // doRegKeys();
        true
    }

    // func name was parseArgs
    pub fn parse_args(&mut self, args: VString, manual_clone_id: i8)
    {
        println!("Env args: {:?}", args);

        let mut clone_id: i8 = 0;
        let mut is_develop_mod: bool = false;

        if args.len() > 1 {
            clone_id = args[1].parse().unwrap();
        }

        if manual_clone_id > 0 {
            clone_id = manual_clone_id;
        }

        if args.len() > 2 {
            is_develop_mod = true;
        }

        self.set_clone_dev(clone_id, is_develop_mod);
    }

    // func name was setCloneDev
    pub fn set_clone_dev(&mut self, clone_id: i8, is_develop_mod: bool) -> bool
    {
        self.m_clone_id = clone_id;
        self.m_is_develop_mod = is_develop_mod;
        true
    }


    //func name was shouldLoopThreads
    pub fn should_loop_threads(&self) -> bool {
        println!("should_ loop_ threads > {:?}", self.m_should_loop_threads);
        self.m_should_loop_threads
    }


    /**
     * if the creationdate of latest recorded block in DAG is older than 2 cycle, so machine must go to synching mode
     * @param {*} args
     */
    //old_name_was isInSyncProcess
    pub fn is_in_sync_process(&mut self, force_to_control_based_on_dag_status: bool) -> bool
    {
        // return false;//FIXME: remove this line after finishing develop
        //  put last_sync_status in CMachine as a static member
        if !self.m_is_in_sync_process {
            return false;
        }

        if cutils::time_diff(
            self.m_last_sync_status_check.clone(),
            "".to_string()).as_minutes < 2 {
            return self.m_is_in_sync_process;
        }

        let mut last_sync_status = get_value("last_sync_status");
        if last_sync_status == "" {
            self.init_last_sync_status();
            last_sync_status = get_value("last_sync_status");
        }
        let mut last_sync_status_json_obj: JSonObject = cutils::parse_to_json_obj(&last_sync_status);


        let cycle_by_minutes = cutils::get_cycle_by_minutes();
        // control if the last status-check is still valid (is younger than 30 minutes?= 24 times in a cycle)
        if !force_to_control_based_on_dag_status &&
            (last_sync_status_json_obj["checkDate"].to_string() > cutils::minutes_before(
                cycle_by_minutes / 24,
                &cutils::get_now())) {
            let is_in_sync: bool = last_sync_status_json_obj["lastDAGBlockCreationDate"].to_string() < cutils::minutes_before(2 * cycle_by_minutes, &cutils::get_now());
            self.set_is_in_sync_process(is_in_sync, &cutils::get_now());
            return is_in_sync;
        } else {
            // re-check graph info&  update status-check info too
            let (status, block) = get_latest_block_record();
            if !status {
                panic!("No block in DAG exit!!");
            }

            let is_in_sync_process: bool = block.m_block_creation_date < cutils::minutes_before(2 * cycle_by_minutes, &cutils::get_now());

            if is_in_sync_process {
                last_sync_status_json_obj["isInSyncMode"] = "Y".into();
            } else {
                last_sync_status_json_obj["isInSyncMode"] = "N".into();
            }
            last_sync_status_json_obj["checkDate"] = cutils::get_now().into();
            last_sync_status_json_obj["lastDAGBlockCreationDate"] = block.m_block_creation_date.into();
            if is_in_sync_process {
                last_sync_status_json_obj["lastTimeMachineWasInSyncMode"] = cutils::get_now().into();
            }
            upsert_kvalue("last_sync_status", &cutils::serialize_json(&last_sync_status_json_obj), false);
            self.set_is_in_sync_process(is_in_sync_process, &cutils::get_now());
            return is_in_sync_process;
        }
    }

    //old_name_was setIsInSyncProcess
    pub fn set_is_in_sync_process(&mut self, status: bool, c_date: &CDateT)
    {
        self.m_is_in_sync_process = status;
        self.m_last_sync_status_check = c_date.clone();
    }

    //old_name_was reportThreadStatus
    pub fn report_thread_status(&mut self, thread_prefix: &String, thread_code: &String, thread_status: &String)
    {
        self.m_threads_status.insert((thread_prefix.to_string() + &thread_code).clone(), thread_status.clone());
        self.m_map_thread_code_to_prefix.insert(thread_code.clone(), thread_prefix.clone());
    }


    /*

    class MachineTransientBalances
    {
    public:
      CMPAIValueT m_one_cycle_issued = 0;
      uint64_t m_cycle_counts_from_began = 0;
      uint64_t m_distinct_coinbases_count = 0;
      CMPAIValueT m_total_minted_coins = 0;
      CMPAIValueT m_total_spendable_coins = 0;

      QVDRecordsT m_wallet_spendable_UTXOs {};

      CMPAIValueT m_cb_not_imported_sum = 0;
      QVDRecordsT m_cb_not_imported_processed_outputs = {};
      CMPAIValueT m_cb_not_imported_coinbase_value = 0;

      CMPAIValueT m_normal_not_imported_sum = 0;
      CMPAIValueT m_waited_treasury_incomes = 0;

      CMPAIValueT m_cb_floorished_coins = 0;

      CMPAISValueT m_final_balance = 0;
      CMPAIValueT m_wallet_wealth_value = 0;

      QVDRecordsT m_coins_existance = {};

    };




    struct CoinsVisibilityRes {
      bool status = false;
      StringList records = {};
      bool is_visible = false;
    };

      static void parseArgs(int argc, char *argv[], int manual_clone_id = 0);
      static void onAboutToQuit(MainWindow* w){ get().IonAboutToQuit(w); };



      // machine_handler.cpp
      GenRes initDefaultProfile();

      static bool getDAGIsInitialized(){ return get().s_DAG_is_initialized; }

      static std::tuple<bool, QVDRecordsT> cachedSpendableCoins(
        const String& action = "read",
        const QVDRecordsT& coins = {},
        const CBlockHashT& visible_by = "",
        const CCoinCodeT& the_coin = "") { return get().IcachedSpendableCoins(action, coins, visible_by, the_coin); };

      static CoinsVisibilityRes cachedCoinsVisibility(
        const String& action = "read",
        const StringList& entries = {}){ return get().IcachedCoinsVisibility(action, entries); };

      static bool shouldLoopThreads(){return get().IshouldLoopThreads();}
      static void setShouldLoopThreads(const bool v){return get().IsetShouldLoopThreads(v); }

      static bool canStartLazyLoadings(){return get().IcanStartLazyLoadings();}
      static void setCanStartLazyLoadings(bool v){ get().IsetCanStartLazyLoadings(v);}
      static String mapThreadCodeToPrefix(const String& code){ return get().ImapThreadCodeToPrefix(code);}

      static bool isGUIConnected(){ return get().IisGUIConnected(); }
      static void setIsGUIConnected(const bool status){ get().IsetIsGUIConnected(status); }

      bool loadSelectedProfile();

      static std::tuple<bool, String, UnlockSet, StringList> signByMachineKey(
        const String& signMsg,
        const CSigIndexT& unlockIndex = 0);


      static double getMinPollingTimeframeByHour();
      static TimeByHoursT getMinVotingTimeframe();

      static double getMachineServiceInterests(
        const String& dType,
        const String& dClass = "",
        const DocLenT& dLen = 0,
        const DocLenT& extra_length = 0,
        const int& supported_P4P_trx_count = 1)
      {
          return get().IgetMachineServiceInterests(
            dType,
            dClass,
            dLen,
            extra_length,
            supported_P4P_trx_count);
      }


      //  -  -  -  -  -  machine_backup.cpp


      //  -  -  -  -  -  machine_services_interests.cpp


      //  -  -  -  -  -  neighbors handler

      static std::tuple<bool, bool> parseHandshake(
        const String& sender_email,
        const JSonObject& message,
        const String& connection_type);

      static bool deleteNeighbors(
        const String& n_id,
        const String& connection_type,
        const String& mp_code = getSelectedMProfile()){return get().IdeleteNeighbors(n_id, connection_type, mp_code);}

      static std::tuple<bool, bool> parseNiceToMeetYou(
        const String& sender_email,
        const JSonObject& message,
        const String& connection_type);

      //  -  -  -  accounts balances
      static MachineTransientBalances machineBalanceChk();


      //  -  -  -  block buffer part
      static QVDRecordsT searchBufferedDocs(
        const ClausesT& clauses = {},
        const StringList& fields = stb_machine_block_buffer_fields,
        const OrderT& order = {},
        const uint64_t limit = 0);

      static std::tuple<bool, String> pushToBlockBuffer(
        const Document* doc,
        const CMPAIValueT dp_cost,
        const String& mp_code = getSelectedMProfile());

      static std::tuple<bool, String> broadcastBlock(
        const String& cost_pay_mode = "normal",
        const String& create_date_type = "");

      static std::tuple<bool, bool, String> fetchBufferedTransactions(
        Block* block,
        TransientBlockInfo& transient_block_info);

      static std::tuple<bool, bool, String> retrieveAndGroupBufferedDocuments(
        Block* block,
        TransientBlockInfo& transient_block_info);

      static bool removeFromBuffer(const ClausesT& clauses);


      //  -  -  -  on-chain contracts part
      static JSonArray searchInMyOnchainContracts(
        const ClausesT& clauses,
        const StringList& fields = stbl_machine_onchain_contracts_fields,
        const OrderT order = {},
        const uint64_t limit = 0);

      void IsetIsGUIConnected(const bool status)
      {
        m_is_GUI_connected = status;
      }

      bool IisGUIConnected()
      {
        return m_is_GUI_connected;
      }

      static double IgetMachineServiceInterests(
        const String& dType,
        const String& dClass = "",
        const DocLenT& dLen = 0,
        const DocLenT& extra_length = 0,
        const int& supported_P4P_trx_count = 1);
    */

    //old_name_was getAppCloneId
    pub fn get_app_clone_id(&self) -> i8
    {
        return self.m_clone_id;
    }
    /*
      void IsetShouldLoopThreads(const bool v)
      {
        m_should_loop_threads = v;
      }

      bool IshouldLoopThreads()
      {
        return m_should_loop_threads;
      }
*/

    //old_name_was isDevelopMod
    pub fn is_develop_mod(&self) -> bool
    {
        return self.m_is_develop_mod;
    }

    //old_name_was getPubEmailInfo
    pub fn get_pub_email_info(&self) -> &EmailSettings {
        return &self.m_profile.m_mp_settings.m_public_email;
    }

    //old_name_was getPrivEmailInfo
    pub fn get_priv_email_info(&self) -> &EmailSettings {
        return &self.m_profile.m_mp_settings.m_private_email;
    }

    //old_name_was saveSettings
    pub fn save_settings(&self) -> bool
    {
        // panic!("OOOOOOO2 self.m_profile.m_mp_settings.m_public_email.m_address: {}", self.m_profile.m_mp_settings.m_public_email.m_address);
        // &self.m_profile.m_mp_last_modified
        //     let mp_settings = b
        // let serialized_profile = json!({
        //     "m_mp_code": &self.m_profile.m_mp_code,
        //     "m_mp_name": &self.m_profile.m_mp_name,
        //     "m_mp_last_modified": &self.m_profile.m_mp_last_modified,
        //     "m_mp_settings": mp_settings});
        // panic!("serialized_profile {}", serialized_profile);
        let (status, serialized_settings) = match serde_json::to_string(&self.m_profile) {
            Ok(ser) => { (true, ser) }
            Err(e) => {
                dlog(
                    &format!("Failed in serialization machine profile: {:?}", e),
                    constants::Modules::App,
                    constants::SecLevel::Error);
                (false, "Failed in serialization machine profile!".to_string())
            }
        };
        if !status
        { return false; }

        println!("serialized_settings to be saved {}", serialized_settings);
        let values = HashMap::from([
            ("mp_code", &self.m_profile.m_mp_code as &(dyn ToSql + Sync)),
            ("mp_name", &self.m_profile.m_mp_name as &(dyn ToSql + Sync)),
            ("mp_settings", &serialized_settings as &(dyn ToSql + Sync)),
            ("mp_last_modified", &self.m_profile.m_mp_last_modified as &(dyn ToSql + Sync)),
        ]);

        return q_upsert(
            C_MACHINE_PROFILES,
            "mp_code",
            self.m_profile.m_mp_code.as_str(),
            &values,
            true);
    }
    /*


      std::tuple<bool, QVDRecordsT> IcachedSpendableCoins(
        const String& action,
        const QVDRecordsT& coins = {},
        const CBlockHashT& visible_by = "",
        const CCoinCodeT& the_coin = "");

      CoinsVisibilityRes IcachedCoinsVisibility(
        const String& action,
        const StringList& entries);

      //  -  -  -  -  -  neighbors handler

      bool IdeleteNeighbors(
        const String& n_id,
        const String& connection_type,
        const String& mp_code = getSelectedMProfile());

      void IonAboutToQuit(MainWindow* w);



      String ImapThreadCodeToPrefix(const String& code);

        };

    //  -  -  -  EmailSettings
    JSonObject EmailSettings::exportJson() const
    {
      return JSonObject
      {
        {"m_address", m_address},
        {"m_password", m_password},
        {"m_income_imap", m_income_imap},
        {"m_income_pop3", m_income_pop3},
        {"m_incoming_mail_server", m_incoming_mail_server},
        {"m_outgoing_mail_server", m_outgoing_mail_server},
        {"m_outgoing_smtp", m_outgoing_smtp},
        {"m_fetching_interval_by_minute", m_fetching_interval_by_minute}, // it depends on smtp server, but less than 5 minute is useless
        {"m_pgp_private_key", m_pgp_private_key},
        {"m_pgp_public_key", m_pgp_public_key}
      };
    }

    void EmailSettings::importJson(const JSonObject& obj)
    {
      m_address = obj["m_address"].to_string();
      m_password = obj["m_password"].to_string();
      m_income_imap = obj["m_income_imap"].to_string();
      m_income_pop3 = obj["m_income_pop3"].to_string();
      m_incoming_mail_server = obj["m_incoming_mail_server"].to_string();
      m_outgoing_mail_server = obj["m_outgoing_mail_server"].to_string();
      m_outgoing_smtp = obj["m_outgoing_smtp"].to_string();
      m_fetching_interval_by_minute = obj["m_fetching_interval_by_minute"].to_string();
      m_pgp_private_key = obj["m_pgp_private_key"].to_string();
      m_pgp_public_key = obj["m_pgp_public_key"].to_string();
    }




    //  -  -  -  MPSetting

    JSonObject MPSetting::exportJson() const
    {
      return JSonObject
      {
        {"m_machine_alias", m_machine_alias},
        {"m_language", m_language},
        {"m_term_of_services", m_term_of_services},
        {"m_machine_alias", m_machine_alias},
        {"m_backer_detail", m_backer_detail->exportJson()},
        {"m_public_email", m_public_email.exportJson()},
        {"m_private_email", m_private_email.exportJson()},
        {"m_already_presented_neighbors", m_already_presented_neighbors}
      };
    }


    */
    //old_name_was getLaunchDate
    pub fn get_launch_date(&self) -> String
    {
        if constants::NETWORK_LAUNCH_DATE != "" {
            return constants::NETWORK_LAUNCH_DATE.to_string();
        }
        return self.m_develop_launch_date.clone();
    }

    //old_name_was setLaunchDateAndCloneId
    pub fn set_launch_date_and_clone_id(&mut self, c_date: CDateT, clone_id: i8)
    {
        self.m_develop_launch_date = c_date;
        if clone_id != 0
        { self.m_clone_id = clone_id; }
    }

    //old_name_was initDefaultProfile
    pub fn init_default_profile(&mut self) -> (bool, String)
    {
        let (_status, profile) = MachineProfile::get_profile_from_db(&constants::DEFAULT);
        self.m_profile = profile;
        // panic!("elf.m_profile.m_mp_code {}", self.m_profile.m_mp_code);
        if self.m_profile.m_mp_code == constants::DEFAULT.to_string()
        {
            return (true, "The Default profile Already initialized".to_string());
        }

        // initializing default valuies and save it
        self.m_profile.m_mp_code = constants::DEFAULT.to_string();
        self.m_profile.m_mp_name = constants::DEFAULT.to_string();
        self.m_profile.m_mp_last_modified = cutils::get_now();

        {
            // initializing email PGP pair keys (it uses native node.js crypto lib. TODO: probably depricated and must refactor to use new one)
            let (status, private_pgp, public_pgp) = ccrypto::rsa_generate_key_pair(0);
            if !status {
                return (false, "Couldn't creat RSA key pairs (for private channel)".to_string());
            }
            self.m_profile.m_mp_settings.m_private_email.m_pgp_private_key = private_pgp;
            self.m_profile.m_mp_settings.m_private_email.m_pgp_public_key = public_pgp;
        }

        {
            // initializing email PGP pair keys (it uses native node.js crypto lib. TODO: probably depricated and must refactor to use new one)
            let (status, private_pgp, public_pgp) = ccrypto::rsa_generate_key_pair(0);
            if !status {
                return (false, "Couldn't creat RSA key pairs (for public channel)".to_string());
            }
            self.m_profile.m_mp_settings.m_public_email.m_pgp_private_key = private_pgp;
            self.m_profile.m_mp_settings.m_public_email.m_pgp_public_key = public_pgp;
        }

        let (status, unlock_doc) = create_a_new_address(
            constants::signature_types::STRICT,
            "2/3",
            "0.0.1");
        if !status
        {
            return (false, "Couldn't creat ECDSA key pairs (for public channel)".to_string());
        }

        self.m_profile.m_mp_settings.m_machine_alias = "node-".to_owned() + &cutils::hash6c(&ccrypto::keccak256(&unlock_doc.m_account_address));
        self.m_profile.m_mp_settings.m_backer_detail = unlock_doc;

        self.save_settings();


        // set selected profile=default
        let values = HashMap::from([
            ("kv_value", &self.m_profile.m_mp_code as &(dyn ToSql + Sync)),
            ("kv_last_modified", &self.m_profile.m_mp_last_modified as &(dyn ToSql + Sync)),
        ]);
        q_upsert(
            C_KVALUE,
            "kv_key",
            "selected_profile",
            &values,
            true,
        );

        // add backer address to wallet as well
        let w_address = WalletAddress::new(
            &self.m_profile.m_mp_settings.m_backer_detail,
            constants::DEFAULT.to_string(),   // mp code
            "Backer Address (".to_owned() +
                &self.m_profile.m_mp_settings.m_backer_detail.m_unlock_sets[0].m_signature_type + &" ".to_owned() +
                &self.m_profile.m_mp_settings.m_backer_detail.m_unlock_sets[0].m_signature_ver + &")".to_owned(),
            cutils::get_now(),
        );
        let (_status, _msg) = insert_address(&w_address);

        self.maybe_add_seed_neighbors();

        return (true, "The Default profile initialized".to_string());
    }

    //old_name_was bootMachine
    pub fn boot_machine(&mut self) -> bool
    {
        let clone_id = self.get_app_clone_id();

        let mut devel_msg: String = "".to_string();
        if self.is_develop_mod() {
            devel_msg = " (develop mode)".to_string();
        }

        let mut sync_msg: String = "".to_string();
        if self.is_in_sync_process(false) {
            sync_msg = " Syncing".to_string();
        }

        if clone_id > 0 {
            println!("Launched machine({}){}{}", clone_id, devel_msg, sync_msg);
        } else {
            println!("Launched machine{}{}", devel_msg, sync_msg);
        }
        dlog(
            &format!("Booting App({})", clone_id),
            constants::Modules::App,
            constants::SecLevel::Info);

        // check if flag machine_and_dag_are_safely_initialized is setted
        let is_inited = get_value("machine_and_dag_are_safely_initialized");
        if is_inited != constants::YES
        {
            empty_db(self);  // machine didn't initialized successfully, so empty all tables and try from zero
            set_value("machine_and_dag_are_safely_initialized", constants::NO, true);
        }

        let (status, _msg) = self.init_default_profile();
        if status != true {
            return false;
        }

        /*

              // load machine settings
              loadSelectedProfile();

              m_machine_is_loaded = true;
              s_DAG_is_initialized = true;


              {
                // remove this block after improving db and fixing database block problem
                auto[status, coins] = cachedSpendableCoins("read");
                if (!status)
                {
                  CLog::log("couldn't read from cached Spendable Coins!", "app", "fatal");
                }

                if (coins.len() < 500)
                {
                  QueryRes exist_coins = DbModel::select(
                    "c_trx_utxos",
                    {"ut_coin", "ut_creation_date", "ut_ref_creation_date", "ut_visible_by", "ut_o_address", "ut_o_value"});
                  cachedSpendableCoins("assign", exist_coins.records);
                }
              }
            */
        return true;
    }
    /*

    void CMachine::parseArgs(int argc, char *argv[], int manual_clone_id)
    {
      qDebug() << argc;
      for (int param_inx = 0; param_inx < argc; param_inx++)
        qDebug() << param_inx << " " << argv[param_inx];

      uint8_t clone_id = 0;    // FIXME: this value must be defined by command line
      if (argc > 1)
        clone_id = atoi(argv[1]);

      if (manual_clone_id > 0)
        clone_id = manual_clone_id;

      bool is_develop_mod = false;
      if (argc > 2)
        is_develop_mod = true;

    //  clone_id = 1;
      qDebug() << " clone_id " << clone_id;

      setCloneDev(clone_id, is_develop_mod);

    }

*/
    //old_name_was getHDPath
    pub fn get_clone_path(&self) -> String
    {
        if self.get_app_clone_id() == 0
        {
            return HD_ROOT_FILES.to_string();
        }
        return format!("{}/{}", HD_ROOT_FILES, self.get_app_clone_id());
    }


    //old_name_was getReportsPath
    pub fn get_reports_path(&self) -> String
    {
        return self.get_clone_path() + &"/reports";
    }

    //old_name_was getInboxPath
    pub fn get_inbox_path(&self) -> String
    {
        return self.get_clone_path() + &"/inbox";
    }

    //old_name_was getOutboxPath
    pub fn get_outbox_path(&self) -> String
    {
        return self.get_clone_path() + &"/outbox";
    }

    //old_name_was getReportsPath
    pub fn get_logs_path(&self) -> String
    {
        return self.get_clone_path() + &"/logs";
    }

    //old_name_was getCachePath
    pub fn get_cache_path(&self) -> String
    {
        return self.get_clone_path() + &"/cache_files";
    }

    //old_name_was getDAGBackup
    pub fn get_dag_backup(&self) -> String { return self.get_clone_path() + &"/dag_backup"; }

    //old_name_was createFolders
    pub fn create_folders(&self) -> bool
    {
        if constants::HD_ROOT_FILES != ""
        {
            if !path_exist(&constants::HD_ROOT_FILES.to_string())
            { mkdir(&constants::HD_ROOT_FILES.to_string()); }
        }

        if !path_exist(&self.get_clone_path())
        { mkdir(&self.get_clone_path()); }

        if !path_exist(&self.get_reports_path())
        { mkdir(&self.get_reports_path()); }

        if !path_exist(&self.get_inbox_path())
        { mkdir(&self.get_inbox_path()); }

        if !path_exist(&self.get_outbox_path())
        { mkdir(&self.get_outbox_path()); }

        if !path_exist(&self.get_dag_backup())
        { mkdir(&self.get_dag_backup()); }

        if !path_exist(&self.get_cache_path())
        { mkdir(&self.get_cache_path()); }

        return true;
    }

    //old_name_was getBackerAddress
    pub fn get_backer_address(&self) -> CAddressT
    {
        self.m_profile.m_mp_settings.m_backer_detail.m_account_address.clone()
    }

    //old_name_was getBackerDetails
    pub fn get_backer_details(&self) -> &UnlockDocument
    {
        return &self.m_profile.m_mp_settings.m_backer_detail;
    }


    //old_name_was getProfile
    pub fn get_profile(&self) -> MachineProfile
    {
        return self.m_profile.clone();
    }

    //old_name_was loadSelectedProfile
    pub fn load_selected_profile(&mut self) -> bool
    {
        let selected_prof = get_value("selected_profile");
        if selected_prof == "" {
            return false;
        }

        let mp: MachineProfile = self.read_profile(selected_prof);
        self.m_profile = mp;
        return true;


        // importJson(&self, profile: MachineProfile)
        // {
        //     m_machine_alias = obj["m_machine_alias"].to_string();
        //     m_language = obj["m_language"].to_string();
        //     m_term_of_services = obj["m_term_of_services"].to_string();
        //     m_machine_alias = obj["m_machine_alias"].to_string();
        //     m_already_presented_neighbors = obj["m_already_presented_neighbors"].toArray();
        //     m_backer_detail = new UnlockDocument();
        //     m_backer_detail->importJson(obj["m_backer_detail"].toObject());
        //     m_public_email.importJson(obj["m_public_email"].toObject());
        //     m_private_email.importJson(obj["m_private_email"].toObject());
        // }
    }

    pub fn read_profile(&self, mp_code: String) -> MachineProfile
    {
        let (_status, records) = q_select(
            C_MACHINE_PROFILES,
            vec!["mp_code", "mp_name", "mp_settings"],
            vec![
                simple_eq_clause("mp_code", &*mp_code)],
            vec![],
            0,
            true,
        );
        if records.len() != 1
        {
            return MachineProfile::new();
        }

        let serialized_profile = records[0].get("mp_settings").unwrap().clone();
        let profile: MachineProfile = serde_json::from_str(serialized_profile.as_str()).unwrap();
        return profile;
    }

    //old_name_was getSelectedMProfile
    pub fn get_selected_m_profile(&mut self) -> String
    {
        if self.m_selected_profile != ""
        {
            return self.m_selected_profile.clone();
        }

        let mp_code: String = get_value("selected_profile");
        // console.log('resresresresres', res);
        if mp_code == "" {
            dlog(
                &format!("NoooOOOOOOOOOOOOOOooooooooooooo profile!"),
                constants::Modules::App,
                constants::SecLevel::Fatal);

            panic!("NoooOOOOOOOOOOOOOOooooooooooooo profile!");
        }
        self.m_selected_profile = mp_code;
        return self.m_selected_profile.clone();
    }

    /*


    TimeByHoursT CMachine::getMinVotingTimeframe()
    {
      TimeByHoursT voting_timeframe = (cutils::get_cycle_by_minutes() * 2) / 60;   // at least 2 cycle for voting
      if (voting_timeframe == static_cast<uint64_t>(voting_timeframe))
        return voting_timeframe;
      return cutils::customFloorFloat(static_cast<double>(voting_timeframe), 2);
    }

    bool CMachine::IsetPublicEmailAddress(const String&  v)
    {
      m_profile.m_mp_settings.m_public_email.m_address = v;
      return true;
    }

    bool CMachine::IsetPublicEmailInterval(const String&  v)
    {
      m_profile.m_mp_settings.m_public_email.m_fetching_interval_by_minute = v;
      return true;
    }

    bool CMachine::IsetPublicEmailIncomeHost(const String&  v)
    {
      m_profile.m_mp_settings.m_public_email.m_incoming_mail_server = v;
      return true;
    }

    bool CMachine::IsetPublicEmailPassword(const String&  v)
    {
      m_profile.m_mp_settings.m_public_email.m_password = v;
      return true;
    }

    bool CMachine::IsetPublicEmailIncomeIMAP(const String&  v)
    {
      m_profile.m_mp_settings.m_public_email.m_income_imap = v;
      return true;
    }

    bool CMachine::IsetPublicEmailIncomePOP(const String&  v)
    {
      m_profile.m_mp_settings.m_public_email.m_income_pop3 = v;
      return true;
    }

    bool CMachine::IsetPublicEmailOutgoingSMTP(const String&  v)
    {
      m_profile.m_mp_settings.m_public_email.m_outgoing_smtp = v;
      return true;
    }

    bool CMachine::IsetPublicEmailOutgoingHost(const String&  v)
    {
      m_profile.m_mp_settings.m_public_email.m_outgoing_mail_server = v;
      return true;
    }

    bool CMachine::IsetTermOfServices(const String&  v)
    {
      m_profile.m_mp_settings.m_term_of_services = v;
      return true;
    }
*/
    //old_name_was getLastSyncStatus
    pub fn get_last_sync_status(&self) -> JSonObject
    {
        let mut last_sync_status: String = get_value("last_sync_status");
        if last_sync_status == ""
        {
            self.init_last_sync_status();
            last_sync_status = get_value("last_sync_status");
        }
        return cutils::parse_to_json_obj(&last_sync_status);
    }

    //old_name_was initLastSyncStatus
    pub fn init_last_sync_status(&self) -> bool
    {
        let last_sync_status: JSonObject = json!({
              "isInSyncMode": "Unknown",
              "lastTimeMachineWasInSyncMode":
                          cutils::minutes_before(cutils::get_cycle_by_minutes() * 2, &cutils::get_now()),
              "checkDate": cutils::minutes_before(cutils::get_cycle_by_minutes(), &cutils::get_now()),
              "lastDAGBlockCreationDate": "Unknown"
            });
        return upsert_kvalue(
            "last_sync_status",
            &cutils::serialize_json(&last_sync_status),
            true);
    }
    /*


        /**
         * @brief CMachine::signByMachineKey
         * @param sign_message
         * @param unlock_index
         * @return {status, signer address, unlock set, signatures}
         */
        std::tuple<bool, String, UnlockSet, StringList> CMachine::signByMachineKey(
          const String& sign_message,
          const CSigIndexT& unlock_index)
        {
          String signer = getBackerAddress();
          auto[sign_status, res_msg, sign_signatures, sign_unlock_set] = Wallet::signByAnAddress(
            signer,
            sign_message,
            unlock_index);
          if (!sign_status)
          {
            return {false, "", {}, {}};
          }

          UnlockSet uSet {};
          uSet.importJson(sign_unlock_set);
          return {true, signer, uSet, sign_signatures};

        }
    */

    /*

    void CMachine::IonAboutToQuit(MainWindow* w)
    {
      setShouldLoopThreads(false);
      bool any_thread_is_runing = true;
      int i = 0;

      for (String a_thread: m_threads_status.keys())
        if (m_threads_status[a_thread] == constants::THREAD_STATE::SLEEPING)
          CLog::log("Gracefully stopped sleeping thread(" + a_thread + ")");

      while (any_thread_is_runing && (i < 3000))
      {
        i++;
        std::this_thread::sleep_for(std::chrono::seconds(1));
        any_thread_is_runing = false;
        for (String a_thread: m_threads_status.keys())
          if (m_threads_status[a_thread] == constants::THREAD_STATE::RUNNING)
          {
            if ((i > 10) && (i%5==0))
              CLog::log("The thread (" + a_thread + ") still is running!");

            any_thread_is_runing = true;
          }
      }

      if (w)
        w->saveConfigurationParameters();

      CLog::log("Preparing to shout down...");
      m_global_configs->save();

      delete m_global_configs;

      DbHandler::closeConnections(); //TODO: use delete &DbHandler::get();

      m_cache_coins_visibility = StringList {};

      CLog::log("Gracefully shouted down");
    }


    String CMachine::ImapThreadCodeToPrefix(const String& code)
    {
      if (m_map_thread_code_to_prefix.keys().contains(code))
        return m_map_thread_code_to_prefix.value(code);
      return "Un-assigned thread(" + code + ")!";
    }

    //bool CMachine::IaddToCachedBlocks(const QVDicT& block)
    //{
    //  try {
    //    // using a local lock_guard to lock mtx guarantees unlocking on destruction / exception:
    //    std::lock_guard<std::mutex> lck ();
    //    m_dag_cached_blocks.push(block);
    //    return true;
    //  }
    //  catch (std::logic_error&) {
    //    std::cout << "[exception caught]\n";
    //    return false;
    //  }
    //}

*/
    //old_name_was cachedBlocks
    pub fn cached_blocks(
        &mut self,
        action: &str,
        blocks: QVDRecordsT,
        status: &String) -> bool
    {
        {
            if action == "assign" {
                self.m_dag_cached_blocks = blocks;
            } else if action == "append" {
                for a_block in blocks {
                    self.m_dag_cached_blocks.push(a_block);
                }
            } else if action == "update" {
                for a_block in blocks {
                    for i in 0..self.m_dag_cached_blocks.len() {
                        if self.m_dag_cached_blocks[i]["b_hash"].to_string() == a_block["b_hash"].to_string() {
                            // self.m_dag_cached_blocks[i]["b_utxo_imported"] = status.to_string();
                        }
                    }
                }
            }
            return true;
        }
        // catch (std::logic_error&)
        // {
        //     String thread_code = String::number((quint64)QThread::currentThread(), 16);
        //     CLog::log("Failed in cached blocks action(" + action + ") Thread(" + thread_code + " / " + mapThreadCodeToPrefix(thread_code)+ ")");
        //     std::cout << "[exception caught]\n";
        //     return {false, m_dag_cached_blocks};
        //   }
    }

    //old_name_was cachedBlockHashes
    pub fn cached_block_hashes(
        &mut self,
        action: &str,
        block_hashes: &Vec<String>) -> bool
    {
        // using a local lock_guard to lock mtx guarantees unlocking on destruction / exception:
        // std::lock_guard<std::mutex> lck (mutex_cached_block_hashes);

        if action == "assign" {
            self.m_dag_cached_block_hashes = block_hashes.clone();
        }

        if action == "append" {
            for a_hash in block_hashes {
                self.m_dag_cached_block_hashes.push(a_hash.clone());
            }
        }

        return true;

        // catch (std::logic_error&)
        // {
        //          String thread_code = String::number((quint64)QThread::currentThread(), 16);
        //          CLog::log("Failed in cached block hashes action(" + action + ") Thread(" + thread_code + " / " + mapThreadCodeToPrefix(thread_code)+ ")");
        //          std::cout << "[exception caught]\n";
        //          return {false, m_dag_cached_block_hashes};
        //        }
    }

    /*


       std::tuple<bool, QVDRecordsT> CMachine::IcachedSpendableCoins(
         const String& action,
         const QVDRecordsT& coins,
         const CBlockHashT& visible_by,
         const CCoinCodeT& the_coin)
       {
         try {
           // using a local lock_guard to lock mtx guarantees unlocking on destruction / exception:
           std::lock_guard<std::mutex> lck (mutex_cached_spendable_coins);

           if (action == "assign")
           {
             m_cache_spendable_coins = coins;
           }

           if (action == "append")
           {
             for (QVDicT coin: coins)
               m_cache_spendable_coins.push(coin);
           }

           if (action == "remove")
           {
             QVDRecordsT remined_coins = {};
             if ((visible_by != "") || (the_coin != ""))
             {
               for (QVDicT a_coin: m_cache_spendable_coins)
               {
                 if ((visible_by != "") && (the_coin != ""))
                 {
                   if ((a_coin["ut_visible_by"].to_string() != visible_by) || (a_coin["ut_coin"].to_string() != the_coin))
                     remined_coins.push(a_coin);

                 }
                 else if (visible_by != "")
                 {
                   if (a_coin["ut_visible_by"].to_string() != visible_by)
                     remined_coins.push(a_coin);
                 }
                 else if (the_coin != "")
                 {
                   if (a_coin["ut_coin"].to_string() != the_coin)
                     remined_coins.push(a_coin);
                 }
               }

               m_cache_spendable_coins = remined_coins;
             }
           }

           return {true, m_cache_spendable_coins};

         }
         catch (std::logic_error&)
         {
           String thread_code = String::number((quint64)QThread::currentThread(), 16);
           CLog::log("Failed in cached spendable coins action(" + action + ") Thread(" + thread_code + " / " + mapThreadCodeToPrefix(thread_code)+ ")");
           std::cout << "[exception caught]\n";
           return {false, m_cache_spendable_coins};
         }
       }

       CoinsVisibilityRes CMachine::IcachedCoinsVisibility(
         const String& action,
         const StringList& entries)
       {
         try {
           // using a local lock_guard to lock mtx guarantees unlocking on destruction / exception:
           std::lock_guard<std::mutex> lck (mutex_cached_coins_visibility);

           bool contains = true;

           if (action == "assign")
           {
             m_cache_coins_visibility = entries;
           }

           if (action == "append")
           {
             for (String a_visiblity: entries)
               m_cache_coins_visibility.push(a_visiblity);
           }

           if (action == "contains")
           {
             contains = m_cache_coins_visibility.contains(entries[0]);
           }

           return CoinsVisibilityRes {true, m_cache_coins_visibility, contains};

         }
         catch (std::logic_error&)
         {
           String thread_code = String::number((quint64)QThread::currentThread(), 16);
           CLog::log("Failed in cached spendable coins action(" + action + ") Thread(" + thread_code + " / " + mapThreadCodeToPrefix(thread_code)+ ")");
           std::cout << "[exception caught]\n";
           return CoinsVisibilityRes {false, m_cache_coins_visibility, false};
         }
       }


       double CMachine::getMinPollingTimeframeByHour()
       {
         return (cutils::get_cycle_by_minutes() * 2.0) / 60.0;
       }

        */

    pub fn maybe_add_seed_neighbors(&mut self) -> bool
    {
        let (status, msg) = add_a_new_neighbor(
            "seed1@seed.pro".to_string(),
            constants::PUBLIC.to_string(),
            "-----BEGIN PUBLIC KEY-----\nMIICIjANBgkqhkiG9w0BAQEFAAOCAg8AMIICCgKCAgEA1RG+nLSuYWszuVQL9ZaJ\nMflUZXlGfPKk+tmFxUnEGLKG4/QuTN/1m/Bm6AkFnHZkXWhGisyHG8ujgSAQZQnK\nUWsI+VGJ41YnqvxAKYIL3qvBSPLo8ppvN21tr7pbCL3uR0isHjXSp6XGH3mVBTd6\ntaJhRBtuQKdeFd3QMZCyofnaagA1wPHtT8wCz4X+7LckrfSRGhdjPUoT3pZ2R3Z8\noyAOtBzr7IRHDObs11Z5sdFmZVRQV1iSgxZyS3jEYjMqZN5FaxVYLq64MHIEYIdw\nLpofmWqkDrKUws9jTmiirmDfaAqsu6siHdCbCpnV026QMtbQukguJv3UFbdN/lh2\n2Obz9OKw802xMSgt4nULDSAvt8mrJsbyWbX66yVNkmN3OKiy36Ig9faCoxJTjzjW\nS5Kr7JXcBCyavog1n0NcNCOApde3TsoHNAt/5GJ8pMON2jG+i58Ug4/1mtz1tYEs\ndKFj4tbAVXgNPKNl0MlmReSjFati3K8H14twvOLsN0wnycWqJThwFCRFRfSV2weY\nw1w+k4hmsL0FvbZtl0OdQePvqbTQQTQc8SROc3Ejq/04oyc5S9D1MdaDEfdXqcqk\nnFzc3u3rzw1BPGdkw0LoiwDjp0WOSSB5u5NRI9UYxDOWdTaEPGpChKycm4kgUjYK\nvucjKoPGeLsBGmH8+NRT+RsCAwEAAQ==\n-----END PUBLIC KEY-----\n".to_string(),
            constants::DEFAULT.to_string(),
            constants::YES.to_string(),
            NeighborInfo::new(),
            cutils::get_now());
        dlog(
            &format!("result of add a new neighbor({}): {}", status, msg),
            constants::Modules::App,
            constants::SecLevel::Info);


        if self.is_develop_mod() {
            // this block existed ONLY for test and development environment

            let clone_id = self.get_app_clone_id();
            println!("Machine is in Dev mode, so make some neighborhoods! clone({})", clone_id);
            if [0, 1, 2, 3].contains(&clone_id) {
                println!("Machine is a fake neighbor, so make some connections!");

                if clone_id == 0 {
                    // update machine settings to dev mode settings (user@imagine.com)
                    println!("Setting machine as a developing user node");
                    self.m_profile.m_mp_settings.m_machine_alias = "node-user".to_string();
                    self.m_profile.m_mp_settings.m_public_email.m_pgp_private_key = USER_PRIVATE_KEY.to_string();
                    self.m_profile.m_mp_settings.m_public_email.m_pgp_public_key = USER_PUPLIC_KEY.to_string();
                    self.m_profile.m_mp_settings.m_public_email.m_address = USER_PUBLIC_EMAIL.to_string();
                    println!("OOOOOOO xx self.m_profile.m_mp_settings.m_public_email.m_address: {}", self.m_profile.m_mp_settings.m_public_email.m_address);

                    // add Hu as a neighbor
                    add_a_new_neighbor(
                        HU_PUBLIC_EMAIL.to_string(),
                        constants::PUBLIC.to_string(),
                        HU_PUPLIC_KEY.to_string(),
                        constants::DEFAULT.to_string(),
                        constants::YES.to_string(),
                        NeighborInfo::new(),
                        cutils::get_now());

                    // add Eve as a neighbor
                    add_a_new_neighbor(
                        EVE_PUBLIC_EMAIL.to_string(),
                        constants::PUBLIC.to_string(),
                        EVE_PUPLIC_KEY.to_string(),
                        constants::DEFAULT.to_string(),
                        constants::YES.to_string(),
                        NeighborInfo::new(),
                        cutils::get_now());

                    self.save_settings();
                    println!("OOOOOOO yy self.m_profile.m_mp_settings.m_public_email.m_address: {}", self.m_profile.m_mp_settings.m_public_email.m_address);

                } else if clone_id == 1
                {
                    // set profile as hu@imagine.com
                    self.m_profile.m_mp_settings.m_machine_alias = "node-hu".to_string();
                    self.m_profile.m_mp_settings.m_public_email.m_pgp_private_key = HU_PRIVATE_KEY.to_string();
                    self.m_profile.m_mp_settings.m_public_email.m_pgp_public_key = HU_PUPLIC_KEY.to_string();
                    self.m_profile.m_mp_settings.m_public_email.m_address = HU_PUBLIC_EMAIL.to_string();

                    // add user as a neighbor
                    add_a_new_neighbor(
                        USER_PUBLIC_EMAIL.to_string(),
                        constants::PUBLIC.to_string(),
                        USER_PUPLIC_KEY.to_string(),
                        constants::DEFAULT.to_string(),
                        constants::YES.to_string(),
                        NeighborInfo::new(),
                        cutils::get_now());

                    // add Eve as a neighbor
                    add_a_new_neighbor(
                        EVE_PUBLIC_EMAIL.to_string(),
                        constants::PUBLIC.to_string(),
                        EVE_PUPLIC_KEY.to_string(),
                        constants::DEFAULT.to_string(),
                        constants::YES.to_string(),
                        NeighborInfo::new(),
                        cutils::get_now());

                    println!("OOOOOOO1 self.m_profile.m_mp_settings.m_public_email.m_address: {}", self.m_profile.m_mp_settings.m_public_email.m_address);

                    self.save_settings();
                } else if self.m_clone_id == 2
                {
                    // set profile as eve@imagine.com
                    self.m_profile.m_mp_settings.m_machine_alias = "node-eve".to_string();
                    self.m_profile.m_mp_settings.m_public_email.m_address = EVE_PUBLIC_EMAIL.to_string();
                    self.m_profile.m_mp_settings.m_public_email.m_pgp_private_key = EVE_PRIVATE_KEY.to_string();
                    self.m_profile.m_mp_settings.m_public_email.m_pgp_public_key = EVE_PUPLIC_KEY.to_string();

                    // add User as a neighbor
                    add_a_new_neighbor(
                        USER_PUBLIC_EMAIL.to_string(),
                        constants::PUBLIC.to_string(),
                        USER_PUPLIC_KEY.to_string(),
                        constants::DEFAULT.to_string(),
                        constants::YES.to_string(),
                        NeighborInfo::new(),
                        cutils::get_now());

                    // add Hu as a neighbor
                    add_a_new_neighbor(
                        HU_PUBLIC_EMAIL.to_string(),
                        constants::PUBLIC.to_string(),
                        HU_PUPLIC_KEY.to_string(),
                        constants::DEFAULT.to_string(),
                        constants::YES.to_string(),
                        NeighborInfo::new(),
                        cutils::get_now());

                    // add Bob as a neighbor
                    add_a_new_neighbor(
                        HU_PUBLIC_EMAIL.to_string(),
                        constants::PUBLIC.to_string(),
                        BOB_PUPLIC_KEY.to_string(),
                        constants::DEFAULT.to_string(),
                        constants::YES.to_string(),
                        NeighborInfo::new(),
                        cutils::get_now());

                    self.save_settings();
                } else if self.m_clone_id == 3
                {
                    // set profile as bob@imagine.com
                    self.m_profile.m_mp_settings.m_machine_alias = "node-bob".to_string();
                    self.m_profile.m_mp_settings.m_public_email.m_address = BOB_PUBLIC_EMAIL.to_string();
                    self.m_profile.m_mp_settings.m_public_email.m_pgp_private_key = BOB_PRIVATE_KEY.to_string();
                    self.m_profile.m_mp_settings.m_public_email.m_pgp_public_key = BOB_PUPLIC_KEY.to_string();

                    // add Eve as a neighbor
                    add_a_new_neighbor(
                        EVE_PUBLIC_EMAIL.to_string(),
                        constants::PUBLIC.to_string(),
                        EVE_PUPLIC_KEY.to_string(),
                        constants::DEFAULT.to_string(),
                        constants::YES.to_string(),
                        NeighborInfo::new(),
                        cutils::get_now());

                    // add Alice as a neighbor
                    add_a_new_neighbor(
                        ALICE_PUBLIC_EMAIL.to_string(),
                        constants::PUBLIC.to_string(),
                        ALICE_PUPLIC_KEY.to_string(),
                        constants::DEFAULT.to_string(),
                        constants::YES.to_string(),
                        NeighborInfo::new(),
                        cutils::get_now());

                    self.save_settings();
                } else if self.m_clone_id == 4
                {
                    // set profile as alice@imagine.com
                    self.m_profile.m_mp_settings.m_machine_alias = "node-alice".to_string();
                    self.m_profile.m_mp_settings.m_public_email.m_address = ALICE_PUBLIC_EMAIL.to_string();
                    self.m_profile.m_mp_settings.m_public_email.m_pgp_private_key = ALICE_PRIVATE_KEY.to_string();
                    self.m_profile.m_mp_settings.m_public_email.m_pgp_public_key = ALICE_PUPLIC_KEY.to_string();

                    // add Hu as a neighbor
                    add_a_new_neighbor(
                        HU_PUBLIC_EMAIL.to_string(),
                        constants::PUBLIC.to_string(),
                        HU_PUPLIC_KEY.to_string(),
                        constants::DEFAULT.to_string(),
                        constants::YES.to_string(),
                        NeighborInfo::new(),
                        cutils::get_now());

                    // add Bob as a neighbor
                    add_a_new_neighbor(
                        BOB_PUBLIC_EMAIL.to_string(),
                        constants::PUBLIC.to_string(),
                        BOB_PUPLIC_KEY.to_string(),
                        constants::DEFAULT.to_string(),
                        constants::YES.to_string(),
                        NeighborInfo::new(),
                        cutils::get_now());

                    self.save_settings();
                }
            }
        }

        return true;
    }
}