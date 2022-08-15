// use std::sync::{Arc, Mutex};

use std::collections::HashMap;
use crate::{ccrypto, cutils};
use crate::lib::constants::LAUNCH_DATE;
// use crate::lib::constants as cconsts;
use crate::lib::custom_types::{CAddressT, CDateT, JSonObject, QSDicT, QVDRecordsT, VString};
use crate::lib::database::db_handler::{empty_db, init_db};
use postgres::Client;
use postgres::types::ToSql;
use serde_json::json;
use crate::constants::HD_ROOT_FILES;
use crate::lib::address::address_handler::create_a_new_address;
use crate::lib::constants;
use crate::lib::database::abs_psql::{q_select, q_upsert, simple_eq_clause};
use crate::lib::database::tables::{STBL_KVALUE, STBL_MACHINE_PROFILES};
use crate::lib::dlog::dlog;
use crate::lib::file_handler::{mkdir, path_exist};
use crate::lib::k_v_handler::{get_value, set_value, upsert_kvalue};
use crate::lib::machine::machine_neighbor::{NeighborInfo};
use crate::lib::machine::machine_profile::{EmailSettings, MachineProfile};
use crate::lib::services::initialize_node::maybe_init_dag;
use crate::lib::transactions::basic_transactions::signature_structure_handler::unlock_document::UnlockDocument;
use crate::lib::wallet::wallet_address_handler::{insert_address, WalletAddress};

//  '  '  '  '  '  '  '  '  '  '  '  '  '  '  '  machine_handler.cpp file
// #[derive(Default)]
pub struct CMachine{
    m_clone_id: i8,
    m_should_loop_threads: bool,

    m_is_in_sync_process: bool,
    m_last_sync_status_check: CDateT,

    m_threads_status: QSDicT,
    m_map_thread_code_to_prefix: QSDicT,
    m_is_develop_mod: bool,

    // pub(crate) m_db_connection: Client,
    m_develop_launch_date: CDateT,

    /*


      bool m_machine_is_loaded = false;
*/
    s_dag_is_initialized: bool,
    m_selected_profile: String,
    m_db_is_connected: bool,
    pub(crate) m_databases_are_created: bool,
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
        let (_status, profile) = MachineProfile::get_profile(constants::DEFAULT);
        CMachine {
            m_clone_id: 0,
            m_should_loop_threads: true,

            m_is_in_sync_process: true,
            m_last_sync_status_check: LAUNCH_DATE.to_string(),

            m_is_develop_mod: false,

            m_threads_status: HashMap::new(),
            m_map_thread_code_to_prefix: HashMap::new(),
            // m_db_connection: get_connection_1(),

            m_develop_launch_date: "".to_string(),

            s_dag_is_initialized: false,
            m_selected_profile: "".to_string(),
            m_db_is_connected: false,
            m_databases_are_created: false,

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
            m_profile: profile,
        }
    }

    pub fn init(&mut self) -> bool
    {
        self.create_folders();

        maybe_init_dag(self);

        //launchThreads();

        // doRegKeys();
        true
    }

    // func name was parseArgs
    pub fn parse_args(&mut self, args: VString, manual_clone_id: i8)
    {
        // println!("Env args: {:?}", args);

        let mut clone_id: i8 = 0;    // FIXME: this value must be defined by command line
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
    pub fn is_in_sync_process(&self, _force_to_control_based_on_dag_status: bool) -> bool
    {
        return false;//FIXME: remove this line after finishing develop
        //  put LAST_SYNC_STATUS in CMachine as a static member
        if !self.m_is_in_sync_process {
            return false;
        }

        if cutils::time_diff(self.m_last_sync_status_check.clone(), "".to_string()).as_minutes < 2 {
            return self.m_is_in_sync_process;
        }

        true

        /*
        String lastSyncStatus = KVHandler::getValue("LAST_SYNC_STATUS");
        if (lastSyncStatus == "")
        {
            IinitLastSyncStatus();
            String
            lastSyncStatus = KVHandler::getValue("LAST_SYNC_STATUS");
        }
        JSonObject
        lastSyncStatusObj = cutils::parseToJsonObj(lastSyncStatus);

        uint64_t
        cycleByMinutes = cutils::get_cycle_by_minutes();
        // control if the last status-check is still valid (is younger than 30 minutes?= 24 times in a cycle)
        if (!force_to_control_based_on_DAG_status & &
            (lastSyncStatusObj.value("checkDate").to_string() > cutils::minutes_before((cycleByMinutes / 24))))
        {
            bool
            is_in_sync = lastSyncStatusObj.value("lastDAGBlockCreationDate").to_string() < cutils::minutes_before(2 * cycleByMinutes);
            setIsInSyncProcess(is_in_sync, cutils::get_now());
            return is_in_sync;
        } else {
            // re-check graph info&  update status-check info too
            auto
            [status, blockRecord] = DAG::getLatestBlockRecord();
            if (status)
            cutils::exiter("No block in DAG exit!!", 111);

            bool
            is_in_sync_process = (blockRecord.m_creation_date < cutils::minutes_before(2 * cycleByMinutes));

            lastSyncStatusObj.insert("isInSyncMode", is_in_sync_process? "Y": "N");
            lastSyncStatusObj.insert("checkDate", cutils::get_now());
            lastSyncStatusObj.insert("lastDAGBlockCreationDate", blockRecord.m_creation_date);
            if (is_in_sync_process)
            lastSyncStatusObj.insert("lastTimeMachineWasInSyncMode", cutils::get_now());
            KVHandler::upsertKValue("LAST_SYNC_STATUS", cutils::serializeJson(lastSyncStatusObj));
            setIsInSyncProcess(is_in_sync_process, cutils::get_now());
            return is_in_sync_process;
        }
        */
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


//
// trait Booting{
//     fn parse_args(self, args: Vec<String>, manual_clone_id: i8);
//     fn set_clone_dev(self, clone_id: i8, is_develop_mod: bool) -> bool;
// }


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

*/
    pub fn setDAGIsInitialized(&mut self, status: bool) {
        self.s_dag_is_initialized = status;
    }
    /*
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





      static bool handshakeNeighbor(const String& n_id, const String& connection_type);

      static std::tuple<bool, bool> parseHandshake(
        const String& sender_email,
        const JSonObject& message,
        const String& connection_type);

      static bool floodEmailToNeighbors(
        const String& email,
        String PGP_public_key = ""){ return get().IfloodEmailToNeighbors(email, PGP_public_key); };

      static bool setAlreadyPresentedNeighbors(const JSonArray& already_presented_neighbors){ return get().IsetAlreadyPresentedNeighbors(already_presented_neighbors); }

      static JSonArray getAlreadyPresentedNeighbors(){ return get().IgetAlreadyPresentedNeighbors(); }

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
    //old_name_was setDatabasesAreCreated
    pub fn set_databases_are_created(&mut self, status: bool)
    {
        self.m_databases_are_created = status;
    }

    //old_name_was databasesAreCreated
    pub fn databases_are_created(&self) -> bool
    {
        return self.m_databases_are_created;
    }


    pub fn isDevelopMod(&self) -> bool
    {
        return self.m_is_develop_mod.clone();
    }

    pub fn getPubEmailInfo(&self) -> &EmailSettings {
        return &self.m_profile.m_mp_settings.m_public_email;
    }
    pub fn getPrivEmailInfo(&self) -> &EmailSettings {
        return &self.m_profile.m_mp_settings.m_private_email;
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

      bool IfloodEmailToNeighbors(
        const String& email,
        String PGP_public_key = "");

      JSonArray IgetAlreadyPresentedNeighbors(){ return m_profile.m_mp_settings.m_already_presented_neighbors; }
      bool IsetAlreadyPresentedNeighbors(const JSonArray& already_presented_neighbors){ m_profile.m_mp_settings.m_already_presented_neighbors = already_presented_neighbors; return true; }
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
      m_address = obj.value("m_address").to_string();
      m_password = obj.value("m_password").to_string();
      m_income_imap = obj.value("m_income_imap").to_string();
      m_income_pop3 = obj.value("m_income_pop3").to_string();
      m_incoming_mail_server = obj.value("m_incoming_mail_server").to_string();
      m_outgoing_mail_server = obj.value("m_outgoing_mail_server").to_string();
      m_outgoing_smtp = obj.value("m_outgoing_smtp").to_string();
      m_fetching_interval_by_minute = obj.value("m_fetching_interval_by_minute").to_string();
      m_pgp_private_key = obj.value("m_pgp_private_key").to_string();
      m_pgp_public_key = obj.value("m_pgp_public_key").to_string();
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
        if constants::LAUNCH_DATE != "" {
            return constants::LAUNCH_DATE.to_string();
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
        let (_status, profile) = MachineProfile::get_profile(&constants::DEFAULT);
        self.m_profile = profile;
        if self.m_profile.m_mp_code == constants::DEFAULT
        { return (true, "The Default profile Already initialized".to_string()); }


        // initializing default valuies and save it
        self.m_profile.m_mp_code = constants::DEFAULT.to_string();
        self.m_profile.m_mp_name = constants::DEFAULT.to_string();
        self.m_profile.m_mp_last_modified = cutils::get_now();

        {
            // initializing email PGP pair keys (it uses native node.js crypto lib. TODO: probably depricated and must refactor to use new one)
            let (status, private_pgp, public_pgp) = ccrypto::rsa_generate_key_pair();
            if !status {
                return (false, "Couldn't creat RSA key pairs (for private channel)".to_string());
            }
            self.m_profile.m_mp_settings.m_private_email.m_pgp_private_key = private_pgp;
            self.m_profile.m_mp_settings.m_private_email.m_pgp_public_key = public_pgp;
        }

        {
            // initializing email PGP pair keys (it uses native node.js crypto lib. TODO: probably depricated and must refactor to use new one)
            let (status, private_pgp, public_pgp) = ccrypto::rsa_generate_key_pair();
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

        {
            // this block existed ONLY for test and development environment
            let aaa_pgp_private_key: String = "-----BEGIN PRIVATE KEY-----\n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\nmNI4cf8hgrmIb2AgIyZ9LJMhSDaF+g==\n-----END PRIVATE KEY-----".to_string();
            let aaa_pgp_public_key: String = "-----BEGIN PUBLIC KEY-----\nMIICIDANBgkqhkiG9w0BAQEFAAOCAg0AMIICCAKCAgEAjSeu+Qny40lpzFfDy1XN\nWYyNHAldVUwH2I++PwV+xN9LE0JIbg3kh4BMHB5zN468qmlvE8QvTYbA2NNkHDYH\ngbi+cszEZveERJK8i34ZqqrgFh8+nBiRExVFjmtJMqxjUpGUIURlw4U0/EC3fM5O\nuQjX4z992F0L0vs0gWgFd3o3mapoh+NPTeyJ+b6IZ8JUm0Mw/i0hLGyBoK5wCY1Z\npqBuMmnK+st8/QG2UxS1TifiEy/zSFDdmvJ14B2CdckAY3PEtzCPYnm7Q1NpM9vs\nDdjrkOtiaREJaqvfuEw9g9IlECb5wkmXUNpWMfgd31ecD766SK/YP8dqR5zVW+Vh\ndp8FtRZGiGyWbOglyARKh4GC37LoJZJk8uTvp1FmEnUZK6nSSDmDzNAq8M8849zH\nMGXpoRd2A04nmuL5JAA2WpkW7WQqbvpmunXHRrOKt9cFmE6zuslm8TiRTigjD69Y\nOuSl48x7o/3LLJw4dbgPlmYi5Ezf2ZncllFSbIPxXSP5t95cqOrxTA7/Ue8/VJnu\nD3GJym8lXPFn8zHMjVH9ujxpm69+0XqSsBnkiQtcv0qeaPEBMobbABBcvMumvUAb\ndU1oMVr4Md+5Oapn7M03w+ZNIqofZtQFbmEiWajAYTixhYQHSqLXdp1YXgVIcfBa\nIyo+tB2Zu7WF6b4mp0tD0tUCARE=\n-----END PUBLIC KEY-----".to_string();

            let bbb_pgp_private_key: String = "-----BEGIN PRIVATE KEY-----\n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\nbxC8SYNfb397H1VNQUzlPIW6oA==\n-----END PRIVATE KEY-----".to_string();
            let bbb_pgp_public_key: String = "-----BEGIN PUBLIC KEY-----\nMIICIDANBgkqhkiG9w0BAQEFAAOCAg0AMIICCAKCAgEAzruXdhodXPeHsaMgNSw6\nUN4sRVGXeQRri8PJ4EFGKuTcVF6/2ZWy+I/VHHoq1Y/15vm/RFR/Qfit9TIgi+p2\neCw9djM7jlrKvTZfs+vAKx+qJ56x1gu0Cja+RkdbJdliqVKALZfY0+jWaco+zH+e\nuE8qPjUygUoISzmsNdcZrNNi2eLJ0yNMVa4x5y0Y0ywuhJvbxwMfvdRxMuy0YZwc\nC8i5lkCPCPKjr9OzR+jC7weUM/O/HmfqJOFhXiCKXvjXaBZdEt5WqU86lMRElk9p\nXhyglOlrPe/39n1Bhs0wu4MDbhcrXPj8pfEmxplhQfaR3/ezxwVxY0MEffFGlMgX\n60RwoEgR7BX1BZv2qwp7KBf7gYYJeOVYCjxdbKYRBLdOMfqQ3rXbtpljioa1975X\nZu3bAH0ByhKKPZKqVApL3sA1Pm7vWjS8XEUOcLgLKxYeqjrZeTes0VW30y/tHnkU\nOHT0zV8SYpwuS3mgQmbivrvCLhdAYE1e8HGztb2bhBPHsQgBnqRnfNDHyrErMQHB\nfwXT79uN/oyj4CReoMqfgm0n/GH2ZijKb94LR2MMHeTrTxOh3rI94t0HCqVxB7Ks\nh2H5H9ynZ6YUtM0FQQIjzUgU/bTQuAQHqOBTiiSCKFGrPf0iUcY0tt/pX8HQMP82\nzLZXKLKXNWgRfTAXiZTOzVUCARE=\n-----END PUBLIC KEY-----".to_string();

            if self.m_clone_id == 1
            {
                self.m_profile.m_mp_settings.m_machine_alias = "node-AAA".to_string();
                self.m_profile.m_mp_settings.m_public_email.m_address = "AAA@AAA.AAA".to_string();
                self.m_profile.m_mp_settings.m_public_email.m_pgp_private_key = aaa_pgp_private_key;
                self.m_profile.m_mp_settings.m_public_email.m_pgp_public_key = aaa_pgp_public_key.clone();
                self.add_a_new_neighbor(
                    "BBB@BBB.BBB".to_string(),
                    constants::PUBLIC.to_string(),
                    bbb_pgp_public_key.clone(),
                    constants::DEFAULT.to_string(),
                    constants::YES.to_string(),
                    NeighborInfo::new(),
                    cutils::get_now());
            }
            if self.m_clone_id == 2
            {
                self.m_profile.m_mp_settings.m_machine_alias = "node-BBB".to_string();
                self.m_profile.m_mp_settings.m_public_email.m_address = "BBB@BBB.BBB".to_string();
                self.m_profile.m_mp_settings.m_public_email.m_pgp_private_key = bbb_pgp_private_key;
                self.m_profile.m_mp_settings.m_public_email.m_pgp_public_key = bbb_pgp_public_key.to_string();
                self.add_a_new_neighbor(
                    "AAA@AAA.AAA".to_string(),
                    constants::PUBLIC.to_string(),
                    aaa_pgp_public_key,
                    constants::DEFAULT.to_string(),
                    constants::YES.to_string(),
                    NeighborInfo::new(),
                    cutils::get_now());
            }
        }


        self.save_settings();


        // set selected profile=default
        let values = HashMap::from([
            ("kv_value", &self.m_profile.m_mp_code as &(dyn ToSql + Sync)),
            ("kv_last_modified", &self.m_profile.m_mp_last_modified as &(dyn ToSql + Sync)),
        ]);
        q_upsert(
            STBL_KVALUE,
            "kv_key",
            "SELECTED_PROFILE",
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

        /*

                         ImaybeAddSeedNeighbors();

               */
        return (true, "The Default profile initialized".to_string());
    }

    //old_name_was bootMachine
    pub fn boot_machine(&mut self) -> bool
    {
        //  if (![1, 4].includes(appCloneId))
        //    machine.neighborHandler.addSeedNeighbors()
        //  //TODO: start remove it BEFORERELEASE
        //  if (appCloneId == 1) {
        //    let issetted = model.sRead({
        //        table: 'i_kvalue',
        //        fields: ['kv_value'],
        //        query: [
        //            ['kv_key', 'setSampleMachines']
        //        ]
        //    });
        //    if (issetted.length > 0)
        //        return { err: false, msg: 'sample machines already setted' };
        //    sampleMachines.setSampleMachines();
        //    db.setAppCloneId(1);
        //  }
        //  //TODO: end remove it BEFORERELEASE
        //  // initialize document & content
        //  let initRes = initContentSetting.doInit();
        //  if (initRes.err != false)
        //    return initRes;
        //  let doesSafelyInitialized = initContentSetting.doesSafelyInitialized();
        //  if (doesSafelyInitialized.err != false) {
        //    utils.exiter(`doesSafelyInitialized ${doesSafelyInitialized}`, 609);
        //  }
        //  return { err: false, msg: 'The machine fully initilized' };

        let clone_id = self.get_app_clone_id();
        dlog(
            &format!("Booting App({})", clone_id),
            constants::Modules::App,
            constants::SecLevel::Info);

        self.m_last_sync_status_check = self.get_launch_date();
        // control DataBase
        if !self.m_db_is_connected
        {
            let (status, _msg) = init_db(self);
            self.m_db_is_connected = status;

            if !status {
                dlog(
                    &format!("Coudn't establish database connections!"),
                    constants::Modules::App,
                    constants::SecLevel::Fatal);
                panic!("Coudn't establish database connections!");
            }
        }

        // check if flag MACHINE_AND_DAG_ARE_SAFELY_INITIALIZED is setted
        let is_inited = get_value("MACHINE_AND_DAG_ARE_SAFELY_INITIALIZED");
        if is_inited != constants::YES
        {
            empty_db(self);  // machine didn't initialized successfully, so empty all tables and try from zero
            set_value("MACHINE_AND_DAG_ARE_SAFELY_INITIALIZED", constants::NO, true);
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

    //old_name_was getReportsPath
    pub fn get_inbox_path(&self) -> String
    {
        return self.get_clone_path() + "/inbox";
    }

    //old_name_was getReportsPath
    pub fn get_outbox_path(&self) -> String
    {
        return self.get_clone_path() + "/outbox";
    }

    //old_name_was getReportsPath
    pub fn get_logs_path(&self) -> String
    {
        return self.get_clone_path() + &"/logs";
    }

    //old_name_was getCachePath
    pub fn get_cache_path(&self) -> String
    {
        return self.get_clone_path() + "/cache_files";
    }

    //old_name_was getDAGBackup
    pub fn get_dag_backup(&self) -> String { return self.get_clone_path() + "/dag_backup"; }

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
    pub fn getBackerAddress(&self) -> CAddressT
    {
        self.m_profile.m_mp_settings.m_backer_detail.m_account_address.clone()
    }

    //old_name_was getBackerDetails
    pub fn getBackerDetails(&self) -> &UnlockDocument
    {
        return &self.m_profile.m_mp_settings.m_backer_detail;
    }

    //old_name_was loadSelectedProfile
    pub fn loadSelectedProfile(&mut self) -> bool
    {
        let selected_prof = get_value("SELECTED_PROFILE");
        if selected_prof == "" {
            return false;
        }

        let mp: MachineProfile = self.read_profile(selected_prof);
        self.m_profile = mp;
        return true;


        // importJson(&self, profile: MachineProfile)
        // {
        //     m_machine_alias = obj.value("m_machine_alias").to_string();
        //     m_language = obj.value("m_language").to_string();
        //     m_term_of_services = obj.value("m_term_of_services").to_string();
        //     m_machine_alias = obj.value("m_machine_alias").to_string();
        //     m_already_presented_neighbors = obj.value("m_already_presented_neighbors").toArray();
        //     m_backer_detail = new UnlockDocument();
        //     m_backer_detail->importJson(obj.value("m_backer_detail").toObject());
        //     m_public_email.importJson(obj.value("m_public_email").toObject());
        //     m_private_email.importJson(obj.value("m_private_email").toObject());
        // }
    }

    pub fn read_profile(&self, mp_code: String) -> MachineProfile
    {
        let (_status, records) = q_select(
            STBL_MACHINE_PROFILES,
            &vec!["mp_code", "mp_name", "mp_settings"],
            &vec![
                simple_eq_clause("mp_code", &*mp_code)],
            vec![],
            0,
            true,
        );
        if records.len() != 1
        {
            return MachineProfile::get_null();
        }

        let serialized_profile = records[0].get("mp_settings").unwrap().clone();
        let profile: MachineProfile = serde_json::from_str(serialized_profile.as_str()).unwrap();
        return profile;
    }

    pub fn getSelectedMProfile(&mut self) -> String
    {
        if self.m_selected_profile != ""
        {
            return self.m_selected_profile.clone();
        }

        let mp_code: String = get_value("SELECTED_PROFILE");
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
    pub fn save_settings(&self) -> bool
    {
        let (status, serialized_settings) = match serde_json::to_string(&self.m_profile) {
            Ok(ser) => { (true, ser) }
            Err(e) => {
                dlog(
                    &format!("Failed in serialization{:?}", e),
                    constants::Modules::App,
                    constants::SecLevel::Error);
                (false, "".to_string())
            }
        };
        if !status
        { return false; }

        //   String serialized_setting = cutils::serializeJson(m_profile.exportJson());
        let values = HashMap::from([
            ("mp_code", &self.m_profile.m_mp_code as &(dyn ToSql + Sync)),
            ("mp_name", &self.m_profile.m_mp_name as &(dyn ToSql + Sync)),
            ("mp_settings", &serialized_settings as &(dyn ToSql + Sync)),
            ("mp_last_modified", &self.m_profile.m_mp_last_modified as &(dyn ToSql + Sync)),
        ]);

        return q_upsert(
            STBL_MACHINE_PROFILES,
            "mp_code",
            &self.m_profile.m_mp_code[..],
            &values,
            true);
    }

    /*

    bool CMachine::ImaybeAddSeedNeighbors()
    {
      add_a_new_neighbor("matbit@secmail.pro", constants::PUBLIC);
      return true;
    }

*/
    pub fn getLastSyncStatus(&self) -> JSonObject
    {
        let mut lastSyncStatus: String = get_value("LAST_SYNC_STATUS");
        if lastSyncStatus == ""
        {
            self.initLastSyncStatus();
            lastSyncStatus = get_value("LAST_SYNC_STATUS");
        }
        return cutils::parseToJsonObj(&lastSyncStatus);
    }

    pub fn initLastSyncStatus(&self) -> bool
    {
        let lastSyncStatus: JSonObject = json!({
              "isInSyncMode": "Unknown",
              "lastTimeMachineWasInSyncMode":
                          cutils::minutes_before(cutils::get_cycle_by_minutes() * 2, cutils::get_now()),
              "checkDate": cutils::minutes_before(cutils::get_cycle_by_minutes(), cutils::get_now()),
              "lastDAGBlockCreationDate": "Unknown"
            });
        return upsert_kvalue(
            "LAST_SYNC_STATUS",
            &cutils::serializeJson(&lastSyncStatus),
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
    pub fn cachedBlocks(
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

    pub fn cachedBlockHashes(
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
                   if ((a_coin.value("ut_visible_by").to_string() != visible_by) || (a_coin.value("ut_coin").to_string() != the_coin))
                     remined_coins.push(a_coin);

                 }
                 else if (visible_by != "")
                 {
                   if (a_coin.value("ut_visible_by").to_string() != visible_by)
                     remined_coins.push(a_coin);
                 }
                 else if (the_coin != "")
                 {
                   if (a_coin.value("ut_coin").to_string() != the_coin)
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
}