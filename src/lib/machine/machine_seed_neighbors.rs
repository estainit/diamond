use crate::{application, CMachine, constants, dlog, get_value};
use crate::lib::k_v_handler::upsert_kvalue;
use crate::lib::machine::dev_neighbors::dev_neighbors::{ALICE_PRIVATE_KEY, ALICE_PUBLIC_EMAIL, ALICE_PUPLIC_KEY, BOB_PRIVATE_KEY, BOB_PUBLIC_EMAIL, BOB_PUPLIC_KEY, EVE_PRIVATE_KEY, EVE_PUBLIC_EMAIL, EVE_PUPLIC_KEY, get_hu_profile, HU_PRIVATE_KEY, HU_PUBLIC_EMAIL, HU_PUPLIC_KEY, USER_PRIVATE_KEY, USER_PUBLIC_EMAIL, USER_PUPLIC_KEY};
use crate::lib::machine::machine_neighbor::{add_a_new_neighbor, NeighborInfo};
use crate::lib::machine::machine_profile::MachineProfile;
use crate::lib::wallet::wallet_address_handler::{insert_address, WalletAddress};

impl CMachine {
    pub fn maybe_add_seed_neighbors(&mut self) -> bool
    {
        println!("Adding some neighbors");

        let now_ = application().now();
        let (status, msg) = add_a_new_neighbor(
            "seed1@seed.pro".to_string(),
            constants::PUBLIC.to_string(),
            "-----BEGIN PUBLIC KEY-----\nMIICIjANBgkqhkiG9w0BAQEFAAOCAg8AMIICCgKCAgEA1RG+nLSuYWszuVQL9ZaJ\nMflUZXlGfPKk+tmFxUnEGLKG4/QuTN/1m/Bm6AkFnHZkXWhGisyHG8ujgSAQZQnK\nUWsI+VGJ41YnqvxAKYIL3qvBSPLo8ppvN21tr7pbCL3uR0isHjXSp6XGH3mVBTd6\ntaJhRBtuQKdeFd3QMZCyofnaagA1wPHtT8wCz4X+7LckrfSRGhdjPUoT3pZ2R3Z8\noyAOtBzr7IRHDObs11Z5sdFmZVRQV1iSgxZyS3jEYjMqZN5FaxVYLq64MHIEYIdw\nLpofmWqkDrKUws9jTmiirmDfaAqsu6siHdCbCpnV026QMtbQukguJv3UFbdN/lh2\n2Obz9OKw802xMSgt4nULDSAvt8mrJsbyWbX66yVNkmN3OKiy36Ig9faCoxJTjzjW\nS5Kr7JXcBCyavog1n0NcNCOApde3TsoHNAt/5GJ8pMON2jG+i58Ug4/1mtz1tYEs\ndKFj4tbAVXgNPKNl0MlmReSjFati3K8H14twvOLsN0wnycWqJThwFCRFRfSV2weY\nw1w+k4hmsL0FvbZtl0OdQePvqbTQQTQc8SROc3Ejq/04oyc5S9D1MdaDEfdXqcqk\nnFzc3u3rzw1BPGdkw0LoiwDjp0WOSSB5u5NRI9UYxDOWdTaEPGpChKycm4kgUjYK\nvucjKoPGeLsBGmH8+NRT+RsCAwEAAQ==\n-----END PUBLIC KEY-----\n".to_string(),
            constants::DEFAULT.to_string(),
            constants::YES.to_string(),
            NeighborInfo::new(),
            now_);
        dlog(
            &format!("result of add a new neighbor({}): {}", status, msg),
            constants::Modules::App,
            constants::SecLevel::Info);


        if self.is_develop_mod()
        {
            // this block existed ONLY for test and development environment

            if get_value("dev_settings_done") == constants::YES.to_string()
            {
                return true;
            }
            upsert_kvalue("dev_settings_done", constants::YES, false);

            let user_and_hu_are_neighbor = false;
            let clone_id = self.get_app_clone_id();
            println!("Machine is in Dev mode, so make some neighborhoods! clone({})", clone_id);
            if [0, 1, 2, 3].contains(&clone_id) {
                println!("Machine is a fake neighbor, so make some connections!");

                if clone_id == 0 {
                    // update machine settings to dev mode settings (user@imagine.com)
                    println!("Setting machine as a developing node (User)");
                    self.m_profile.m_mp_settings.m_machine_alias = "node-user".to_string();
                    self.m_profile.m_mp_settings.m_public_email.m_pgp_private_key = USER_PRIVATE_KEY.to_string();
                    self.m_profile.m_mp_settings.m_public_email.m_pgp_public_key = USER_PUPLIC_KEY.to_string();
                    self.m_profile.m_mp_settings.m_public_email.m_address = USER_PUBLIC_EMAIL.to_string();

                    let now_ = application().now();
                    if user_and_hu_are_neighbor
                    {
                        // add Hu as a neighbor
                        add_a_new_neighbor(
                            HU_PUBLIC_EMAIL.to_string(),
                            constants::PUBLIC.to_string(),
                            HU_PUPLIC_KEY.to_string(),
                            constants::DEFAULT.to_string(),
                            constants::YES.to_string(),
                            NeighborInfo::new(),
                            now_);
                    }

                    // add Eve as a neighbor
                    let now_ = application().now();
                    add_a_new_neighbor(
                        EVE_PUBLIC_EMAIL.to_string(),
                        constants::PUBLIC.to_string(),
                        EVE_PUPLIC_KEY.to_string(),
                        constants::DEFAULT.to_string(),
                        constants::YES.to_string(),
                        NeighborInfo::new(),
                        now_);

                    self.save_settings();

                    // add Hu address to wallet
                    insert_hu_key();
                } else if clone_id == 1
                {
                    println!("Setting machine as a developing node (Hu)");
                    // set profile as hu@imagine.com
                    self.m_profile.m_mp_settings.m_machine_alias = "node-hu".to_string();
                    self.m_profile.m_mp_settings.m_public_email.m_pgp_private_key = HU_PRIVATE_KEY.to_string();
                    self.m_profile.m_mp_settings.m_public_email.m_pgp_public_key = HU_PUPLIC_KEY.to_string();
                    self.m_profile.m_mp_settings.m_public_email.m_address = HU_PUBLIC_EMAIL.to_string();

                    if user_and_hu_are_neighbor
                    {
                        // add user as a neighbor
                        let now_ = application().now();
                        add_a_new_neighbor(
                            USER_PUBLIC_EMAIL.to_string(),
                            constants::PUBLIC.to_string(),
                            USER_PUPLIC_KEY.to_string(),
                            constants::DEFAULT.to_string(),
                            constants::YES.to_string(),
                            NeighborInfo::new(),
                            now_);
                    }

                    // add Eve as a neighbor
                    let now_ = application().now();
                    add_a_new_neighbor(
                        EVE_PUBLIC_EMAIL.to_string(),
                        constants::PUBLIC.to_string(),
                        EVE_PUPLIC_KEY.to_string(),
                        constants::DEFAULT.to_string(),
                        constants::YES.to_string(),
                        NeighborInfo::new(),
                        now_);

                    self.save_settings();

                    // add Hu address to wallet
                    insert_hu_key();
                } else if self.m_clone_id == 2
                {
                    println!("Setting machine as a developing node (Eve)");
                    // set profile as eve@imagine.com
                    self.m_profile.m_mp_settings.m_machine_alias = "node-eve".to_string();
                    self.m_profile.m_mp_settings.m_public_email.m_address = EVE_PUBLIC_EMAIL.to_string();
                    self.m_profile.m_mp_settings.m_public_email.m_pgp_private_key = EVE_PRIVATE_KEY.to_string();
                    self.m_profile.m_mp_settings.m_public_email.m_pgp_public_key = EVE_PUPLIC_KEY.to_string();

                    // add User as a neighbor
                    let now_ = application().now();
                    add_a_new_neighbor(
                        USER_PUBLIC_EMAIL.to_string(),
                        constants::PUBLIC.to_string(),
                        USER_PUPLIC_KEY.to_string(),
                        constants::DEFAULT.to_string(),
                        constants::YES.to_string(),
                        NeighborInfo::new(),
                        now_);

                    // add Hu as a neighbor
                    let now_ = application().now();
                    add_a_new_neighbor(
                        HU_PUBLIC_EMAIL.to_string(),
                        constants::PUBLIC.to_string(),
                        HU_PUPLIC_KEY.to_string(),
                        constants::DEFAULT.to_string(),
                        constants::YES.to_string(),
                        NeighborInfo::new(),
                        now_);

                    // add Bob as a neighbor
                    let now_ = application().now();
                    add_a_new_neighbor(
                        HU_PUBLIC_EMAIL.to_string(),
                        constants::PUBLIC.to_string(),
                        BOB_PUPLIC_KEY.to_string(),
                        constants::DEFAULT.to_string(),
                        constants::YES.to_string(),
                        NeighborInfo::new(),
                        now_);

                    self.save_settings();
                } else if self.m_clone_id == 3
                {
                    println!("Setting machine as a developing node (Bob)");
                    // set profile as bob@imagine.com
                    self.m_profile.m_mp_settings.m_machine_alias = "node-bob".to_string();
                    self.m_profile.m_mp_settings.m_public_email.m_address = BOB_PUBLIC_EMAIL.to_string();
                    self.m_profile.m_mp_settings.m_public_email.m_pgp_private_key = BOB_PRIVATE_KEY.to_string();
                    self.m_profile.m_mp_settings.m_public_email.m_pgp_public_key = BOB_PUPLIC_KEY.to_string();

                    // add Eve as a neighbor
                    let now_ = application().now();
                    add_a_new_neighbor(
                        EVE_PUBLIC_EMAIL.to_string(),
                        constants::PUBLIC.to_string(),
                        EVE_PUPLIC_KEY.to_string(),
                        constants::DEFAULT.to_string(),
                        constants::YES.to_string(),
                        NeighborInfo::new(),
                        now_);

                    // add Alice as a neighbor
                    let now_ = application().now();
                    add_a_new_neighbor(
                        ALICE_PUBLIC_EMAIL.to_string(),
                        constants::PUBLIC.to_string(),
                        ALICE_PUPLIC_KEY.to_string(),
                        constants::DEFAULT.to_string(),
                        constants::YES.to_string(),
                        NeighborInfo::new(),
                        now_);

                    self.save_settings();
                } else if self.m_clone_id == 4
                {
                    println!("Setting machine as a developing node (Alice)");
                    // set profile as alice@imagine.com
                    self.m_profile.m_mp_settings.m_machine_alias = "node-alice".to_string();
                    self.m_profile.m_mp_settings.m_public_email.m_address = ALICE_PUBLIC_EMAIL.to_string();
                    self.m_profile.m_mp_settings.m_public_email.m_pgp_private_key = ALICE_PRIVATE_KEY.to_string();
                    self.m_profile.m_mp_settings.m_public_email.m_pgp_public_key = ALICE_PUPLIC_KEY.to_string();

                    // add Hu as a neighbor
                    let now_ = application().now();
                    add_a_new_neighbor(
                        HU_PUBLIC_EMAIL.to_string(),
                        constants::PUBLIC.to_string(),
                        HU_PUPLIC_KEY.to_string(),
                        constants::DEFAULT.to_string(),
                        constants::YES.to_string(),
                        NeighborInfo::new(),
                        now_);

                    // add Bob as a neighbor
                    let now_ = application().now();
                    add_a_new_neighbor(
                        BOB_PUBLIC_EMAIL.to_string(),
                        constants::PUBLIC.to_string(),
                        BOB_PUPLIC_KEY.to_string(),
                        constants::DEFAULT.to_string(),
                        constants::YES.to_string(),
                        NeighborInfo::new(),
                        now_);

                    self.save_settings();
                }
            }
        }

        return true;
    }
}

pub fn insert_hu_key() -> bool
{
    let profile: MachineProfile = get_hu_profile();
    println!("xxxxx profile {:?}", profile);
    let w_address = WalletAddress::new(
        &profile.m_mp_settings.m_backer_detail,
        constants::DEFAULT.to_string(),   // mp code
        "Backer Address (".to_owned() +
            &profile.m_mp_settings.m_backer_detail.m_unlock_sets[0].m_signature_type + &" ".to_owned() +
            &profile.m_mp_settings.m_backer_detail.m_unlock_sets[0].m_signature_ver + &")".to_owned(),
        application().now(),
    );
    let (status, msg) = insert_address(&w_address);
    if !status
    {
        println!("Failed in Hu info insertion {}", msg);
    }
    return status;
}