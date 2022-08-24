use std::collections::HashMap;
use substring::Substring;
use crate::{application, CMachine, constants};
use crate::lib::custom_types::VString;
use crate::lib::k_v_handler::set_value;

impl CMachine {
    // func name was parseArgs
    pub fn parse_args(&mut self, args: VString, forcing_manual_clone_id: i8)
    {
        // cargo run cid=1 dev verbose config=/Users/silver/Documents/Diamond_files/config.txt
        // println!("Env args: {:?}", args);

        let mut clone_id: i8 = 0;
        let mut config_file: String;
        let mut is_develop_mod: bool = false;
        let mut _verbose: bool = false;

        let mut args_dic: HashMap<String, String> = HashMap::new();
        for a_param in args {
            if a_param.contains("=")
            {
                let arg_details = a_param.split("=").collect::<Vec<&str>>();
                args_dic.insert(arg_details[0].to_string(), arg_details[1].to_string());
            } else {
                if a_param == "dev"
                {
                    args_dic.insert(a_param, constants::YES.to_string());
                    is_develop_mod = true;
                } else if a_param == "verbose"
                {
                    args_dic.insert(a_param, constants::YES.to_string());
                    _verbose = true;
                }
            }
        }

        // cid: clone id
        if args_dic.contains_key("cid") {
            clone_id = args_dic["cid"].parse::<i8>().unwrap();
        }

        // config: config file
        config_file = "/Users/silver/Documents/Diamond_files/config.txt".to_string();
        if std::env::consts::OS == "windows" {
            config_file = "c:\\Documents\\config.ini".to_string();
        }

        let mut config_source = "Default";
        if args_dic.contains_key("config") {
            config_file = args_dic["config"].clone();
            config_source = "Command-line";
        }

        if forcing_manual_clone_id > 0 {
            clone_id = forcing_manual_clone_id;
        }

        self.m_clone_id = clone_id;
        self.m_is_develop_mod = is_develop_mod;
        self.m_config_file = config_file.clone();

        if args_dic.contains_key("config") {
            // update database
            set_value("config_file", &config_file.to_string(), false);
        } else {
            // it should be loaded from somewhere (probably db)
        }

        self.m_config_source = config_source.to_string();
        self.parse_config_file();

        // set global values
        application().setup_app(self);

        // maybe_switch_db(self.m_clone_id);

    }

    // func name was setCloneDev
    pub fn parse_config_file(&mut self) -> bool
    {
        use configparser::ini::Ini;

        let mut config = Ini::new();
        let (status, _configs_map) = match config.load(&self.m_config_file) {
            Ok(r) => (true, r),
            Err(e) => {
                eprintln!("{}", e);
                (false, HashMap::new())
            }
        };
        if !status
        {
            eprintln!("Please give the config file path through commandline (config=\"C:\\Documents\\config.txt\") or copy the config.txt file from source folder to default path \"C:\\Home\\config.txt\"");
            panic!("Invalid config file path");
        }

        println!("Config file was loaded({}). {}", self.m_config_source, self.m_config_file);
        // remove "/config.txt" from the end of path
        self.m_hard_root_path = self.m_config_file.substring(0, self.m_config_file.len() - 11).to_string();

        self.m_launch_date = config.get("default", "launch_date").unwrap();
        self.m_cycle_length = config.getuint("default", "cycle_length").unwrap().unwrap() as u32;
        self.m_last_sync_status_check = self.m_launch_date.clone();
        self.m_email_is_active = config.getbool("default", "email_is_active").unwrap().unwrap();
        self.m_use_hard_disk_as_a_buffer = Ini::getbool(&config, "default", "use_hard_disk_as_a_buffer").unwrap().unwrap();

        self.m_db_host = config.get("database", "db_host").unwrap();
        self.m_db_name = config.get("database", "db_name").unwrap();
        self.m_db_user = config.get("database", "db_user").unwrap();
        self.m_db_pass = config.get("database", "db_pass").unwrap();

        true
    }
}