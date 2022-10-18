
#[allow(unused, dead_code)]
struct INameRegDocument
{
   m_iname_string:String,
   m_iname_owner:String,

}
/*


#endif // INAMEREGDOCUMENT_H




#include "stable.h"

#include "gui/c_gui.h"
#include "lib/ccrypto.h"
#include "lib/block/document_types/document.h"
#include "lib/services/society_rules/society_rules.h"
#include "lib/services/contracts/flens/iname_handler.h"
#include "lib/services/treasury/treasury_handler.h"
#include "lib/dag/normal_block/import_utxos/utxo_import_data_container.h"

#include "iname_reg_document.h"

INameRegDocument::INameRegDocument(const JSonObject& obj)
{
  set_by_json_obj(obj);
}

INameRegDocument::~INameRegDocument()
{

}

bool INameRegDocument::set_by_json_obj(const JSonObject& obj)
{
  Document::set_by_json_obj(obj);

  if (obj.value("iName").to_string() != "")
    m_iname_string = obj.value("iName").to_string();

  if (obj.value("iNOwner").to_string() != "")
    m_iname_owner = obj.value("iNOwner").to_string();


  return true;
}

JSonObject INameRegDocument::export_doc_to_json(const bool ext_info_in_document) const
{
  JSonObject document = Document::export_doc_to_json(ext_info_in_document);

  document["iName"] = m_iname_string;
  document["iNOwner"] = m_iname_owner;

  return document;
}


String INameRegDocument::safe_stringify_doc(const bool ext_info_in_document) const
{
  JSonObject document_json = export_doc_to_json(ext_info_in_document);

  if (ext_info_in_document)
    document_json["dExtInfo"] = SignatureStructureHandler::compactUnlockersArray(document_json["dExtInfo"].toArray());

//  // recaluculate block final length
//  document_json["dLen"] = cutils::padding_length_value(cutils::serializeJson(document_json).len());

  CLog::log("8 safe Sringify Doc(" + cutils::hash8c(m_doc_hash) + "): " + m_doc_type + " / " + m_doc_class + " length:" + String::number(cutils::serializeJson(document_json).len()) + " serialized document: " + cutils::serializeJson(document_json), "app", "trace");

  return cutils::serializeJson(document_json);
}

// js name was calcCostDINameRegReq
std::tuple<bool, CMPAIValueT> calc_doc_data_and_process_cost(
  const String& stage,
  String cDate,
  const uint32_t& extraLength) const
{
  Q_UNUSED(extraLength);
  if (cDate == "")
    cDate =cutils::getNow();

  DocLenT dLen = m_doc_length;

  CMPAIValueT the_cost =
    dLen *
    SocietyRules::getBasePricePerChar(cDate) *
    SocietyRules::getDocExpense(m_doc_type, dLen, m_doc_class, cDate);

  CMPAIValueT pure_cost = INameHandler::getPureINameRegCost(m_iname_string);
  the_cost += pure_cost;

  if (stage == constants::stages::Creating)
  {
    the_cost = the_cost * machine().get_machine_service_interests(
      m_doc_type,
      m_doc_class,
      dLen);
    CLog::log("calc custom post the_cost + machine interest(" + cutils::sep_num_3(the_cost) +" micro PAIs) type/class(" + m_doc_type + "/" + m_doc_class + ") Doc(" + cutils::hash8c(m_doc_hash) + ")", "app", "trace");
  }
  return {true, cutils::CFloor(the_cost)};
}

  // js name was recordINameInDAG
bool INameRegDocument::applyDocFirstImpact(const Block& block) const
{
  bool res = INameHandler::recordINameInDAG(block, this);
  if (res)
  {
    CGUI::refresh_inames_info();
  }
  return res;
}


bool INameRegDocument::hasSignable() const
{
  return true;
}

// old name was getSignMsgDFleNS
String INameRegDocument::getDocSignMsg() const
{
  String iName = INameHandler::normalizeIName(m_iname_string);
  String signables = "{";
  signables += "\"dCDate\":\"" + m_doc_creation_date + "\"," ;
  signables += "\"dClass\":\"" + m_doc_class + "\"," ;
  signables += "\"dType\":\"" + m_doc_type + "\"," ;
  signables += "\"dVer\":\"" + m_doc_version + "\"," ;
  signables += "\"iName\":\"" + iName + "\"," ;
  signables += "\"iNOwner\":\"" + m_iname_owner + "\"}" ;

  String sign_message = ccrypto::keccak256(signables);
  sign_message = sign_message.midRef(0, constants::SIGN_MSG_LENGTH).to_string();
  CLog::log("IName reg sign_message(" + sign_message + ") signables: " + signables + " ", "app", "trace");
  return sign_message;
}

bool INameRegDocument::veridfyDocSignature() const
{

  JSonObject dExtInfo = m_doc_ext_info[0].toObject();
  auto unlock_set = dExtInfo.value("uSet").toObject();

  bool is_valid_unlock = validate_sig_struct(
    unlock_set,
    m_iname_owner);

  if (!is_valid_unlock)
  {
    CLog::log("Invalid creator signature structure on iname(" + m_doc_hash + ")! ", "sec", "error");
    return false;
  }

  // iname signature & permission validate check
  String sign_message = getDocSignMsg();
  JSonArray signatures = m_doc_ext_info[0].toObject().value("signatures").toArray();
  for (CSigIndexT signature_index = 0; signature_index < signatures.len(); signature_index++)
  {
    String a_signature = signatures[signature_index].to_string();
    try {
      bool verifyRes = ccrypto::ECDSAVerifysignature(
        unlock_set.value("sSets").toArray()[signature_index].toObject().value("sKey").to_string(),
        sign_message,
        a_signature);
      if (!verifyRes)
      {
        CLog::log("Invalid creator signature sign on ballot(" + m_doc_hash + ")! ", "sec", "error");
        return false;
      }
    } catch (std::exception) {
      CLog::log("Exception! Invalid creator signature sign on ballot(" + m_doc_hash + ")! ", "sec", "error");
      return false;
    }
  }
  return true;
}


String INameRegDocument::get_doc_hashable_string() const
{
  String iname = INameHandler::normalizeIName(m_iname_string);
  String hahsables = "{";
  hahsables += "\"dCDate\":\"" + m_doc_creation_date + "\"," ;
  hahsables += "\"dClass\":\"" + m_doc_class + "\"," ;
  hahsables += "\"dExtHash\":" + m_doc_ext_hash + ",";
  hahsables += "\"dLen\":\"" + cutils::padding_length_value(m_doc_length) + "\",";
  hahsables += "\"dType\":\"" + m_doc_type + "\"," ;
  hahsables += "\"dVer\":\"" + m_doc_version + "\"," ;
  hahsables += "\"iName\":\"" + iname + "\"," ;
  hahsables += "\"iNOwner\":\"" + m_iname_owner + "\"}" ;
  return hahsables;
}

// old name was calcHashDINameRegReqS
String INameRegDocument::calcDocHash() const
{
  CLog::log("calc HashDPolling iname reg: " + safe_stringify_doc(), "app", "trace");
  String hashables = get_doc_hashable_string();
  String hash = ccrypto::keccak256(hashables);
  CLog::log("\nHashable string for iname reg doc doc(" + m_doc_type + " / " +
    m_doc_class + ") hash(" + hash + ")" + hashables, "app", "trace");
  return hash;
}

String INameRegDocument::calcDocExtInfoHash() const
{
  String hashables = "{";
  hashables += "\"signatures\":" + cutils::serializeJson(m_doc_ext_info[0].toObject().value("signatures").toArray()) + ",";
  hashables += "\"uSet\":" + safe_stringify_unlock_set(m_doc_ext_info[0].toObject().value("uSet").toObject()) + "}";
  String hash = ccrypto::keccak256(hashables);
  CLog::log("Reg iName Ext Root Hash Hashables Doc(" + m_doc_hash + ") hashables: " + hashables + "\nRegenrated hash: " + hash, "app", "trace");
  return hash;
}

void INameRegDocument::importCostsToTreasury(
  const Block* block,
  CoinImportDataContainer* block_inspect_container)
{

  HashMap<CDocHashT, CostPaymentStatus> cost_payment_status {};

  // handle iName register costs

  if (block_inspect_container.m_block_alter_treasury_incomes.contains("TP_INAME_REG"))
  {
    CLog::log("importing FleNS payments " + CoinImportDataContainer::dumpMe(block_inspect_container.m_block_alter_treasury_incomes["TP_INAME_REG"]), "trx", "trace");

    for (BlockAlterTreasuryIncome a_treasury_entry: block_inspect_container.m_block_alter_treasury_incomes["TP_INAME_REG"])
    {
      // if proposal costs was payed by Pledging & a lending transaction
      bool doc_cost_is_payed = true;

      CDocHashT iName_hash = block_inspect_container.m_map_U_trx_hash_to_trx_ref[a_treasury_entry.m_trx_hash];

      if (block_inspect_container.m_rejected_transactions.keys().contains(a_treasury_entry.m_trx_hash))
      {
        doc_cost_is_payed = false;
        cost_payment_status[iName_hash] = {"supporter transaction(" + cutils::hash8c(a_treasury_entry.m_trx_hash) + ") for reg-iname is rejected because of doublespending", false};
      }

      if (!block_inspect_container.m_map_U_trx_hash_to_trx_ref.keys().contains(a_treasury_entry.m_trx_hash))
      {
        doc_cost_is_payed = false;
        cost_payment_status[a_treasury_entry.m_trx_hash] = {"The iName costs is not supported by any trx", false};
      }


      if (doc_cost_is_payed)
      {
        CLog::log("Successfully TP_INAME_REG Block(" + cutils::hash8c(block->getBlockHash()) + ") Coin(" + cutils::short_coin_code(a_treasury_entry.m_coin) + ") importing(TP_INAME_REG)", "app", "trace");

        String title =  "TP_INAME_REG iName(" + cutils::hash8c(iName_hash) + ") Trx(" + cutils::hash8c(a_treasury_entry.m_trx_hash) + ") ";
        cost_payment_status[iName_hash].m_message = "iName reg Cost imported to treasury succsessfully.";
        insert_income(
          title,
          "TP_INAME_REG",
          title,
          block.m_block_creation_date,
          a_treasury_entry.m_value,
          block->getBlockHash(),
          a_treasury_entry.m_coin);

      } else {
        CLog::log("Failed TP_... Block(" + cutils::hash8c(block->getBlockHash()) + ") Coin(" + cutils::short_coin_code(a_treasury_entry.m_coin) + ") importing(TP_INAME_REG)", "sec", "error");
        CLog::log("cost_payment_status not payed: " + CoinImportDataContainer::dumpMe(cost_payment_status[iName_hash]), "sec", "error");

        INameHandler::removeINameByHash(iName_hash);  // iNameInRelatedBlock.removeINameBecauseOfPaymentsFail(iName_hash);

      }
    }
  }

  block_inspect_container.m_cost_payment_status["TP_INAME_REG"] = cost_payment_status;
}

std::tuple<bool, String> INameRegDocument::signINameRegReq()
{
  String msg;

  QVDRecordsT addresses_details = Wallet::getAddressesInfo({m_iname_owner});
  if (addresses_details.len() != 1)
    return {false, "The owner address " + m_iname_owner + " is not controlled by wallet"};

  JSonObject addrDtl = cutils::parseToJsonObj(addresses_details[0].value("wa_detail").to_string());
  VString signatures {};
  JSonObject dExtInfo {};


  CSigIndexT unlocker_index = 0;  // TODO: the unlocker should be customizable
  JSonObject unlock_set = addrDtl.value("uSets").toArray()[unlocker_index].toObject();

  String sign_message = getDocSignMsg();
  JSonArray sSets = unlock_set.value("sSets").toArray();
  for (CSigIndexT inx = 0; inx < sSets.len(); inx++)
  {
    auto[signing_res, signature_hex, signature] = ccrypto::ECDSAsignMessage(
      addrDtl.value("the_private_keys").toObject()[unlock_set.value("salt").to_string()].toArray()[inx].to_string(),
      sign_message);
    if (!signing_res)
    {
      msg = "Failed in sign pledge, Salt(" + unlock_set.value("salt").to_string() + ")";
      CLog::log(msg, "app", "error");
      return {false, msg};
    }
    signatures.push(String::fromStdString(signature_hex));
  }
  if (signatures.len() == 0)
  {
    msg = "The FleNS couldn't be signed";
    CLog::log(msg, "app", "error");
    return {false, msg};
  }
  m_doc_ext_info = JSonArray{JSonObject {
    {"uSet", unlock_set},
    {"signatures", cutils::convertStringListToJSonArray(signatures)}}};

  return {true, "done"};
}

*/