
//old_name_was smartPullQ
pub fn smart_pull_q()->bool
{
/*
  auto[complete_query, values] = prepareSmartQuery(1);
  QueryRes packets = DbModel::customQuery(
    "",
    complete_query,
    stbl_parsing_q_fields,
    0,
    values,
    false,
    false
  );

  if (packets.records.size() == 0)
  {
    CLog::log("No packet in parsing q", "app", "trace");
    return {};
  }

  QVDicT packet = packets.records[0];
  auto unwrap_res = BlockUtils::unwrapSafeContentForDB(packet.value("pq_payload").to_string());
  if (!unwrap_res.status)
  {
    // purge record
    // reputation report
    return false;
  }
  QJsonObject Jpayload = CUtils::parseToJsonObj(unwrap_res.content);
  packet["pq_payload"] = Jpayload;

  increaseToparseAttempsCountSync(packet);

  auto[status, should_purge_record] = handlePulledPacket(packet);
  if (should_purge_record == false)
  {
    CLog::log("Why not purge1! pq_type(" + packet.value("pq_type").to_string() + ") block(" + CUtils::hash8c(packet.value("pq_code").to_string()) + ")" + " from(" + packet.value("pq_sender").to_string() + ")", "app", "error");

  } else {
    DbModel::dDelete(
      stbl_parsing_q,
      {
        {"pq_sender", packet["pq_sender"]},
        {"pq_type", packet["pq_type"]},
        {"pq_code", packet["pq_code"]}
      });
  }

  InitializeNode::refreshGUI();
  return status;
  */
    true
}

/*
std::tuple<QString, QVDicT> ParsingQHandler::prepareSmartQuery(const uint16_t &limit_)
{
  QString fields = stbl_parsing_q_fields.join(", ");
  QString limit = QString(" LIMIT %1 ").arg(limit_);
  // TODO: make a more intelligence query
  QString query;
  srand(time(0));
  uint8_t query_number = rand() % 100;
  if (CMachine::isInSyncProcess())
  {
//    CLog::log("Smart query number: " + QString::number(query_number), "app", "trace");

   if (query_number < 60)
   {
     // since it is in sync phase, so maybe better order is based on creationdate(TODO: optimize it to prevent cheater to vector attack)
     if (CConsts::DATABASAE_AGENT == "psql")
     {
       query = "SELECT " + fields + " FROM " + stbl_parsing_q;
       query += " WHERE pq_prerequisites=?";
       query += " ORDER BY pq_connection_type ASC, pq_creation_date ASC " + limit;
     }
     else if (CConsts::DATABASAE_AGENT == "sqlite")
     {
       query = "SELECT " + fields + " FROM " + stbl_parsing_q;
       query += " WHERE pq_prerequisites=:pq_prerequisites";
       query += " ORDER BY pq_connection_type ASC, pq_creation_date ASC " + limit;
     }

   } else if ((query_number > 60) && (query_number < 90))
   {
     if (CConsts::DATABASAE_AGENT == "psql")
     {
       query = "SELECT " + fields + " FROM " + stbl_parsing_q;
       query += " WHERE pq_prerequisites=?";
       query += " ORDER BY pq_connection_type ASC, pq_parse_attempts ASC, pq_receive_date ASC " + limit;
     }
     else if (CConsts::DATABASAE_AGENT == "sqlite")
     {
       query = "SELECT " + fields + " FROM " + stbl_parsing_q;
       query += " WHERE pq_prerequisites=:pq_prerequisites";
       query += " ORDER BY pq_connection_type ASC, pq_parse_attempts ASC, pq_receive_date ASC " + limit;
     }

   } else {
     if (CConsts::DATABASAE_AGENT == "psql")
     {
       query = "SELECT " + fields + " FROM " + stbl_parsing_q;
       query += " WHERE pq_prerequisites=? " + limit;
     }
     else if (CConsts::DATABASAE_AGENT == "sqlite")
     {
       query = "SELECT " + fields + " FROM " + stbl_parsing_q;
       query += " WHERE pq_prerequisites=:pq_prerequisites " + limit;
     }

   }

  } else {
   if (query_number < 60)
   {
     if (CConsts::DATABASAE_AGENT == "psql")
     {
       query = "SELECT " + fields + " FROM " + stbl_parsing_q;
       query += " WHERE pq_prerequisites=?";
       query += " ORDER BY pq_connection_type ASC, pq_parse_attempts ASC, pq_receive_date ASC " + limit;
     }
     else if (CConsts::DATABASAE_AGENT == "sqlite")
     {
       query = "SELECT " + fields + " FROM " + stbl_parsing_q;
       query += " WHERE pq_prerequisites=:pq_prerequisites";
       query += " ORDER BY pq_connection_type ASC, pq_parse_attempts ASC, pq_receive_date ASC " + limit;
     }

   } else if ((query_number > 60) && (query_number < 90)) {
     if (CConsts::DATABASAE_AGENT == "psql")
     {
       query = "SELECT " + fields + " FROM " + stbl_parsing_q;
       query += " WHERE pq_prerequisites=?";
       query += " ORDER BY pq_connection_type ASC, pq_creation_date ASC " + limit;
     }
     else if (CConsts::DATABASAE_AGENT == "sqlite")
     {
       query = "SELECT " + fields + " FROM " + stbl_parsing_q;
       query += " WHERE pq_prerequisites=:pq_prerequisites";
       query += " ORDER BY pq_connection_type ASC, pq_creation_date ASC " + limit;
     }

   } else {
     if (CConsts::DATABASAE_AGENT == "psql")
     {
       query = "SELECT " + fields + " FROM " + stbl_parsing_q;
       query += " WHERE pq_prerequisites=? " + limit;
     }
     else if (CConsts::DATABASAE_AGENT == "sqlite")
     {
       query = "SELECT " + fields + " FROM " + stbl_parsing_q;
       query += " WHERE pq_prerequisites=:pq_prerequisites " + limit;
     }

   }

  }

  QVDicT values {{"pq_prerequisites", ","}};
  return { query, values };
}

bool ParsingQHandler::increaseToparseAttempsCountSync(const QVDicT &packet)
{
  try
  {
    return DbModel::update(
      stbl_parsing_q,
      {{"pq_parse_attempts", packet.value("pq_parse_attempts").toUInt() + 1},
      {"pq_last_modified", CUtils::getNow() }},
      {{"pq_type", packet.value("pq_type").to_string()},
      {"pq_code", packet.value("pq_code").to_string()},
      {"pq_sender", packet.value("pq_sender").to_string()}});
  return true;
  } catch (std::exception) {
    return false;
  }
}

 */