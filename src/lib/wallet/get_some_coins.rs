use std::collections::HashMap;
use crate::{constants, cutils, dlog};
use crate::lib::custom_types::{CCoinCodeT, CMPAISValueT, CMPAIValueT, VString};
use crate::lib::services::society_rules::society_rules::get_transaction_minimum_fee;
use crate::lib::transactions::basic_transactions::signature_structure_handler::general_structure::TInput;

//old_name_was getSomeCoins
#[allow(unused, dead_code)]
pub fn get_some_coins(
    minimum_spendable: CMPAIValueT,
    _selection_method: &str,
    _unlocker_index: u64,
    excluded_coins: VString,
    allowed_to_use_rejected_coins_to_test: bool) -> (bool, String, HashMap<CCoinCodeT, TInput>, CMPAISValueT)
{
    let mut _excluded_coins = excluded_coins;
    let mut _msg: String;
    let mut minimum_spendable = minimum_spendable;
    let selected_coins: HashMap<CCoinCodeT, TInput> = HashMap::new();
    let mut sum_coin_values: CMPAISValueT = 0;

    if minimum_spendable == 0
    {
        minimum_spendable = get_transaction_minimum_fee(&"".to_string());
    }
    dlog(
        &format!(
            "Fetch some inputs equal to {} PAIs to use in a transaction",
            cutils::nano_pai_to_pai(minimum_spendable as CMPAISValueT)),
        constants::Modules::Trx,
        constants::SecLevel::Info);
    /*
        // retrieve rejected transactions
        let rejected_coins_records = searchInRejectedTrx(
            vec![],
            vec![],
            vec![],
            0);
        let mut rejected_coins: VString = vec![];
        for a_rej_coin in rejected_coins_records
        {
            rejected_coins.push(a_rej_coin["rt_coin"].to_string());
        }
        dlog(
            &format!(
                "Rejected coins: {:?}", rejected_coins),
            constants::Modules::Trx,
            constants::SecLevel::Info);

        // intentioinally sending rejected funds to test network
        if allowed_to_use_rejected_coins_to_test
        {
            // empty rejected coins
            rejected_coins = vec![];
        }

        let mut addresses_dict: HashMap<String, (String, UnlockDocument)> = HashMap::new();
        let (wallet_controlled_accounts, _details) = get_addresses_list(
            &"".to_string(),
            vec!["wa_address", "wa_title", "wa_detail"],
            false,
        );
        for an_address in &wallet_controlled_accounts
        {
            let details: UnlockDocument = serde_json::from_str(&an_address["wa_detail"]).unwrap();
            let address_account = an_address["wa_address"].to_string();
            addresses_dict.insert(
                address_account,
                (an_address["wa_title"].to_string(), details));
        }

        let addresses_accounts: VString = addresses_dict.keys().cloned().collect::<Vec<String>>();
        dlog(
            &format!(
                "The wallet/profile controls {} addresses: {} {}",
                addresses_accounts.len(), addresses_accounts.join(", "), rejected_coins.join(", ")),
            constants::Modules::Trx,
            constants::SecLevel::Info);

        let mut spendable_coins: QVDRecordsT = extract_coins_by_addresses(addresses_accounts);
        if spendable_coins.len() == 0
        {
            msg = format!(
                "Wallet couldn't find proper coins to spend {} micro PAIs!",
                cutils::sep_num_3(minimum_spendable as i64));

            dlog(
                &msg,
                constants::Modules::Trx,
                constants::SecLevel::Warning);

            return (false, msg, HashMap::new(), 0);
        }

        dlog(
            &format!("Going to select some of these coins: {:?}", spendable_coins),
            constants::Modules::Trx,
            constants::SecLevel::TmpDebug);

        let mp_code = machine().get_selected_m_profile();
        let locally_marked_coins_records = searchLocallyMarkedUTXOs(
            vec![simple_eq_clause("lu_mp_code", &mp_code)],
            vec![],
            vec![],
            0,
        );

        let mut locally_marked_coins: VString = vec![];
        for a_coin in locally_marked_coins_records
        {
            locally_marked_coins.push(a_coin["lu_coin"].to_string());
        }

        dlog(
            &format!(
                "Localy marked Coins as spend coins({}): {:?}", locally_marked_coins.len(), locally_marked_coins),
            constants::Modules::Trx,
            constants::SecLevel::Info);


        let mut tmpCoins: QVDRecordsT = vec![];
        for aCoin in spendable_coins
        {
            if !locally_marked_coins.contains(&aCoin["ut_coin"].to_string())
            {
                tmpCoins.push(aCoin);
            }
        }

        spendable_coins = tmpCoins;

        dlog(
            &format!(
                "spendable coins after removing localy markewd coins ({}) coins: {}",
                spendable_coins.len(), spendable_coins),
            constants::Modules::Trx,
            constants::SecLevel::Info);


        if spendable_coins.len() == 0
        {
            msg = format!("Wallet hasn't any un-spended coins to use!");
            dlog(
                &msg,
                constants::Modules::Trx,
                constants::SecLevel::Warning);

            return (false, msg, vec![], 0);
        }

        let mut loop_x_count: u64 = 0; // to prevent unlimited cycle
        while (sum_coin_values < minimum_spendable as CMPAISValueT)
            && (spendable_coins.len() > 0) && (loop_x_count < 1000)
        {
            loop_x_count += 1;

            dlog(
                &format!(
                    "Get Some Inputs, excluding these coins: {:?}",
                    excluded_coins),
                constants::Modules::Trx,
                constants::SecLevel::Info);


            // exclude used coins
            let mut tmp_spendable_coins: QVDRecordsT = vec![];
            for a_coin in spendable_coins
            {
                if !excluded_coins.contains(&a_coin["ut_coin"].to_string())
                    && !rejected_coins.contains(&a_coin["ut_coin"].to_string())
                {
                    tmp_spendable_coins.push(a_coin);
                }
            }
            spendable_coins = tmp_spendable_coins;

            dlog(
                &format!(
                    "Going to select one of these real spendable coins: {:?}", spendable_coins),
                constants::Modules::Trx,
                constants::SecLevel::Info);


            let mut the_selected_coin: QVDicT = HashMap::new();
            let mut values: Vec<CMPAIValueT> = vec![];
            let mut objByValues: HashMap<CMPAIValueT, QVDicT> = HashMap::new();
            if selection_method == constants::coin_selecting_method::PRECISE
            {
                let mut values = vec![];
                objByValues= HashMap::new();
                for a_coin in spendable_coins
                {
                    let val: CMPAIValueT = a_coin["ut_o_value"].parse::<CMPAIValueT>().unwrap();
                    values.push(val);
                    objByValues.insert(val, a_coin);
                }
                values.sort();
                values.dedup();
                dlog(
                    &format!(
                        "get Some Coins: values.sort(): {:?}", values),
                    constants::Modules::Trx,
                    constants::SecLevel::Info);


                for a_value in values
                {
                    dlog(
                        &format!(
                            "get Some Coins: controlling if a coin value is greater than needed money! {} is >= {} - {}",
                            cutils::microPAIToPAI6(a_value as CMPAISValueT),
                            cutils::microPAIToPAI6(minimum_spendable as CMPAISValueT),
                            cutils::microPAIToPAI6(sum_coin_values)),
                        constants::Modules::Trx,
                        constants::SecLevel::Info);

                    if (the_selected_coin.keys().cloned().len() == 0)
                        && (a_value >= (minimum_spendable - sum_coin_values))
                    {
                        the_selected_coin = objByValues[a_value];
                    }
                }

                if the_selected_coin.keys().cloned().len() == 0
                {
                    if objByValues.keys().len() == 0
                    {
                        msg = format!("Failed in select coin to spend");
                        dlog(
                            &msg,
                            constants::Modules::Trx,
                            constants::SecLevel::Warning);

                        return (false, msg, vec![], 0);
                    }
                    values.sort();
                    values.reverse();
                    the_selected_coin = objByValues[values[0]];
                }
            } else if selection_method == constants::coin_selecting_method::BIGGER_FIRST
            {
                let mut values: Vec<CMPAIValueT> = vec![];
                let mut objByValues: HashMap<CMPAIValueT, QVDicT> = HashMap::new();
                for a_coin in spendable_coins
                {
                    let val = a_coin["ut_o_value"].parse::<CMPAIValueT>();
                    values.push(val);
                    objByValues.insert(val, a_coin);
                }
                values.sort();
                values.dedup();
                values.reverse();
                the_selected_coin = objByValues[values[0]];
            } else if selection_method == constants::coin_selecting_method::SMALLER_FIRST
            {
                let mut values: Vec<CMPAIValueT> = vec![];
                let mut objByValues: HashMap<CMPAIValueT, QVDicT> = HashMap::new();
                for a_coin in spendable_coins
                {
                    let val: CMPAIValueT = a_coin["ut_o_value"].parse::<CMPAIValueT>().unwrap();
                    values.push(val);
                    objByValues[val] = a_coin;
                }
                values.sort();
                values.dedup();
                the_selected_coin = objByValues[values[0]];
            } else if selection_method == constants::coin_selecting_method::RANDOM
            {
                let spendable_inx = random::<f64>() * spendable_coins.len();
                the_selected_coin = spendable_coins[spendable_inx];
            }
            dlog(
                &format!(
                    "The selected Coin: {:?}", the_selected_coin),
                constants::Modules::Trx,
                constants::SecLevel::Info);


            if the_selected_coin.keys().cloned().len() > 0
            {
                // console.log('the_selected_coin', the_selected_coin);
                let val: CMPAIValueT = the_selected_coin["ut_o_value"].parse::<CMPAIValueT>().unwrap();
                let coin_code: CCoinCodeT = the_selected_coin["ut_coin"].to_string();
                let coin_owner: CAddressT = the_selected_coin["ut_o_address"].to_string();
                excluded_coins.push(coin_code.clone());
                sum_coin_values += val;

                let (_title, detail) = addresses_dict[coin_owner].clone();
                let an_unlock_set = detail.m_unlock_sets[unlocker_index];

                let (doc_hash_, output_index_) = cutils::unpack_coin_code(&coin_code);

                let private_keys = detail.m_private_keys[an_unlock_set.m_salt];
                // ["the_private_keys"][an_unlock_set["salt"].to_string()];
                // let private_keys: detail
                // ["the_private_keys"][an_unlock_set["salt"].to_string()].toArray();
                // let private_keys: cutils::convertJSonArrayToStringList(detail["the_private_keys"][an_unlock_set["salt"].to_string()].toArray());
                selected_coins.insert(coin_code, TInput {
                    m_transaction_hash: doc_hash_,
                    m_output_index: output_index_,
                    m_owner: coin_owner.to_string(),
                    m_amount: val,
                    m_private_keys: private_keys,
                    m_unlock_set: an_unlock_set,
                });
            }
            dlog(
                &format!(
                    "The selected Coin: {:?}", selected_coins),
                constants::Modules::Trx,
                constants::SecLevel::Info);
        }

        if selected_coins.keys().cloned().len() == 0
        {
            msg = format!("No input was selected to spend");
            dlog(
                &msg,
                constants::Modules::Trx,
                constants::SecLevel::Warning);

            return (false, msg, HashMap::new(), 0);
        }
    */
    return (true, "selected".to_string(), selected_coins, sum_coin_values);
}