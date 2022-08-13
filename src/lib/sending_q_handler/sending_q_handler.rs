use std::collections::HashMap;
use crate::{constants, cutils, dlog, machine};
use crate::lib::custom_types::{QVDRecordsT, VVString};
use crate::lib::database::abs_psql::{q_insert, q_select, simple_eq_clause};
use crate::lib::database::tables::STBLDEV_SENDING_Q;
use crate::lib::network::broadcast_logger::{addSentBlock, listSentBloksIds};
use crate::lib::pgp::cpgp::encryptPGP;
use crate::lib::utils::dumper::{dump_hashmap_of_QVDRecordsT, dump_it};


pub fn preparePacketsForNeighbors(
    sq_type: &str,
    sq_code: &str,
    sq_payload: &str,
    sq_title: &str,
    sq_receivers: &Vec<String>,
    no_receivers: &Vec<String>,
    denay_double_send_check: bool) -> VVString
{
    dlog(
        &format!("prepare PacketsForNeighbors args: "),
        constants::Modules::App,
        constants::SecLevel::Trace);

    if sq_receivers.len() > 0
    {
        dlog(
            &format!("targeted packet to {}", dump_it(sq_receivers)),
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

    let mut neighbors: QVDRecordsT = machine().getActiveNeighbors(&machine().getSelectedMProfile());
    if sq_receivers.len() > 0
    {
        // keep only requested neighbors
        let mut selectedNeighbors: QVDRecordsT = vec![];
        for neighbor in neighbors
        {
            if sq_receivers.contains(&neighbor["n_email"].to_string()) {
                selectedNeighbors.push(neighbor);
            }
        }
        neighbors = selectedNeighbors;
    }

    if no_receivers.len() > 0
    {
        // keep only requested neighbors
        let mut selectedNeighbors: QVDRecordsT = vec![];
        for neighbor in neighbors
        {
            if !no_receivers.contains(&neighbor["n_email"].to_string())
            {
                selectedNeighbors.push(neighbor);
            }
        }
        neighbors = selectedNeighbors;
    }
    dlog(
        &format!("Finall Selected Neighbors= {}", dump_hashmap_of_QVDRecordsT(&neighbors)),
        constants::Modules::App,
        constants::SecLevel::Trace);

    if neighbors.len() == 0
    {
        dlog(
            &format!("There is no neighbore to send prepare Packets For Neighbors"),
            constants::Modules::App,
            constants::SecLevel::Trace);
        return vec![];
    }

    // let pub_email_info: &EmailSettings = machine().getPubEmailInfo();
    // let prive_email_info: &EmailSettings = machine().getPrivEmailInfo();

    let mut packets: VVString = vec![];
    let mut sender: String;
    for neighbor in neighbors
    {
        let receiver_pub_key: String = neighbor["n_pgp_public_key"].to_string();
        if receiver_pub_key == ""
        { continue; }

        let sender_priv_key: String;
        let connection_type: String = neighbor["n_connection_type"].clone();
        let receiver_email: String = neighbor["n_email"].clone();

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
            if !denay_double_send_check
            { continue; }
        }

        let (pgp_status, emailBody) = encryptPGP(
            sq_payload,
            &*sender_priv_key,
            &*receiver_pub_key,
            "",
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
        if emailBody.len() > constants::MAX_BLOCK_LENGTH_BY_CHAR
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
                emailBody,   //sqPyload
            ]);


        addSentBlock(&mut HashMap::from([
            ("lb_type", sq_type),
            ("lb_code", sq_code),
            ("lb_title", sq_title),
            ("lb_sender", sender.as_str()),
            ("lb_receiver", receiver_email.as_str()),
            ("lb_connection_type", connection_type.as_str())
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
            STBLDEV_SENDING_Q,
            &vec!["sq_type", "sq_code"],
            &vec![
                &simple_eq_clause("sq_type", &packet[2]),
                &simple_eq_clause("sq_code", &packet[3]),
                &simple_eq_clause("sq_sender", &packet[4]),
                &simple_eq_clause("sq_receiver", &packet[5]),
            ],
            &vec![],
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
            let values: HashMap<&str, &str> = HashMap::from([
                ("sq_type", &*packet[2]),
                ("sq_code", &*packet[3]),
                ("sq_title", &*packet[1]),
                ("sq_sender", &*packet[4]),
                ("sq_receiver", &*packet[5]),
                ("sq_connection_type", &*packet[0]),
                ("sq_payload", &*packet[6]),
                ("sq_send_attempts", "0"),
                ("sq_creation_date", now.as_str()),
                ("sq_last_modified", now.as_str())
            ]);
            q_insert(
                STBLDEV_SENDING_Q,
                &values,
                false);

            if machine().isDevelopMod()
            {
                let (_status, records) = q_select(
                    STBLDEV_SENDING_Q,
                    &vec!["sq_type", "sq_code"],
                    &vec![
                        &simple_eq_clause("sq_type", &packet[2]),
                        &simple_eq_clause("sq_code", &packet[3]),
                        &simple_eq_clause("sq_sender", &packet[4]),
                        &simple_eq_clause("sq_receiver", &packet[5]),
                    ],
                    &vec![],
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

void SendingQHandler::cancelIvokeBlockRequest(const CBlockHashT& block_hash)
{
  DbModel::dDelete(
    stbl_sending_q,
    {{"sq_type", constants::MESSAGE_TYPES::DAG_INVOKE_BLOCK},
    {"sq_code", block_hash}});
}

void SendingQHandler::maybeCancelIvokeBlocksRequest()
{
  // TODO: optimize it
  QueryRes existed = DbModel::select(
    stbl_sending_q,
    {"sq_code"},
    {{"sq_type", constants::MESSAGE_TYPES::DAG_INVOKE_BLOCK}});
  if(existed.records.len() == 0)
    return;

  StringList hashes;
  for (QVDicT elm: existed.records)
    hashes.push(elm["sq_code"].to_string());
  CLog::log("Potentially block invoke requests(" + String::number(existed.records.len()) + ")");

  QVDRecordsT existed_in_DAG = DAG::searchInDAG(
    {{"b_hash", hashes, "IN"}},
    {"b_hash"});
  CLog::log("Potentially block invoke but existed In DAG(" + String::number(existed_in_DAG.len()) + ")");
  for (QVDicT a_block: existed_in_DAG)
    cancelIvokeBlockRequest(a_block["b_hash"].to_string());

  // remove existed in parsing q
  QVDRecordsT existed_in_parsing_queue = ParsingQHandler::searchParsingQ(
    {{"pq_code", hashes, "IN"}},
    {"pq_code"});
  CLog::log("Potentially block invoke but existed In Parsing queue(" + String::number(existed_in_parsing_queue.len()) + ")");
  for (QVDicT a_block: existed_in_parsing_queue)
    cancelIvokeBlockRequest(a_block["pq_code"].to_string());

}
*/
//old_name_was sendOutThePacket
pub fn send_out_the_packet() -> bool
{
    /*

      maybeCancelIvokeBlocksRequest();

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