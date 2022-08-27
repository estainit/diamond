use std::collections::HashMap;
use crate::{application, cutils};
use crate::lib::block::block_types::block::Block;
use crate::lib::block::block_types::block_coinbase::create_coinbase_core::create_coinbase_core;
use crate::lib::block::block_types::block_floating_signature::floating_signature_block::aggrigate_floating_signatures;
use crate::lib::constants;
use crate::lib::dag::leaves_handler::{get_leave_blocks, LeaveBlock};
use crate::lib::dlog::dlog;


//old_name_was doGenerateCoinbaseBlock
pub fn do_generate_coinbase_block(
    cycle: &str,
    mode: &str,
    version: &str) -> (bool, Block)
{
    dlog(
        &format!("do GenerateCoinbaseBlock cycle({}) mode({})", cycle, mode),
        constants::Modules::CB,
        constants::SecLevel::Info);

    let now_ = application().get_now();
    let (
        _cycle_stamp,
        from,
        to,
        _from_hour,
        _to_hour) = application().get_coinbase_info(&now_, cycle);

    println!("mmmmmmmm ressss: {}, {}, {}, {}, {}",
             _cycle_stamp,
             from,
             to,
             _from_hour,
             _to_hour);

    let (status, mut block) = create_coinbase_core(cycle, mode, version);
    if !status {
        dlog(
            &format!("Failed in create CB Core cycle({cycle}),  mode({mode}),  version({version})"),
            constants::Modules::CB,
            constants::SecLevel::Error);
        return (false, block);
    }

// connecting to existed leaves as ancestors
    let leaves: HashMap<String, LeaveBlock> = get_leave_blocks(&from);
    let mut leaves_hashes: Vec<String> = leaves.keys().cloned().collect::<Vec<String>>();
    leaves_hashes.sort();
    dlog(
        &format!("do GenerateCoinbaseBlock retrieved cbInfo: from_({}) to_({})", from, to),
        constants::Modules::CB,
        constants::SecLevel::Info);
    dlog(
        &format!(
            "do GenerateCoinbaseBlock retrieved leaves from kv: cycle({}) leaves_hashes({}) leaves({})",
            cycle, leaves_hashes.join(", "), serde_json::to_string(&leaves).unwrap()),
        constants::Modules::CB,
        constants::SecLevel::Info);

    println!("GBC ----- 1");
    let now_ = application().get_now();
    let (_confidence, block_hashes, _backers) = aggrigate_floating_signatures(&now_);
    println!("GBC ----- 2");
    leaves_hashes = cutils::array_add(&leaves_hashes, &block_hashes);
    leaves_hashes.sort();
    leaves_hashes.dedup();

// if requested cycle is current cycle and machine hasn't fresh leaves, so can not generate a CB block
    let now_ = application().get_now();
    if (mode == constants::stages::CREATING) &&
        (leaves_hashes.len() == 0) &&
        (cycle == application().get_coinbase_cycle_stamp(&now_))
    {
        if mode == constants::stages::CREATING
        {
            dlog(
                &format!("generating new CB in generating mode failed!! leaves({})", leaves_hashes.join(",")),
                constants::Modules::CB,
                constants::SecLevel::Info);
        } else {
            dlog(
                &format!("strange error generating new CB failed!! mode({}) mode({}) ", mode, leaves_hashes.join(",")),
                constants::Modules::CB,
                constants::SecLevel::Error);
        }
        return (false, block);
    }

    block.m_block_ancestors = leaves_hashes.clone();
    dlog(
        &format!("do GenerateCoinbaseBlock block.ancestors: {}", leaves_hashes.join(",")),
        constants::Modules::CB,
        constants::SecLevel::Info);


    // if the backer is also a shareholder (probably with large amount of shares),
    // would be more usefull if she also signs the dividends by his private key as a shareholder
    // and sends also her corresponding publick key
    // this signature wil be used for 2 reson
    // 1. anti rebuild DAG by adverseries
    // 2. prevent fake sus-blocks to apply to network
    // signing block by backer private key (TODO: or delegated private key for the sake of security)


    return (true, block);
}