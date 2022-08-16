use std::collections::HashMap;
use postgres::types::ToSql;
use crate::{constants, cutils, dlog, machine};
use crate::lib::custom_types::{CBlockHashT, QVDRecordsT, VString, VVString};
use crate::lib::dag::dag::searchInDAG;
use crate::lib::database::abs_psql::{ModelClause, q_delete, q_insert, q_select, simple_eq_clause};
use crate::lib::database::tables::{STBL_SENDING_Q, STBLDEV_SENDING_Q};
use crate::lib::network::broadcast_logger::{addSentBlock, listSentBloksIds};
use crate::lib::parsing_q_handler::queue_utils::searchParsingQ;
use crate::lib::pgp::cpgp::encrypt_pgp;
use crate::lib::utils::dumper::{dump_hashmap_of_QVDRecordsT, dump_it};


pub fn preparePacketsForNeighbors(
    sq_type: &str,
    sq_code: &str,
    sq_payload: &str,
    sq_title: &str,
    sq_receivers: &Vec<String>,
    no_receivers: &Vec<String>,
    deny_double_send_check: bool) -> VVString
{
    dlog(
        &format!("prepare Packets For Neighbors args: sq_type: {}", sq_type),
        constants::Modules::App,
        constants::SecLevel::Trace);

    if sq_receivers.len() > 0
    {
        dlog(
            &format!("targeted packet to Receivers: {:?}", sq_receivers),
            constants::Modules::App,
            constants::SecLevel::Trace);
    }

    if no_receivers.len() > 0
    {
        dlog(
            &format!("no targeted packet to {}", dump_it(no_receivers)),
            constants::Modules::App,
            constants::SecLevel::Trace);
    }

    let mp_code = machine().getSelectedMProfile();
    let mut neighbors: QVDRecordsT = machine().getActiveNeighbors(&mp_code);
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
        constants::SecLevel::Trace);

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
        &format!("Final Selected Neighbors= {}", dump_hashmap_of_QVDRecordsT(&neighbors)),
        constants::Modules::App,
        constants::SecLevel::Trace);

    if neighbors.len() == 0
    {
        dlog(
            &format!("There is no neighbor to send prepare Packets For Neighbors"),
            constants::Modules::App,
            constants::SecLevel::Trace);
        return vec![];
    }

    // let pub_email_info: &EmailSettings = machine().getPubEmailInfo();
    // let prive_email_info: &EmailSettings = machine().getPrivEmailInfo();

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
            sender = machine().getPrivEmailInfo().m_address.clone();
            sender_priv_key = machine().getPrivEmailInfo().m_pgp_private_key.clone();
        } else {
            sender = machine().getPubEmailInfo().m_address.clone();
            sender_priv_key = machine().getPubEmailInfo().m_pgp_private_key.clone();
        }

        let key: String = vec![sq_type, sq_code, &*sender, &receiver_email].join("");

        if listSentBloksIds().contains(&key)
        {
            dlog(
                &format!("already send packet! {}", key),
                constants::Modules::App,
                constants::SecLevel::Error);
            if !deny_double_send_check
            { continue; }
        }

        let (pgp_status, email_body) = encrypt_pgp(
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
        // emailBody = cutils::breakByBR(emailBody);
        // emailBody = wrapPGPEnvelope(emailBody);

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


        addSentBlock(&mut HashMap::from([
            ("lb_type", &sq_type as &(dyn ToSql + Sync)),
            ("lb_code", &sq_code as &(dyn ToSql + Sync)),
            ("lb_title", &sq_title as &(dyn ToSql + Sync)),
            ("lb_sender", &sender as &(dyn ToSql + Sync)),
            ("lb_send_date", &cutils::get_now() as &(dyn ToSql + Sync)),
            ("lb_receiver", &receiver_email as &(dyn ToSql + Sync)),
            ("lb_connection_type", &connection_type as &(dyn ToSql + Sync))
        ]));
    }
    return packets;

    //TODO after successfull sending must save some part the result and change the email to confirmed
}


pub fn pushIntoSendingQ(
    sq_type: &str,
    sq_code: &str,
    sq_payload: &str,
    sq_title: &str,
    sq_receivers: &Vec<String>,
    no_receivers: &Vec<String>,
    denay_double_send_check: bool,
) -> bool
{
    let packets: VVString = preparePacketsForNeighbors(
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
            STBL_SENDING_Q,
            &vec!["sq_type", "sq_code"],
            &vec![
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
            let now = cutils::get_now();
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
                STBL_SENDING_Q,
                &values,
                false);

            if machine().is_develop_mod()
            {
                let (_status, records) = q_select(
                    STBLDEV_SENDING_Q,
                    &vec!["sq_type", "sq_code"],
                    &vec![
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
                        STBLDEV_SENDING_Q,
                        &values,
                        false);
                }
            }
        }
    }
    return true;
}

/*
QVDRecordsT SendingQHandler::fetchFromSendingQ(
  StringList fields,
  ClausesT clauses,
  OrderT order)
{
  if (fields.len() == 0)
    fields = stbl_sending_q_fields;

  QueryRes cpackets = DbModel::select(
    stbl_sending_q,
    fields,
    clauses,
    order);
  return cpackets.records;
}

*/
pub fn cancelIvokeBlockRequest(block_hash: &CBlockHashT)
{
    q_delete(
        STBL_SENDING_Q,
        &vec![
            simple_eq_clause("sq_type", constants::message_types::DAG_INVOKE_BLOCK),
            simple_eq_clause("sq_code", block_hash),
        ],
        false);
}

pub fn maybeCancelIvokeBlocksRequest()
{

    // TODO: optimize it
    let (status, records) = q_select(
        STBL_SENDING_Q,
        &vec!["sq_code"],
        &vec![simple_eq_clause("sq_type", constants::message_types::DAG_INVOKE_BLOCK)],
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

    let hashes = hashes.iter().map(|x| x.as_str()).collect::<Vec<&str>>();
    let existed_in_DAG: QVDRecordsT = searchInDAG(
        &vec![ModelClause {
            m_field_name: "b_hash",
            m_field_single_str_value: "",
            m_clause_operand: "IN",
            m_field_multi_values: hashes.clone(),
        }],
        &vec!["b_hash"],
        vec![],
        0,
        false,
    );
    dlog(
        &format!("Potentially block invoke but existed In DAG({})", existed_in_DAG.len()),
        constants::Modules::App,
        constants::SecLevel::Info);

    for a_block in existed_in_DAG
    {
        cancelIvokeBlockRequest(&a_block["b_hash"].to_string());
    }

    // remove existed in parsing q
    let existed_in_parsing_queue: QVDRecordsT = searchParsingQ(
        &vec![ModelClause {
            m_field_name: "pq_code",
            m_field_single_str_value: "",
            m_clause_operand: "IN",
            m_field_multi_values: hashes,
        }],
        &vec!["pq_code"],
        vec![],
        0,
    );
    dlog(
        &format!("Potentially block invoke but existed In Parsing queue({})", existed_in_parsing_queue.len()),
        constants::Modules::App,
        constants::SecLevel::Info);

    for a_block in existed_in_parsing_queue
    {
        cancelIvokeBlockRequest(&a_block["pq_code"].to_string());
    }
}

//old_name_was sendOutThePacket
pub fn send_out_the_packet() -> bool
{
    maybeCancelIvokeBlocksRequest();
    /*

      QVDRecordsT cpackets = fetchFromSendingQ();
      if (cpackets.len() == 0)
      {
        CLog::log("No packet in sending q to Send", "app", "trace");
        return true;
      }

      // always pick the first pkt! TODO: maybe more intelligent solution needed
      QVDicT packet = cpackets[0];
      bool send_res = NetworkHandler::iPush(
        packet["sq_title"].to_string(),
        packet["sq_payload"].to_string(),
        packet["sq_sender"].to_string(),
        packet["sq_receiver"].to_string());

      // remove packet from sending queue
      if (send_res)
        rmoveFromSendingQ({
          {"sq_type", packet["sq_type"]},
          {"sq_code", packet["sq_code"]},
          {"sq_sender", packet["sq_sender"]},
          {"sq_receiver", packet["sq_receiver"]}});
    */
    return true;
}

/*
bool SendingQHandler::rmoveFromSendingQ(const ClausesT& clauses)
{
  DbModel::dDelete(
    stbl_sending_q,
    clauses);
  return true;
}

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