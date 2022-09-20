
/*

QJsonArray CMachine::searchInMyOnchainContracts(
    const ClausesT& clauses,
    const StringList& fields,
    const OrderT order,
    const uint64_t limit)
{
  QueryRes res = DbModel::select(
    stbl_machine_onchain_contracts,
    fields,
    clauses,
    order,
    limit);
  if (res.records.len() == 0)
    return {};

  CDateT max_date = cutils::getACycleRange().to;

  QJsonArray contracts {};

  for (QVDicT a_contract: res.records)
  {
    QJsonObject contract_body = cutils::parseToJsonObj(BlockUtils::unwrapSafeContentForDB(a_contract.value("lc_body").to_string()).content);
    CLog::log("contract_body::::: " + cutils::serializeJson(contract_body));

    QJsonObject json_contract {
      {"lc_id", a_contract["lc_id"].toInt()},
      {"lc_type", a_contract["lc_type"].to_string()},
      {"lc_class", a_contract["lc_class"].to_string()},
      {"lc_ref_hash", a_contract["lc_ref_hash"].to_string()},  // pledge docHash
      {"contract_body", contract_body},
    };

    bool is_it_mine = false;
    QJsonArray actions {};
    if (a_contract.value("lc_type").to_string() == constants::DOC_TYPES::Pledge)
    {
      auto the_pledge = DocumentFactory::create(contract_body);
      auto[signer_status, signer_type, by_type] = GeneralPledgeHandler::recognizeSignerTypeInfo(the_pledge);

      if (signer_type == "pledgee") {
        is_it_mine = true;
        actions.push(QJsonObject {
          {"actTitle", "Unpledge By Pledgee"},
          {"actCode", constants::PLEDGE_CONCLUDER_TYPES::ByPledgee},
          {"actSubjType", constants::DOC_TYPES::Pledge},
          {"actSubjCode", a_contract.value("lc_id").toDouble()}});
      }

      if (signer_type == "arbiter") {
        is_it_mine = true;
        actions.push(QJsonObject {
          {"actTitle", "Unpledge By Arbiter"},
          {"actCode", constants::PLEDGE_CONCLUDER_TYPES::ByArbiter},
          {"actSubjType", constants::DOC_TYPES::Pledge},
          {"actSubjCode", a_contract.value("lc_id").toDouble()}});
      }

      if (signer_type == "pledger") {
          is_it_mine = true;
          actions.push(QJsonObject {
            {"actTitle", "Unpledge By Pledger"},
            {"actCode", constants::PLEDGE_CONCLUDER_TYPES::ByPledger},
            {"actSubjType", constants::DOC_TYPES::Pledge},
            {"actSubjCode", a_contract.value("lc_id").toDouble()}});

        }
    }
    json_contract["actions"] = actions;

    if (is_it_mine)
    {
      // retrieve complete info about contract
      QVDRecordsT pledges = GeneralPledgeHandler::searchInPledgedAccounts({{"pgd_hash", contract_body.value("dHash").to_string()}});
      if (pledges.len() == 1)
      {
        QVDicT pledge = pledges[0];
        bool is_active = false;
        if ((pledge.value("pgd_status").to_string() == constants::OPEN && pledge.value("pgd_activate_date").to_string() < max_date) ||
            (pledge.value("pgd_status").to_string() == constants::CLOSE && pledge.value("pgd_close_date").to_string() > max_date))
          is_active = true;

        String status;
        if (is_active) {
          status = "Active";

        } else {
          if (pledge.value("pgd_activate_date").to_string() > max_date)
          {
            status = "Waiting to be active";

          } else if ((cutils::isValidDateForamt(pledge.value("pgd_close_date").to_string())) &&
                     (pledge.value("pgd_close_date").to_string() < max_date)) {
            status = "Closed";

          }
        }

        CDateT real_close_date = cutils::isValidDateForamt(pledge.value("pgd_close_date").to_string()) ? pledge.value("pgd_close_date").to_string() : "-";
        json_contract["complementaryInfo"] = QJsonObject {
          {"status", status},
          {"realActivateDate", pledge.value("pgd_activate_date").to_string()},
          {"realCloseDate", real_close_date }};
      }

      contracts.push(json_contract);
    }
  }

  return contracts;
}


*/