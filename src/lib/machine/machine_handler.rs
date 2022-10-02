use std::collections::HashMap;
use crate::{application, ccrypto, cutils};
use crate::lib::custom_types::{CAddressT, CBlockHashT, CCoinCodeT, CDateT, DocLenT, JSonObject, QVDRecordsT, VString};
use crate::lib::database::db_handler::{empty_db, maybe_initialize_db};
use postgres::types::ToSql;
use serde_json::json;
use crate::lib::address::address_handler::create_a_new_address;
use crate::lib::constants;
use crate::lib::dag::dag_walk_through::get_latest_block_record;
use crate::lib::database::abs_psql::{q_select, q_upsert, simple_eq_clause};
use crate::lib::database::tables::{C_KVALUE, C_MACHINE_PROFILES, C_TRX_COINS};
use crate::lib::dlog::dlog;
use crate::lib::file_handler::file_handler::{mkdir, path_exist};
use crate::lib::k_v_handler::{get_value, set_value, upsert_kvalue};
// use crate::lib::machine::dev_neighbors::dev_neighbors::{ALICE_PRIVATE_KEY, ALICE_PUBLIC_EMAIL, ALICE_PUPLIC_KEY, BOB_PRIVATE_KEY, BOB_PUBLIC_EMAIL, BOB_PUPLIC_KEY, EVE_PRIVATE_KEY, EVE_PUBLIC_EMAIL, EVE_PUPLIC_KEY, HU_PRIVATE_KEY, HU_PUBLIC_EMAIL, HU_PUPLIC_KEY, USER_PRIVATE_KEY, USER_PUBLIC_EMAIL, USER_PUPLIC_KEY};
use crate::lib::machine::machine_profile::{EmailSettings, get_profile_from_db, MachineProfile};
use crate::lib::services::initialize_node::maybe_init_dag;
use crate::lib::transactions::basic_transactions::signature_structure_handler::unlock_document::UnlockDocument;
use crate::lib::wallet::wallet_address_handler::{insert_address, WalletAddress};

#[allow(unused, dead_code)]
pub struct CoinsVisibilityRes {
    status: bool,
    records: VString,
    is_visible: bool,
}


// #[derive(Default)]
pub struct CMachine {
    pub m_clone_id: i8,
    pub m_should_loop_threads: bool,

    pub m_is_in_sync_process: bool,
    pub m_last_sync_status_check: CDateT,

    // m_threads_status: QSDicT,
    // m_map_thread_code_to_prefix: QSDicT,
    pub m_config_file: String,
    pub m_is_develop_mod: bool,

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
    #[allow(unused, dead_code)]
    // TODO: remove this variable(mechanism) after fixing sqlite database lock problem
    pub m_recorded_blocks_in_db: u32,

    // TODO: remove this variable(mechanism) after fixing sqlite database lock problem and bloom filter implementation
    pub m_cache_coins_visibility: VString,

    // TODO: remove this variable(mechanism) after fixing sqlite database lock problem
    pub m_cache_spendable_coins: QVDRecordsT,

    // TODO: optimize it ASAP
    pub m_dag_cached_blocks: QVDRecordsT,

    // TODO: optimize it ASAP
    pub m_dag_cached_block_hashes: Vec<String>,

    pub m_profile: MachineProfile,

    pub m_email_is_active: bool,
    pub m_use_hard_disk_as_a_buffer: bool,
    pub m_should_run_web_server: bool,
    pub m_web_server_address: String,
    pub m_config_source: String,
    pub m_hard_root_path: String,
    pub m_launch_date: String,

    pub m_db_host: String,
    pub m_db_name: String,
    pub m_db_user: String,
    pub m_db_pass: String,

}
/*
pub trait CMachineThreadGaps {
    fn get_coinbase_import_gap(&self) -> TimeBySecT;
    fn get_block_invoke_gap(&self) -> TimeBySecT;
    fn get_nb_coins_import_gap(&mut self) -> TimeBySecT;
}
 */


impl CMachine {
    pub fn new() -> Self {
        eprintln!("New CMachine was create.");

        CMachine {
            m_clone_id: 0,
            m_should_loop_threads: true,

            m_is_in_sync_process: true,

            m_is_develop_mod: false,

            m_selected_profile: "".to_string(),
            m_email_is_active: false,
            m_use_hard_disk_as_a_buffer: false,
            m_should_run_web_server: false,
            m_web_server_address: "".to_string(),

            /*
          const static String stb_machine_block_buffer;
          const static VString stb_machine_block_buffer_fields;

          static const String stbl_machine_onchain_contracts;
          static const VString stbl_machine_onchain_contracts_fields;

          Config* m_global_configs {};
          */
            m_recorded_blocks_in_db: 0, // TODO: remove this variable(mechanism) after fixing sqlite database lock problem
            /*
              VString m_cache_coins_visibility = {}; // TODO: remove this variable(mechanism) after fixing sqlite database lock problem and bloom filter implementation
            QVDRecordsT m_dag_cached_blocks; // TODO: optimize it ASAP
            VString m_dag_cached_block_hashes = {}; // TODO: optimize it ASAP

            MachineProfile m_profile;

              */
            m_cache_coins_visibility: vec![],
            // TODO: remove this variable(mechanism) after fixing sqlite database lock problem
            m_cache_spendable_coins: vec![],
            m_dag_cached_blocks: vec![],
            m_dag_cached_block_hashes: vec![],
            m_profile: MachineProfile::new(),
            m_config_file: "".to_string(),
            m_hard_root_path: "".to_string(),
            m_config_source: "".to_string(),
            m_last_sync_status_check: "2024-00-00 00:00:00".to_string(),
            m_launch_date: "2024-00-00 00:00:00".to_string(),

            m_db_host: "".to_string(),
            m_db_name: "".to_string(),
            m_db_user: "".to_string(),
            m_db_pass: "".to_string(),

        }
    }

    pub fn initialize_machine(&mut self) -> bool
    {
        self.create_folders();

        self.m_last_sync_status_check = self.get_launch_date();

        // control DataBase
        let (status, msg) = maybe_initialize_db();
        if !status
        {
            panic!("failed on maybe initialize db1 {}", msg)
        }

        if !application().is_db_initialized()
        {
            panic!("failed on maybe initialize db2 {}", msg)
        }


        let (status, _msg) = self.maybe_init_default_profile();
        if status != true {
            return false;
        }

        // load machine selected profile
        self.load_selected_profile();

        self.maybe_add_seed_neighbors();

        maybe_init_dag();

        //launchThreads();

        // doRegKeys();
        true
    }

    //old_name_was getAppCloneId
    pub fn get_app_clone_id(&self) -> i8
    {
        return self.m_clone_id;
    }

    pub fn get_app_machine_id(&self) -> String
    {
        return self.m_profile.m_mp_settings.m_machine_alias.clone();
    }

    //func name was shouldLoopThreads
    #[allow(dead_code, unused)]
    pub fn should_loop_threads(&self) -> bool {
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

        if application().time_diff(
            self.m_last_sync_status_check.clone(),
            "".to_string()).as_minutes < 2
        {
            return self.m_is_in_sync_process;
        }

        let mut last_sync_status = get_value("last_sync_status");
        if last_sync_status == ""
        {
            self.init_last_sync_status();
            last_sync_status = get_value("last_sync_status");
        }
        let (_status, mut last_sync_status_json_obj) = cutils::controlled_str_to_json(&last_sync_status);

        let cycle_by_minutes = application().get_cycle_by_minutes();
        // control if the last status-check is still valid (is younger than 30 minutes?= 24 times in a cycle)
        let now_ = application().now();
        if !force_to_control_based_on_dag_status &&
            (last_sync_status_json_obj["checkDate"].to_string() >
                application().minutes_before(
                    cycle_by_minutes / 24,
                    &now_))
        {
            let is_in_sync: bool = last_sync_status_json_obj["lastDAGBlockCreationDate"].to_string()
                <
                application().minutes_before(
                    2 * cycle_by_minutes,
                    &now_);
            self.set_is_in_sync_process(is_in_sync, &application().now());
            return is_in_sync;
        } else {
            // re-check graph info&  update status-check info too
            let (status, block) = get_latest_block_record();
            if !status {
                panic!("No block in DAG exit!!");
            }
            let now_ = application().now();
            let is_in_sync_process: bool = block.m_block_creation_date < application().minutes_before(2 * cycle_by_minutes, &now_);

            if is_in_sync_process {
                last_sync_status_json_obj["isInSyncMode"] = "Y".into();
            } else {
                last_sync_status_json_obj["isInSyncMode"] = "N".into();
            }
            last_sync_status_json_obj["checkDate"] = application().now().into();
            last_sync_status_json_obj["lastDAGBlockCreationDate"] = block.m_block_creation_date.into();
            if is_in_sync_process {
                last_sync_status_json_obj["lastTimeMachineWasInSyncMode"] = application().now().into();
            }
            upsert_kvalue("last_sync_status", &cutils::controlled_json_stringify(&last_sync_status_json_obj), false);
            self.set_is_in_sync_process(is_in_sync_process, &application().now());
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
    #[allow(dead_code, unused)]
    pub fn report_thread_status(&mut self, thread_prefix: &String, thread_code: &String, thread_status: &String)
    {
        // self.m_threads_status.insert((thread_prefix.to_string() + &thread_code).clone(), thread_status.clone());
        // self.m_map_thread_code_to_prefix.insert(thread_code.clone(), thread_prefix.clone());
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


*/

    //old_name_was getMachineServiceInterests
    pub fn get_machine_service_interests(
        &self,
        d_type: &String,
        _d_class: &String,
        _d_len: DocLenT,
        _extra_length: DocLenT,
        _supported_p4p_trx_count: u8) -> f64
    {
        // TODO: these values must be costumizable for backers
        let services_price_coefficient: HashMap<String, f64> = HashMap::from([
            (constants::document_types::BASIC_TX.to_string(), 1.000),    // the output must be 1 or greater. otherwise other nodes reject the block
            (constants::document_types::FREE_POST.to_string(), 1.0001)]);

        if services_price_coefficient.contains_key(d_type)
        {
            return services_price_coefficient[d_type];
        }

        // node doesn't support this type of documents so accept it as a base feePerByte
        return 1.000;
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

        if self.m_profile.m_mp_code == "" || self.m_profile.m_mp_name == ""
        {
            println!("selffffffff m_mp_code {:?}", &self.m_profile.m_mp_code);
            println!("selffffffff m_mp_name {:?}", &self.m_profile.m_mp_name);
            panic!("Why mp_code and mp_name are empty!");
        }

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
        self.m_launch_date.clone()
    }

    //old_name_was initDefaultProfile
    pub fn maybe_init_default_profile(&mut self) -> (bool, String)
    {
        let (_status, profile) = get_profile_from_db(&constants::DEFAULT);
        self.m_profile = profile;
        // panic!("elf.m_profile.m_mp_code {}", self.m_profile.m_mp_code);
        if self.m_profile.m_mp_code == constants::DEFAULT.to_string()
        {
            return (true, "The Default profile Already initialized".to_string());
        }

        // initializing default valuies and save it
        self.m_profile.m_mp_code = constants::DEFAULT.to_string();
        self.m_profile.m_mp_name = constants::DEFAULT.to_string();
        self.m_profile.m_mp_last_modified = application().now();

        println!("Generating RSA key pairs.");
        {
            // initializing email PGP pair keys (it uses native node.js crypto lib. TODO: probably depricated and must refactor to use new one)
            let (status, private_pgp, public_pgp) = ccrypto::rsa_generate_key_pair(0);
            if !status {
                return (false, "Couldn't create RSA key pairs (for private channel)".to_string());
            }
            self.m_profile.m_mp_settings.m_private_email.m_pgp_private_key = private_pgp;
            self.m_profile.m_mp_settings.m_private_email.m_pgp_public_key = public_pgp;
        }
        println!("RSA 1 Done.");

        {
            // initializing email PGP pair keys (it uses native node.js crypto lib. TODO: probably depricated and must refactor to use new one)
            let (status, private_pgp, public_pgp) = ccrypto::rsa_generate_key_pair(0);
            if !status {
                return (false, "Couldn't creat RSA key pairs (for public channel)".to_string());
            }
            self.m_profile.m_mp_settings.m_public_email.m_pgp_private_key = private_pgp;
            self.m_profile.m_mp_settings.m_public_email.m_pgp_public_key = public_pgp;
        }
        println!("RSA 2 Done.");

        println!("Generating Node Master Private Key.");
        let (status, unlock_doc) = create_a_new_address(
            constants::signature_types::STRICT,
            "2/3",
            constants::CURRENT_SIGNATURE_VERSION);
        if !status
        {
            return (false, "Couldn't creat ECDSA key pairs (for public channel)".to_string());
        }
        println!("Master Private Key Done.");

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
            application().now(),
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
            empty_db();  // machine didn't initialized successfully, so empty all tables and try from zero
            set_value("machine_and_dag_are_safely_initialized", constants::NO, true);
        }

        /*

              m_machine_is_loaded = true;
              s_DAG_is_initialized = true;

   */

        {
            // remove this block after improving db and fixing database block problem
            let (status, coins) = self.cached_spendable_coins(
                "read",
                &vec![],
                &"".to_string(),
                &"".to_string());
            if !status
            {
                dlog(
                    &format!("Couldn't read from cached Spendable Coins!"),
                    constants::Modules::App,
                    constants::SecLevel::Fatal);
            }

            if coins.len() < 500
            {
                let (status, records) = q_select(
                    C_TRX_COINS,
                    vec!["ut_coin", "ut_creation_date", "ut_ref_creation_date", "ut_visible_by", "ut_o_address", "ut_o_value"],
                    vec![],
                    vec![],
                    0,
                    false,
                );
                if !status
                {
                    dlog(
                        &format!("Couldn't read from db Spendable Coins!"),
                        constants::Modules::App,
                        constants::SecLevel::Fatal);
                } else {
                    dlog(
                        &format!("Assigned {} spendable coins to cache!", records.len()),
                        constants::Modules::App,
                        constants::SecLevel::Info);
                }
                self.cached_spendable_coins(
                    "assign",
                    &records,
                    &"".to_string(),
                    &"".to_string());
            }
        }

        return true;
    }

    //old_name_was createFolders
    pub fn create_folders(&self) -> bool
    {
        if application().root_path() != ""
        {
            if !path_exist(&application().root_path())
            { mkdir(&application().root_path()); }
        }

        if !path_exist(&application().clone_path())
        { mkdir(&application().clone_path()); }

        if !path_exist(&application().reports_path())
        { mkdir(&application().reports_path()); }

        if !path_exist(&application().inbox_path())
        { mkdir(&application().inbox_path()); }

        if !path_exist(&application().outbox_path())
        { mkdir(&application().outbox_path()); }

        if !path_exist(&application().dag_backup())
        { mkdir(&application().dag_backup()); }

        if !path_exist(&application().cache_path())
        { mkdir(&application().cache_path()); }

        return true;
    }

    //old_name_was getBackerAddress
    pub fn get_backer_address(&self) -> CAddressT
    {
        self.m_profile.m_mp_settings.m_backer_detail.m_account_address.clone()
    }

    //old_name_was getBackerDetails
    #[allow(dead_code, unused)]
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
    #[allow(dead_code, unused)]
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

    #[allow(dead_code, unused)]
    pub fn read_profile(&self, mp_code: String) -> MachineProfile
    {
        let (_status, records) = q_select(
            C_MACHINE_PROFILES,
            vec!["mp_code", "mp_name", "mp_settings"],
            vec![
                simple_eq_clause("mp_code", &mp_code.to_string())],
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
    #[allow(dead_code, unused)]
    pub fn get_last_sync_status(&self) -> JSonObject
    {
        let mut last_sync_status: String = get_value("last_sync_status");
        if last_sync_status == ""
        {
            self.init_last_sync_status();
            last_sync_status = get_value("last_sync_status");
        }
        let (_status, j_obj) = cutils::controlled_str_to_json(&last_sync_status);
        return j_obj;
    }

    //old_name_was initLastSyncStatus
    pub fn init_last_sync_status(&self) -> bool
    {
        let back_in_time = application().get_cycle_by_minutes() * 2;
        let now_ = application().now();
        let last_time_machine_was_in_sync_mode = application().minutes_before(
            back_in_time,
            &now_);

        let back_in_time = application().get_cycle_by_minutes();
        let check_date = application().minutes_before(
            back_in_time,
            &now_);

        let last_sync_status: JSonObject = json!({
              "isInSyncMode": "Unknown",
              "lastTimeMachineWasInSyncMode": last_time_machine_was_in_sync_mode,
              "checkDate": check_date,
              "lastDAGBlockCreationDate": "Unknown"
            });
        return upsert_kvalue(
            "last_sync_status",
            &cutils::controlled_json_stringify(&last_sync_status),
            true);
    }
    /*


        /**
         * @brief CMachine::signByMachineKey
         * @param sign_message
         * @param unlock_index
         * @return {status, signer address, unlock set, signatures}
         */
        std::tuple<bool, String, UnlockSet, VString> CMachine::signByMachineKey(
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

      m_cache_coins_visibility = VString {};

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
    #[allow(unused, dead_code)]
    pub fn cached_blocks(
        &mut self,
        action: &str,
        blocks: QVDRecordsT,
        _status: &String) -> bool
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
                            // self.m_dag_cached_blocks[i]["b_coins_imported"] = status.to_string();
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
    #[allow(unused, dead_code)]
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

    // old name was cachedSpendableCoins
    pub fn cached_spendable_coins(
        &mut self,
        action: &str,
        coins: &QVDRecordsT,
        visible_by: &CBlockHashT,
        the_coin: &CCoinCodeT) -> (bool, QVDRecordsT)
    {
        if action == "assign"
        {
            self.m_cache_spendable_coins = coins.clone();
        }

        if action == "append"
        {
            for coin in coins
            {
                self.m_cache_spendable_coins.push(coin.clone());
            }
        }

        if action == "remove"
        {
            let mut remained_coins = vec![];
            if (visible_by != "") || (the_coin != "")
            {
                for a_coin in &self.m_cache_spendable_coins
                {
                    if (visible_by != "") && (the_coin != "")
                    {
                        if (&a_coin["ut_visible_by"].to_string() != visible_by) || (&a_coin["ut_coin"].to_string() != the_coin)
                        {
                            remained_coins.push(a_coin.clone());
                        }
                    } else if visible_by != ""
                    {
                        if &a_coin["ut_visible_by"].to_string() != visible_by
                        {
                            remained_coins.push(a_coin.clone());
                        }
                    } else if the_coin != ""
                    {
                        if &a_coin["ut_coin"].to_string() != the_coin
                        {
                            remained_coins.push(a_coin.clone());
                        }
                    }
                }

                self.m_cache_spendable_coins = remained_coins;
            }
        }

        return (true, self.m_cache_spendable_coins.clone());

        // } catch (std::logic_error&)
        // {
        //   String thread_code = String::number((quint64)QThread::currentThread(), 16);
        //   CLog::log("Failed in cached spendable coins action(" + action + ") Thread(" + thread_code + " / " + mapThreadCodeToPrefix(thread_code)+ ")");
        //   std::cout << "[exception caught]\n";
        //   return {false, m_cache_spendable_coins};
        // }
    }

    // old name was cachedCoinsVisibility
    pub fn cached_coins_visibility(
        &mut self,
        action: &str,
        entries: &VString) -> CoinsVisibilityRes
    {
        let mut contains: bool = true;

        if action == "assign"
        {
            self.m_cache_coins_visibility = entries.clone();
        }

        if action == "append"
        {
            for a_visiblity in entries
            {
                self.m_cache_coins_visibility.push(a_visiblity.clone());
            }
        }

        if action == "contains"
        {
            contains = self.m_cache_coins_visibility.contains(&entries[0]);
        }

        return CoinsVisibilityRes {
            status: true,
            records: self.m_cache_coins_visibility.clone(),
            is_visible: contains,
        };

        // }
        // catch (std::logic_error&)
        // {
        //   String thread_code = String::number((quint64)QThread::currentThread(), 16);
        //   CLog::log("Failed in cached spendable coins action(" + action + ") Thread(" + thread_code + " / " + mapThreadCodeToPrefix(thread_code)+ ")");
        //   std::cout << "[exception caught]\n";
        //   return CoinsVisibilityRes {false, m_cache_coins_visibility, false};
        // }
    }

    /*

       double CMachine::getMinPollingTimeframeByHour()
       {
         return (cutils::get_cycle_by_minutes() * 2.0) / 60.0;
       }

        */
}