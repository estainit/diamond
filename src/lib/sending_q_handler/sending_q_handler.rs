/*
#include "stable.h"

#include "lib/network/network_handler.h"
#include "lib/parsing_q_handler/parsing_q_handler.h"

#include "sending_q_handler.h"

const QString SendingQHandler::stbl_sending_q = "c_sending_q";
const QStringList SendingQHandler::stbl_sending_q_fields = {"sq_id", "sq_type", "sq_code", "sq_title", "sq_sender", "sq_receiver", "sq_connection_type", "sq_payload"};
const QString SendingQHandler::stbldev_sending_q = "cdev_sending_q";

SendingQHandler::SendingQHandler()
{

}


CListListT SendingQHandler::preparePacketsForNeighbors(
    const QString& sq_type,
    const QString& sq_code,
    const QString& sq_payload,
    const QString& sq_title,
    const QStringList& sq_receivers,
    const QStringList& no_receivers,
    const bool& denay_double_send_check
    )
{
  CLog::log("prepare PacketsForNeighbors args: ", "app", "trace");

  if (sq_receivers.size() > 0)
     CLog::log("targeted packet to " + CUtils::dumpIt(sq_receivers), "app", "trace");

  if (no_receivers.size() > 0 )
     CLog::log("no targeted packet to " + CUtils::dumpIt(no_receivers));

  QVDRecordsT neighbors = CMachine::get().getActiveNeighbors();
  if (sq_receivers.size() > 0)
  {
    // keep only requested neighbors
    QVDRecordsT selectedNeighbors;
    for (QVDicT neighbor: neighbors)
      if (sq_receivers.contains(neighbor.value("n_email").toString()))
        selectedNeighbors.push_back(neighbor);
    neighbors = selectedNeighbors;
  }

  if (no_receivers.size() > 0)
  {
     // keep only requested neighbors
     QVDRecordsT selectedNeighbors;
     for (QVDicT neighbor: neighbors)
         if (!no_receivers.contains(neighbor.value("n_email").toString()))
             selectedNeighbors.push_back(neighbor);
     neighbors = selectedNeighbors;
  }

  CLog::log("Finall Selected Neighbors= " + CUtils::dumpIt(neighbors), "app", "trace");

  if (neighbors.size() == 0)
  {
     CLog::log("There is no neighbore to send prepare Packets For Neighbors", "app", "trace");
     return {};
  }

  EmailSettings pub_email_info = CMachine::getPubEmailInfo();
  EmailSettings prive_email_info = CMachine::getPrivEmailInfo();

  CListListT packets;
  QString sender;
  for (QVDicT neighbor: neighbors)
  {
    QString receiver_pub_key = neighbor.value("n_pgp_public_key", "").toString();
    if (receiver_pub_key == "")
      continue;

//     let params = {
//         shouldSign: true,
//         shouldCompress: true,
//         message: args.sq_payload,
//     };
    QString sender_priv_key;
    QString connection_type = neighbor.value("n_connection_type", "").toString();
    QString receiver_email = neighbor.value("n_email", "").toString();
    if (connection_type == CConsts::PRIVATE)
    {
      sender = prive_email_info.m_address;
      sender_priv_key = prive_email_info.m_PGP_private_key;
    } else {
      sender = pub_email_info.m_address;
      sender_priv_key = pub_email_info.m_PGP_private_key;
    }

    QString key = QStringList {sq_type, sq_code, sender, receiver_email}.join("");

    if (BroadcastLogger::listSentBloksIds().contains(key))
    {
      CLog::log("already send packet! " + key, "app", "error");
      if (!denay_double_send_check)
        continue;
    }

    auto[pgp_status, emailBody] = CPGP::encryptPGP(sq_payload, sender_priv_key, receiver_pub_key);
    if (!pgp_status)
    {
      CLog::log("failed in encrypt PGP", "app", "error");
      continue;
    }
    emailBody = CUtils::breakByBR(emailBody);
    emailBody = CPGP::wrapPGPEnvelope(emailBody);

    // control output size
    if (static_cast<uint64_t>(emailBody.length()) > CConsts::MAX_BLOCK_LENGTH_BY_CHAR)
    {
      CLog::log("excedded max packet size for packet type(" + sq_type + ") code(" + sq_code + ")", "app", "error");
      continue;
    }

    packets.append(
      QStringList{
        connection_type,
        sq_title,
        sq_type,
        sq_code,
        sender,
        receiver_email,
        emailBody   //sqPyload
    });


    BroadcastLogger::addSentBlock({
      {"lb_type", sq_type},
      {"lb_code", sq_code},
      {"lb_title", sq_title},
      {"lb_sender", sender},
      {"lb_receiver", receiver_email},
      {"lb_connection_type", connection_type}
    });
  }
  return packets;

  //TODO after successfull sending must save some part the result and change the email to confirmed
}


bool SendingQHandler::pushIntoSendingQ(
    const QString& sq_type,
    const QString& sq_code,
    const QString& sq_payload,
    const QString& sq_title,
    const QStringList& sq_receivers,
    const QStringList& no_receivers,
    const bool& denay_double_send_check
    )
{
  CListListT packets = preparePacketsForNeighbors(
    sq_type,
    sq_code,
    sq_payload,
    sq_title,
    sq_receivers,
    no_receivers,
    denay_double_send_check);

  CLog::log("prepare PacketsForNeighbors res packets: " + CUtils::dumpIt(packets));

  for (QStringList packet: packets)
  {
    CLog::log("inserting in '_sending_q' " + packet[2] +"-" + packet[3] + " for " + packet[5] + " " + packet[1]);
    QueryRes dblChk = DbModel::select(
      stbl_sending_q,
      {"sq_type", "sq_code"},
      {
        {"sq_type", packet[2]},
        {"sq_code", packet[3]},
        {"sq_sender", packet[4]},
        {"sq_receiver", packet[5]},
      });
    CLog::log("packet pushed to send(" + QString::number(dblChk.records.size()) + ") from " + packet[4] + " to " + packet[5] + " " + packet[2] + "(" + packet[3] + ")", "app", "trace");

    if (dblChk.records.size() == 0)
    {
      QVDicT values {
        {"sq_type", packet[2]},
        {"sq_code", packet[3]},
        {"sq_title", packet[1]},
        {"sq_sender", packet[4]},
        {"sq_receiver", packet[5]},
        {"sq_connection_type", packet[0]},
        {"sq_payload", packet[6]},
        {"sq_send_attempts", 0},
        {"sq_creation_date", CUtils::getNow()},
        {"sq_last_modified", CUtils::getNow()}
      };
      DbModel::insert(
        stbl_sending_q,
        values,
        false,
        false);

      if (CMachine::isDevelopMod())
      {
        QueryRes dblChk = DbModel::select(
          stbldev_sending_q,
          {"sq_type", "sq_code"},
          {
            {"sq_type", packet[2]},
            {"sq_code", packet[3]},
            {"sq_sender", packet[4]},
            {"sq_receiver", packet[5]}});

        if (dblChk.records.size() == 0)
          DbModel::insert(
            stbldev_sending_q,
            values,
            false,
            false);
      }
    }
  }
  return true;
}

QVDRecordsT SendingQHandler::fetchFromSendingQ(
  QStringList fields,
  ClausesT clauses,
  OrderT order)
{
  if (fields.size() == 0)
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
    {{"sq_type", CConsts::MESSAGE_TYPES::DAG_INVOKE_BLOCK},
    {"sq_code", block_hash}});
}

void SendingQHandler::maybeCancelIvokeBlocksRequest()
{
  // TODO: optimize it
  QueryRes existed = DbModel::select(
    stbl_sending_q,
    {"sq_code"},
    {{"sq_type", CConsts::MESSAGE_TYPES::DAG_INVOKE_BLOCK}});
  if(existed.records.size() == 0)
    return;

  QStringList hashes;
  for (QVDicT elm: existed.records)
    hashes.append(elm.value("sq_code").toString());
  CLog::log("Potentially block invoke requests(" + QString::number(existed.records.size()) + ")");

  QVDRecordsT existed_in_DAG = DAG::searchInDAG(
    {{"b_hash", hashes, "IN"}},
    {"b_hash"});
  CLog::log("Potentially block invoke but existed In DAG(" + QString::number(existed_in_DAG.size()) + ")");
  for (QVDicT a_block: existed_in_DAG)
    cancelIvokeBlockRequest(a_block.value("b_hash").toString());

  // remove existed in parsing q
  QVDRecordsT existed_in_parsing_queue = ParsingQHandler::searchParsingQ(
    {{"pq_code", hashes, "IN"}},
    {"pq_code"});
  CLog::log("Potentially block invoke but existed In Parsing queue(" + QString::number(existed_in_parsing_queue.size()) + ")");
  for (QVDicT a_block: existed_in_parsing_queue)
    cancelIvokeBlockRequest(a_block.value("pq_code").toString());

}
*/
//old_name_was sendOutThePacket
pub fn send_out_the_packet() -> bool
{
    /*

      maybeCancelIvokeBlocksRequest();

      QVDRecordsT cpackets = fetchFromSendingQ();
      if (cpackets.size() == 0)
      {
        CLog::log("No packet in sending q to Send", "app", "trace");
        return true;
      }

      // always pick the first pkt! TODO: maybe more intelligent solution needed
      QVDicT packet = cpackets[0];
      bool send_res = NetworkHandler::iPush(
        packet.value("sq_title").toString(),
        packet.value("sq_payload").toString(),
        packet.value("sq_sender").toString(),
        packet.value("sq_receiver").toString());

      // remove packet from sending queue
      if (send_res)
        rmoveFromSendingQ({
          {"sq_type", packet.value("sq_type")},
          {"sq_code", packet.value("sq_code")},
          {"sq_sender", packet.value("sq_sender")},
          {"sq_receiver", packet.value("sq_receiver")}});
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
  QString thread_prefix = "pull_from_sending_q_";
  QString thread_code = QString::number((quint64)QThread::currentThread(), 16);

  while (CMachine::shouldLoopThreads())
  {
    CMachine::reportThreadStatus(thread_prefix, thread_code, CConsts::THREAD_STATE::RUNNING);
    sendOutThePacket();

    CMachine::reportThreadStatus(thread_prefix, thread_code, CConsts::THREAD_STATE::SLEEPING);
    std::this_thread::sleep_for(std::chrono::seconds(CMachine::getSendingQGap()));
  }

  CMachine::reportThreadStatus(thread_prefix, thread_code, CConsts::THREAD_STATE::STOPPED);
  CLog::log("Gracefully stopped thread(" + thread_prefix + thread_code + ") of loop Pull Sending Q");
}

 */