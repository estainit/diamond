use crate::{constants, dlog};
use crate::lib::custom_types::JSonObject;
use crate::lib::messaging_protocol::dispatcher::PacketParsingResult;
use crate::lib::messaging_protocol::dispatcher_a_single_block::handle_a_single_block;
use crate::lib::messaging_protocol::dispatcher_a_single_card::handle_a_single_card;

//old_name_was innerDispatchMessage
pub fn dispatch_a_card(
    sender: &String,
    connection_type: &String,
    c_date: &String,
    card_body: &mut JSonObject,
    card_type: &String,
    card_ver: &String,
    packet_ver: &str) -> PacketParsingResult
{
    dlog(
        &format!("--- dispatching card({}) from({}) ", card_type, sender),
        constants::Modules::App,
        constants::SecLevel::Info);

    // FIXME: security issue. what happend if adversary creates million of blocks in minute and send the final descendente?
    // in this case all nodes have to download entire blocks all the way back to find ancestor
    // and start to validate from the oldest one and add it to DAG(if is VALID)
    // in this process nodes can not control if the blocks in between are valid or not?
    // so the bandwidth&  machine harddisk will be vasted
    // and network will be blocked!
    // here we need implement a system to control creation date of eache received block(profiled for each neighbor or backer address)
    // and limit creating block(e.g 10 bloocks per minute) in proportion to neighbor's reputation.


    if constants::THE_BLOCK_TYPES.contains(&card_type.as_str())
    {
        return handle_a_single_block(
            sender,
            connection_type,
            card_body,
            card_type);
    } else if constants::THE_CARD_TYPES.contains(&card_type.as_str())
    {
        let pa_pa_res = handle_a_single_card(
            sender,
            connection_type,
            c_date,
            card_body,
            card_type,
            card_ver,
            packet_ver);

        dlog(
            &format!("Handle a single card response status({}) should purge file({})",
                     pa_pa_res.m_status, pa_pa_res.m_should_purge_file),
            constants::Modules::App,
            constants::SecLevel::Info);

        return pa_pa_res;
    }

    let msg = format!(
        "Reached end of func! sender: {}, connection type: {}, c_date: {}, card type: {}, card ver:{}, packet ver: {}, card body: {:?}",
        sender, connection_type, c_date, card_type, card_ver, packet_ver, card_body);
    return PacketParsingResult {
        m_status: false,
        m_should_purge_file: false,
        m_message: msg,
    };
}
