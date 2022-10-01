use crate::lib::block::document_types::document::{Document, set_document_outputs};
use crate::lib::custom_types::JSonObject;
use crate::lib::transactions::basic_transactions::signature_structure_handler::general_structure::{stringify_outputs, TOutput};
use serde::{Serialize, Deserialize};
use substring::Substring;
use crate::{ccrypto, constants, cutils, dlog};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DPCostPaymentDocument
{
    pub m_outputs: Vec<TOutput>,
}

impl DPCostPaymentDocument {
    pub fn new() -> Self
    {
        Self {
            m_outputs: vec![]
        }
    }

    // old name was setByJsonObj
    pub fn set_doc_by_json_obj(&mut self, json_obj: &JSonObject) -> bool {
        if json_obj["outputs"].is_array() {
            self.set_document_outputs(json_obj["outputs"].as_array().unwrap());
        }
        return true;
    }

    //old_name_was setDocumentOutputs
    pub fn set_document_outputs(&mut self, obj: &Vec<JSonObject>) -> bool
    {
        self.m_outputs = set_document_outputs(obj);
        return true;
    }

    pub fn calc_doc_hash(&self, doc: &Document) -> String
    {
        let to_be_hashed_string = self.get_doc_hashable_string(doc);
        dlog(
            &format!("\nHashable string for dp-cost-pay-doc: {}", to_be_hashed_string),
            constants::Modules::App,
            constants::SecLevel::TmpDebug);
        let the_hash = ccrypto::keccak256_dbl(&to_be_hashed_string); // NOTE: absolutely using double hash for more security
        return the_hash;
    }

    pub fn get_doc_hashable_string(&self, doc: &Document) -> String
    {
        let doc_hash_ables: String = format!(
            "dCDate:{},dClass:{},dType:{},dVer:{},outputs:{}",
            doc.m_doc_creation_date,
            doc.m_doc_class,
            doc.m_doc_type,
            doc.m_doc_version,
            stringify_outputs(self.get_outputs()));
        return doc_hash_ables;
    }

    //old_name_was getOutputs
    pub fn get_outputs(&self) -> &Vec<TOutput>
    {
        return &self.m_outputs;
    }

    //old_name_was exportDocToJson
    pub fn export_doc_to_json(&self, doc: &Document, ext_info_in_document: bool) -> JSonObject
    {
        // if (document.keys().contains("dLen"))
        //     document.remove("dLen");
        // if (document.keys().contains("dExtHash"))
        //     document.remove("dExtHash");
        let mut document: JSonObject = doc.export_doc_to_json_super(ext_info_in_document);
        return document;
    }

}

/*


class DPCostPayDocument : public Document
{
public:
  DPCostPayDocument(const QJsonObject& obj);


};


DPCostPayDocument::DPCostPayDocument(const QJsonObject& obj)
{
  setByJsonObj(obj);

}

DPCostPayDocument::~DPCostPayDocument()
{
  deleteOutputs();
}



bool DPCostPayDocument::deleteInputs()
{
  return true;
}

bool DPCostPayDocument::deleteOutputs()
{
  for (TOutput* an_output: m_outputs)
    delete an_output;
  return true;
}

std::tuple<bool, QJsonArray> DPCostPayDocument::exportInputsToJson() const
{
  return {false, QJsonArray {}};
}

std::tuple<bool, QJsonArray> DPCostPayDocument::exportOutputsToJson() const
{
  QJsonArray outputs {};
  if (m_outputs.size() ==0)
    return {false, outputs};

  for(auto an_output: m_outputs)
    outputs.push_back(QJsonArray {
      an_output->m_address,
      QVariant::fromValue(an_output->m_amount).toDouble()});

  return {true, outputs};
}



QString DPCostPayDocument::getRef() const
{
  return "";
}

QVector<COutputIndexT> DPCostPayDocument::getDPIs() const
{
  return {};
}

bool DPCostPayDocument::setDocumentOutputs(const QJsonValue& obj)
{
  QJsonArray outputs = obj.toArray();
  for(QJsonValueRef an_output: outputs)
  {
    QJsonArray oo = an_output.toArray();
    TOutput *o  = new TOutput({oo[0].toString(), static_cast<CMPAIValueT>(oo[1].toDouble())});
    m_outputs.push_back(o);
  }
  return true;
}



QString DPCostPayDocument::calcDocExtInfoHash() const
{
  return CConsts::NO_EXT_HASH;
}

bool DPCostPayDocument::applyDocFirstImpact(const Block &block) const
{
  Q_UNUSED(block);
  // dp cost docs haven't first impact
  return true;
}

std::vector<TInput*> DPCostPayDocument::getInputs() const
{
  return {};
}

std::vector<TOutput*> DPCostPayDocument::getOutputs() const
{
  return m_outputs;
}


//static createDPCostPaymentTrx(args) {
////  let creationDate = _.has(args, 'creationDate') ? args.creationDate : utils.getNow();
////  let treasury = _.has(args, 'treasury') ? args.treasury : 0;
////  let backerNetFee = _.has(args, 'backerNetFee') ? args.backerNetFee : 0;
////  let trx = {
////    hash: "0000000000000000000000000000000000000000000000000000000000000000",
////    dType: iConsts.DOC_TYPES.DPCostPay,
////    dClass: "",
////    dVer: "0.0.0",
////    description: `Data & Process Cost Payment`,
////    creationDate,
////    outputs: [
////        ['TP_DP', treasury],
////        [machine.getMProfileSettingsSync().backerAddress, backerNetFee]
////    ]
//  };
//  trx.hash = trxHashHandler.doHashTransaction(trx);
//  clog.trx.info(`Data & Process Cost Payment transaction: ${JSON.stringify(trx)}`);
//  return trx;
//}

*/