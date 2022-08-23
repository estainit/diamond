use std::collections::HashMap;
use postgres::types::ToSql;
use crate::{application, constants, cutils, dlog, machine};
use crate::lib::custom_types::{CBlockHashT, ClausesT, OrderT, QVDicT, QVDRecordsT, VVString};
use crate::lib::dag::dag::search_in_dag;
use crate::lib::database::abs_psql::{ModelClause, q_delete, q_insert, q_select, simple_eq_clause};
use crate::lib::database::tables::{C_SENDING_Q, C_SENDING_Q_FIELDS, CDEV_SENDING_Q};
use crate::lib::machine::machine_neighbor::get_active_neighbors;
use crate::lib::network::broadcast_logger::{add_sent_block, list_sent_bloks_ids};
use crate::lib::network::network_handler::i_push;
use crate::lib::parsing_q_handler::queue_utils::search_parsing_q;
use crate::lib::pgp::cpgp::{wrap_pgp_envelope};
use crate::lib::pgp::cpgp_encrypt::pgp_encrypt;
use crate::lib::utils::dumper::{dump_hashmap_of_qvd_records, dump_it};

//old_name_was preparePacketsForNeighbors
pub fn prepare_packets_for_neighbors(
    sq_type: &str,
    sq_code: &str,
    sq_payload: &str,
    sq_title: &str,
    sq_receivers: &Vec<String>,
    no_receivers: &Vec<String>,
    deny_double_send_check: bool) -> VVString
{
    dlog(
        &format!("prepare Packets For Neighbors args: sq_type: ({}/{}), receivers({:?}) not receivers({:?}) title:{}",
                 sq_type, sq_code, sq_receivers, no_receivers, sq_title),
        constants::Modules::App,
        constants::SecLevel::Info);

    if sq_receivers.len() > 0
    {
        dlog(
            &format!("targeted packet to Receivers: {:?}", sq_receivers),
            constants::Modules::App,
            constants::SecLevel::Info);
    }

    if no_receivers.len() > 0
    {
        dlog(
            &format!("no targeted packet to {}", dump_it(no_receivers)),
            constants::Modules::App,
            constants::SecLevel::Info);
    }

    let mp_code = machine().get_selected_m_profile();
    let mut neighbors: QVDRecordsT = get_active_neighbors(&mp_code);
    if sq_receivers.len() > 0
    {
        // keep only requested neighbors
        let mut selected_neighbors: QVDRecordsT = vec![];
        for neighbor in neighbors
        {
            if sq_receivers.contains(&neighbor["n_email"].to_string()) {
                selected_neighbors.push(neighbor);
            }
        }
        neighbors = selected_neighbors;
    }
    dlog(
        &format!("Active neighbors {:?}", neighbors),
        constants::Modules::App,
        constants::SecLevel::Info);

    if no_receivers.len() > 0
    {
        // keep only requested neighbors
        let mut selected_neighbors: QVDRecordsT = vec![];
        for neighbor in neighbors
        {
            if !no_receivers.contains(&neighbor["n_email"].to_string())
            {
                selected_neighbors.push(neighbor);
            }
        }
        neighbors = selected_neighbors;
    }
    dlog(
        &format!("Final Selected Neighbors= {}", dump_hashmap_of_qvd_records(&neighbors)),
        constants::Modules::App,
        constants::SecLevel::Info);

    if neighbors.len() == 0
    {
        dlog(
            &format!("There is no neighbor to send prepare Packets For Neighbors"),
            constants::Modules::App,
            constants::SecLevel::Info);
        return vec![];
    }

    let mut packets: VVString = vec![];
    let mut sender: String;
    for a_neighbor in neighbors
    {
        let receiver_pub_key: String = a_neighbor["n_pgp_public_key"].to_string();
        if receiver_pub_key == ""
        { continue; }

        let sender_priv_key: String;
        let connection_type: String = a_neighbor["n_connection_type"].clone();
        let receiver_email: String = a_neighbor["n_email"].clone();

        if connection_type == constants::PRIVATE
        {
            sender = machine().get_priv_email_info().m_address.clone();
            sender_priv_key = machine().get_priv_email_info().m_pgp_private_key.clone();
        } else {
            sender = machine().get_pub_email_info().m_address.clone();
            sender_priv_key = machine().get_pub_email_info().m_pgp_private_key.clone();
        }

        let key: String = vec![sq_type, sq_code, &*sender, &receiver_email].join("");

        if list_sent_bloks_ids().contains(&key)
        {
            dlog(
                &format!("already send packet! {}", key),
                constants::Modules::App,
                constants::SecLevel::Error);
            if !deny_double_send_check
            { continue; }
        }

        let (pgp_status, email_body) = pgp_encrypt(
            &sq_payload.to_string(),
            &sender_priv_key,
            &receiver_pub_key,
            &"".to_string(),
            "",
            true,
            true);
        if !pgp_status
        {
            dlog(
                &format!("failed in encrypt PGP"),
                constants::Modules::App,
                constants::SecLevel::Error);
            continue;
        }
        let mut email_body = cutils::break_by_br(&email_body, 128);
        email_body = wrap_pgp_envelope(&email_body);

        // control output size
        if email_body.len() > constants::MAX_BLOCK_LENGTH_BY_CHAR
        {
            dlog(
                &format!("excedded max packet size for packet type({}) code({})", sq_type, sq_code),
                constants::Modules::App,
                constants::SecLevel::Error);
            continue;
        }

        packets.push(
            vec![
                connection_type.clone(),
                sq_title.to_string(),
                sq_type.to_string(),
                sq_code.to_string(),
                sender.clone(),
                receiver_email.clone(),
                email_body,   //sqPyload
            ]);

        let now_ = application().get_now();
        add_sent_block(&mut HashMap::from([
            ("lb_type", &sq_type as &(dyn ToSql + Sync)),
            ("lb_code", &sq_code as &(dyn ToSql + Sync)),
            ("lb_title", &sq_title as &(dyn ToSql + Sync)),
            ("lb_sender", &sender as &(dyn ToSql + Sync)),
            ("lb_send_date", &now_ as &(dyn ToSql + Sync)),
            ("lb_receiver", &receiver_email as &(dyn ToSql + Sync)),
            ("lb_connection_type", &connection_type as &(dyn ToSql + Sync))
        ]));
    }
    return packets;

    //TODO after successfull sending must save some part the result and change the email to confirmed
}

//old_name_was pushIntoSendingQ
pub fn push_into_sending_q(
    sq_type: &str,
    sq_code: &str,
    sq_payload: &str,
    sq_title: &str,
    sq_receivers: &Vec<String>,
    no_receivers: &Vec<String>,
    denay_double_send_check: bool,
) -> bool
{
    let packets: VVString = prepare_packets_for_neighbors(
        sq_type,
        sq_code,
        sq_payload,
        sq_title,
        sq_receivers,
        no_receivers,
        denay_double_send_check);

    dlog(
        &format!("prepare PacketsForNeighbors res packets: {:?}", packets),
        constants::Modules::App,
        constants::SecLevel::Trace);

    for packet in packets
    {
        dlog(
            &format!("inserting in '_sending_q' {}-{} for {} {}", packet[2], packet[3], packet[5], packet[1]),
            constants::Modules::App,
            constants::SecLevel::Trace);


        let (_status, records) = q_select(
            C_SENDING_Q,
            vec!["sq_type", "sq_code"],
            vec![
                simple_eq_clause("sq_type", &packet[2]),
                simple_eq_clause("sq_code", &packet[3]),
                simple_eq_clause("sq_sender", &packet[4]),
                simple_eq_clause("sq_receiver", &packet[5]),
            ],
            vec![],
            0,
            true,
        );
        dlog(
            &format!("packet pushed to send({}) from {} to {} {} ({})", records.len(), packet[4], packet[5], packet[2], packet[3]),
            constants::Modules::App,
            constants::SecLevel::Trace);

        if records.len() == 0
        {
            let now = application().get_now();
            let sq_send_attempts = 0;
            let values: HashMap<&str, &(dyn ToSql + Sync)> = HashMap::from([
                ("sq_type", &packet[2] as &(dyn ToSql + Sync)),
                ("sq_code", &packet[3] as &(dyn ToSql + Sync)),
                ("sq_title", &packet[1] as &(dyn ToSql + Sync)),
                ("sq_sender", &packet[4] as &(dyn ToSql + Sync)),
                ("sq_receiver", &packet[5] as &(dyn ToSql + Sync)),
                ("sq_connection_type", &packet[0] as &(dyn ToSql + Sync)),
                ("sq_payload", &packet[6] as &(dyn ToSql + Sync)),
                ("sq_send_attempts", &sq_send_attempts as &(dyn ToSql + Sync)),
                ("sq_creation_date", &now as &(dyn ToSql + Sync)),
                ("sq_last_modified", &now as &(dyn ToSql + Sync))
            ]);
            q_insert(
                C_SENDING_Q,
                &values,
                false);

            if application().is_develop_mod()
            {
                let (_status, records) = q_select(
                    CDEV_SENDING_Q,
                    vec!["sq_type", "sq_code"],
                    vec![
                        simple_eq_clause("sq_type", &packet[2]),
                        simple_eq_clause("sq_code", &packet[3]),
                        simple_eq_clause("sq_sender", &packet[4]),
                        simple_eq_clause("sq_receiver", &packet[5]),
                    ],
                    vec![],
                    0,
                    true);

                if records.len() == 0 {
                    q_insert(
                        CDEV_SENDING_Q,
                        &values,
                        false);
                }
            }
        }
    }
    return true;
}

//old_name_was fetchFromSendingQ
pub fn fetch_from_sending_q(
    mut fields: Vec<&str>,
    clauses: ClausesT,
    order: OrderT) -> QVDRecordsT
{
    if fields.len() == 0 {
        fields = C_SENDING_Q_FIELDS.iter().map(|&x| x).collect::<Vec<&str>>();
    }

    let (_status, records) = q_select(
        C_SENDING_Q,
        fields,
        clauses,
        order,
        0,
        true);
    return records;
}

//old_name_was cancelIvokeBlockRequest
pub fn cancel_ivoke_block_request(block_hash: &CBlockHashT)
{
    q_delete(
        C_SENDING_Q,
        vec![
            simple_eq_clause("sq_type", &constants::card_types::DAG_INVOKE_BLOCK.to_string()),
            simple_eq_clause("sq_code", block_hash),
        ],
        false);
}

//old_name_was maybeCancelIvokeBlocksRequest
pub fn maybe_cancel_ivoke_blocks_request()
{

    // TODO: optimize it
    let (_status, records) = q_select(
        C_SENDING_Q,
        vec!["sq_code"],
        vec![simple_eq_clause("sq_type", &constants::card_types::DAG_INVOKE_BLOCK.to_string())],
        vec![],
        1,
        false,
    );
    if records.len() == 0
    {
        return;
    }
    dlog(
        &format!("Potentially block invoke requests({})", records.len()),
        constants::Modules::App,
        constants::SecLevel::Info);


    let mut hashes: Vec<String> = vec![];
    for elm in records
    {
        let sq_code = elm["sq_code"].clone();
        hashes.push(sq_code);
    }

    let empty_string = "".to_string();
    let mut c1 = ModelClause {
        m_field_name: "b_hash",
        m_field_single_str_value: &empty_string as &(dyn ToSql + Sync),
        m_clause_operand: "IN",
        m_field_multi_values: vec![],
    };
    for a_hash in &hashes {
        c1.m_field_multi_values.push(a_hash as &(dyn ToSql + Sync));
    }
    let existed_in_dag: QVDRecordsT = search_in_dag(
        vec![c1],
        vec!["b_hash"],
        vec![],
        0,
        false,
    );
    dlog(
        &format!("Potentially block invoke but existed In DAG({})", existed_in_dag.len()),
        constants::Modules::App,
        constants::SecLevel::Info);

    for a_block in existed_in_dag
    {
        cancel_ivoke_block_request(&a_block["b_hash"].to_string());
    }

    // remove existed in parsing q
    let empty_string = "".to_string();
    let mut c1 = ModelClause {
        m_field_name: "pq_code",
        m_field_single_str_value: &empty_string as &(dyn ToSql + Sync),
        m_clause_operand: "IN",
        m_field_multi_values: vec![],
    };
    for a_hash in &hashes {
        c1.m_field_multi_values.push(a_hash as &(dyn ToSql + Sync));
    }
    let existed_in_parsing_queue: QVDRecordsT = search_parsing_q(
        vec![c1],
        vec!["pq_code"],
        vec![],
        0,
    );
    dlog(
        &format!("Potentially block invoke but existed In Parsing queue({})", existed_in_parsing_queue.len()),
        constants::Modules::App,
        constants::SecLevel::Info);

    for a_block in existed_in_parsing_queue
    {
        cancel_ivoke_block_request(&a_block["pq_code"].to_string());
    }
}

//old_name_was sendOutThePacket
pub fn send_out_the_packet() -> bool
{
    maybe_cancel_ivoke_blocks_request();

    let cpackets: QVDRecordsT = fetch_from_sending_q(vec![], vec![], vec![]);
    if cpackets.len() == 0
    {
        dlog(
            &format!("No packet in sending q to Send"),
            constants::Modules::App,
            constants::SecLevel::Trace);

        return true;
    }

    // always pick the first pkt! TODO: maybe more intelligent solution needed
    let packet: &QVDicT = &cpackets[0];
    let send_res: bool = i_push(
        &packet["sq_title"].to_string(),
        &packet["sq_payload"].to_string(),
        &packet["sq_sender"].to_string(),
        &packet["sq_receiver"].to_string());

    // remove packet from sending queue
    if send_res {
        rmove_from_sending_q(vec![
            simple_eq_clause("sq_type", &packet["sq_type"]),
            simple_eq_clause("sq_code", &packet["sq_code"]),
            simple_eq_clause("sq_sender", &packet["sq_sender"]),
            simple_eq_clause("sq_receiver", &packet["sq_receiver"]),
        ]);
    }
    return true;
}

//old_name_was rmoveFromSendingQ
pub fn rmove_from_sending_q(clauses: ClausesT) -> bool
{
    q_delete(
        C_SENDING_Q,
        clauses,
        false);
    return true;
}

/*

void SendingQHandler::loopPullSendingQ()
{
  String thread_prefix = "pull_from_sending_q_";
  String thread_code = String::number((quint64)QThread::currentThread(), 16);

  while (CMachine::shouldLoopThreads())
  {
    CMachine::reportThreadStatus(thread_prefix, thread_code, constants::THREAD_STATE::RUNNING);
    sendOutThePacket();

    CMachine::reportThreadStatus(thread_prefix, thread_code, constants::THREAD_STATE::SLEEPING);
    std::this_thread::sleep_for(std::chrono::seconds(CMachine::getSendingQGap()));
  }

  CMachine::reportThreadStatus(thread_prefix, thread_code, constants::THREAD_STATE::STOPPED);
  CLog::log("Gracefully stopped thread(" + thread_prefix + thread_code + ") of loop Pull Sending Q");
}

 */