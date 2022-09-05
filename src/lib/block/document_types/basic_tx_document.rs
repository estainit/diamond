use serde::{Serialize, Deserialize};
use substring::Substring;
use crate::{ccrypto, constants, cutils, dlog};
use crate::lib::block::document_types::document::{Document, set_document_outputs};
use crate::lib::custom_types::{COutputIndexT, JSonArray, JSonObject};
use crate::lib::transactions::basic_transactions::signature_structure_handler::general_structure::{compact_unlockers_array, stringify_inputs, stringify_outputs, TInput, TOutput};
use crate::lib::transactions::trx_utils::{normalize_inputs, normalize_outputs};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct BasicTxDocument
{
    pub m_inputs: Vec<TInput>,
    pub m_outputs: Vec<TOutput>,
    m_data_and_process_payment_indexes: Vec<COutputIndexT>,  // dPIs
}

impl BasicTxDocument {
    pub fn new() -> BasicTxDocument {
        BasicTxDocument {
            m_inputs: vec![],
            m_outputs: vec![],
            m_data_and_process_payment_indexes: vec![],
        }
    }

    //old_name_was setByJsonObj
    pub fn set_doc_by_json_obj(&mut self, json_obj: &JSonObject) -> bool {
        self.m_data_and_process_payment_indexes = vec![];
        let mut inx: usize = 0;
        while !json_obj["dPIs"][inx].is_null() {
            self.m_data_and_process_payment_indexes
                .push(json_obj["dPIs"][inx]
                    .to_string()
                    .parse::<COutputIndexT>()
                    .unwrap());
            inx += 1;
        }

        if json_obj["inputs"].is_array() {
            self.set_document_inputs(&json_obj["inputs"]);
        }

        if json_obj["outputs"].is_array() {
            self.set_document_outputs(json_obj["outputs"].as_array().unwrap());
        }

        return true;
    }

    //old_name_was safeStringifyDoc
    #[allow(unused, dead_code)]
    pub fn safe_stringify_doc(&self, doc: &Document, ext_info_in_document: bool) -> String
    {
        let document: JSonObject = self.export_doc_to_json(doc, ext_info_in_document);

//  // recaluculate block final length
//  document["dLen"] = constants::LEN_PROP_PLACEHOLDER;
//  document["dLen"] = cutils::padding_length_value(cutils::serializeJson(document).len());

        let res: String = cutils::controlled_json_stringify(&document);
        dlog(
            &format!("3 safe Sringify Doc({}): {}/{} length: {} serialized document: {}",
                     cutils::hash8c(&doc.m_doc_hash),
                     doc.m_doc_type,
                     doc.m_doc_class,
                     cutils::sep_num_3(res.len() as i64),
                     res),
            constants::Modules::App,
            constants::SecLevel::TmpDebug);

        return res;
    }

    /*

    String BasicTxDocument::calcDocExtInfoHash() const //JS name was calcTrxExtRootHash()
    {
      StringList hashes = {};
      for (auto an_ext_info_: m_doc_ext_info)
      {
        JSonObject an_ext_info = an_ext_info_.toObject();
        String hashables = "{";
        hashables += "\"uSet\":" + safeStringifyUnlockSet(an_ext_info.value("uSet").toObject()) + ",";
        hashables += "\"signatures\":" + cutils::serializeJson(an_ext_info.value("signatures").toArray()) + "}";
        String hash = ccrypto::keccak256(hashables);
        CLog::log("Doc Ext Root Hash Hashables Doc(" + m_doc_hash + ") Regenrated Ext hash: " + hash + "\nhashables: " + hashables, "app", "trace");

        hashes.push(hash);
      }
      auto[root, final_verifies, version, levels, leaves] = CMerkle::generate(hashes);
      Q_UNUSED(final_verifies);
      Q_UNUSED(version);
      Q_UNUSED(levels);
      Q_UNUSED(leaves);
      return root;
    }

    std::tuple<bool, CMPAIValueT> BasicTxDocument::calcDocDataAndProcessCost(
      const String& stage,
      String cDate,
      const uint32_t& extra_length) const //calcTrxDPCost
    {
      if (cDate == "")
        cDate =application().now();

      DocLenT dLen = m_doc_length;

      if (stage == constants::STAGES::Creating)
        dLen += extra_length + constants::TRANSACTION_PADDING_LENGTH;

      if (stage == constants::STAGES::Validating)
      {
        if (dLen != static_cast<DocLenT>(calc_doc_length()))
        {
          CLog::log("The trx len and local re-calc len are not same! stage(" + stage + ") remoteLen(" + String::number(dLen) + ") local Len(" + String::number(calc_doc_length()) + ") trx(" + cutils::hash8c(m_doc_hash) + ")", "trx", "error");
          return {false, 0};
        }
      } else {
        if (dLen < static_cast<DocLenT>(calc_doc_length()))
        {
          CLog::log("The trx len and local re-calc len are not same! stage(" + stage + ") remoteLen(" + String::number(dLen) + ") local Len(" + String::number(calc_doc_length()) + ") trx(" + cutils::hash8c(m_doc_hash) + ")", "trx", "error");
          return {false, 0};
        }
      }

      if (m_doc_class == constants::TRX_CLASSES::P4P)
        dLen = dLen * get_dpis().len();  // the transaction which new transaction is going to pay for

      uint64_t theCost =
        dLen *
        SocietyRules::getBasePricePerChar(cDate) *
        SocietyRules::getDocExpense(m_doc_type, dLen, m_doc_class, cDate);

      if (stage == constants::STAGES::Creating)
        theCost = theCost * CMachine::getMachineServiceInterests(
          m_doc_type,
          m_doc_class,
          dLen,
          extra_length,
          get_dpis().len());

      return {true, trunc(theCost) };
    }

    bool BasicTxDocument::deleteInputs()
    {
      for (TInput* an_input: m_inputs)
        delete an_input;
      return true;
    }

    bool BasicTxDocument::deleteOutputs()
    {
      for (TOutput* an_output: m_outputs)
        delete an_output;
      return true;
    }

    */
    //old_name_was exportDocToJson
    pub fn export_doc_to_json(&self, doc: &Document, ext_info_in_document: bool) -> JSonObject
    {
        let mut document: JSonObject = doc.export_doc_to_json_super(ext_info_in_document);

        // impacting uSets
        if ext_info_in_document
        {
            document["dExtInfo"] = compact_unlockers_array(&document["dExtInfo"]);
        }

        if self.m_data_and_process_payment_indexes.len() > 0
        {
            document["dPIs"] = self.m_data_and_process_payment_indexes.clone().into();
        }

        return document;
    }

    /*

    std::tuple<bool, JSonArray> BasicTxDocument::exportInputsToJson() const
    {
    JSonArray inputs {};
    for (TInput* an_input: m_inputs)
    inputs.push(JSonArray{
      an_input.m_transaction_hash,
      an_input.m_output_index});
    return {true, inputs};
    }

    std::tuple<bool, JSonArray> BasicTxDocument::exportOutputsToJson() const
    {
    JSonArray outputs {};
    for (TOutput* an_output: m_outputs)
    outputs.push(JSonArray{
      an_output.m_address,
      QVariant::fromValue(an_output.m_amount).toDouble()});
    return {true, outputs};
    }
*/
    pub fn get_doc_hashable_string(&self, doc: &Document) -> String
    {
        let doc_hashables: String = format!(
            "dCDate:{},dClass:{},dComment:{},dExtHash:{},dLen:{},dPIs:{},dRef:{},dType:{},dVer:{},inputs:{},outputs:{}",
            doc.m_doc_creation_date,
            doc.m_doc_class,
            doc.m_doc_comment,
            doc.m_doc_ext_hash,
            cutils::padding_length_value(doc.m_doc_length.to_string(), constants::LEN_PROP_LENGTH),
            serde_json::to_string(&self.m_data_and_process_payment_indexes).unwrap(),
            doc.get_ref(),
            doc.m_doc_type,
            doc.m_doc_version,
            stringify_inputs(self.get_inputs()),
            stringify_outputs(self.get_outputs())
        );
        return doc_hashables;
    }

    // TODO: some unit test for pure hashable
    //old_name_was extractHPureParts_simple
    pub fn extract_hash_pure_parts_simple(&self, _doc: &Document) -> String
    {
        // the hTrx MUST be constant and NEVER change the order of attribiutes (alphabetical)
        // in case of change the version MUST be changed and the code treats it in new manner
        let normalized_inputs: Vec<TInput> = normalize_inputs(&self.m_inputs);


        let (status, normalized_outputs) = normalize_outputs(&self.m_outputs);
        if !status {
            return "".to_string();
        }

        let mut doc_hahsables: String = "{".to_string();
        doc_hahsables += &*("\"inputs\":".to_owned() + &stringify_inputs(&normalized_inputs) + ",");
        doc_hahsables += &*("\"outputs\":".to_owned() + &stringify_outputs(&normalized_outputs) + "}");

        return doc_hahsables.to_string();
    }

    //old_name_was extractTransactionPureHashableParts
    pub fn extract_transaction_pure_hashable_parts(&self, doc: &Document) -> String
    {
        if doc.m_doc_type == constants::document_types::BASIC_TX
        {
            return self.extract_hash_pure_parts_simple(doc);
        }
        return "".to_string();
    }

    //old_name_was getPureHash
    pub fn get_pure_hash(&self, doc: &Document) -> String
    {
        let hashable_string = self.extract_transaction_pure_hashable_parts(doc);
        let the_hash = ccrypto::keccak256_dbl(&hashable_string);    // NOTE: useing double hash for more security
        dlog(
            &format!("get Pure Hash res! hash({}) version({}) hashable string: ({}) trx({})",
                     cutils::hash8c(&the_hash), doc.m_doc_version, hashable_string, cutils::hash8c(&doc.get_doc_hash())),
            constants::Modules::Trx,
            constants::SecLevel::TmpDebug);

        return the_hash;
    }

    pub fn calc_doc_hash(&self, doc: &Document) -> String
    {
        let hashables = self.get_doc_hashable_string(doc);
        let mut the_hash = ccrypto::keccak256_dbl(&hashables); // NOTE: absolutely using double hash for more security

        // generate deterministic part of trx hash
        let pure_hash = self.get_pure_hash(doc);
        the_hash = pure_hash.substring(32, 64).to_string() + &*the_hash.substring(32, 64).to_string();

        dlog(
            &format!("\nHashable string for Basic Trx doc({} / {}) hash({}) version({}) hashables: {}",
                     doc.m_doc_type, doc.m_doc_class, cutils::hash8c(&the_hash), doc.m_doc_version, hashables),
            constants::Modules::App,
            constants::SecLevel::TmpDebug);

        return the_hash;
    }

    /*
        bool BasicTxDocument::applyDocFirstImpact(const Block& block) const
        {
        return false;
        }

        String BasicTxDocument::get_ref() const
        {
        return m_doc_ref;
        }

    */
    pub fn get_dpis(&self) -> &Vec<COutputIndexT>
    {
        return &self.m_data_and_process_payment_indexes;
    }

    //old_name_was getInputs
    pub fn get_inputs(&self) -> &Vec<TInput>
    {
        return &self.m_inputs;
    }

    //old_name_was getOutputs
    pub fn get_outputs(&self) -> &Vec<TOutput>
    {
        return &self.m_outputs;
    }

    //old_name_was setDocumentInputs
    pub fn set_document_inputs(&mut self, obj: &JSonArray) -> bool
    {
        // JSonArray inputs = obj.toArray();
        for an_input in obj.as_array().unwrap() {
            // JSonArray io = an_input.toArray();
            let inp_entry: TInput = TInput {
                m_transaction_hash: an_input[0].to_string(),
                m_output_index: an_input[1].as_u64().unwrap() as i32,
                m_owner: "".to_string(),
                m_amount: 0,
                m_private_keys: vec![],
                m_unlock_set: Default::default(),
            };
            // ({io[0].to_string(), static_cast< COutputIndexT > (io[1].toVariant().toDouble())});
            self.m_inputs.push(inp_entry);
        }
        return true;
    }

    //old_name_was setDocumentOutputs
    pub fn set_document_outputs(&mut self, obj: &Vec<JSonObject>) -> bool
    {
        self.m_outputs = set_document_outputs(obj);
        // // JSonArray outputs = obj.toArray();
        // for an_output in obj.as_array().unwrap() {
        //     // JSonArray oo = an_output.toArray();
        //     let o: TOutput = TOutput {
        //         m_address: an_output[0].to_string(),
        //         m_amount: an_output[1].as_u64().unwrap(),
        //         m_output_charachter: "".to_string(),
        //         m_output_index: 0,
        //     };
        //     // new TOutput({oo[0].to_string(), static_cast<CMPAIValueT>(oo[1].toDouble())});
        //     self.m_outputs.push(o);
        // }
        return true;
    }

    /*

        bool BasicTxDocument::appendOutput(
        CAddressT address,
        CMPAIValueT value)
        {
        TOutput *o  = new TOutput({address, static_cast<CMPAIValueT>(value)});
        m_outputs.push(o);
        return true;
        }

        bool BasicTxDocument::removeOutputByIndex(COutputIndexT index)
        {
        delete m_outputs[index];
        m_outputs.erase(m_outputs.begin() + index, m_outputs.begin() + index + 1);
        return true;
        }


        int64_t BasicTxDocument::getOutputIndexByAddressValue(const String& address_plus_value)
        {
        for (COutputIndexT inx = 0; inx < m_outputs.len(); inx++)
        {
        if (m_outputs[inx].m_address + m_outputs[inx].m_amount == address_plus_value)
          return inx;
        }
        return -1;
        }

        JSonArray BasicTxDocument::generateInputTuples()
        {
        JSonArray input_tuples {};
        for(TInput* the_in: m_inputs)
        {
        JSonArray elm {the_in.m_transaction_hash, QVariant::fromValue(the_in.m_output_index).toDouble()};
        input_tuples.push(elm);
        }
        return input_tuples;
        }

        JSonArray BasicTxDocument::generateOutputTuples()
        {
        JSonArray output_tuples {};
        for(TOutput* out: m_outputs)
        {
        JSonArray elm {out.m_address, QVariant::fromValue(out.m_amount).toDouble()};
        output_tuples.push(elm);
        }
        return output_tuples;
        }

        String BasicTxDocument::getInputOutputSignables(
        const String& sig_hash,
        const CDateT& cDate)
        {
        JSonArray inputs = generateInputTuples();
        JSonArray outputs = generateOutputTuples();
        return getInputOutputSignables(inputs, outputs, sig_hash, getDocRef(), cDate);
        }



        String BasicTxDocument::getInputOutputSignables(
        const JSonArray& inputs_,
        const JSonArray& outputs_,
        const String& sig_hash,
        const CDocHashT& document_ref,
        const CDateT& cDate)
        {
        auto[inputs, outputs] = extractInputOutputs(inputs_, outputs_);

        String signables = "{";
        signables += "\"dCDate\":\"" + cDate + "\",";
        if (document_ref != "")
        signables += "\"dRef\":\"" + document_ref + "\",";
        signables += "\"inputs\":" + stringify_inputs(inputs) + ",";
        signables += "\"outputs\":" + stringify_outputs(outputs) + ",";
        signables += "\"sigHash\":\"" + sig_hash + "\"}";
        return signables;
        }

        String BasicTxDocument::signingInputOutputs(
        const String& private_key,
        const JSonArray& inputs,
        const JSonArray& outputs,
        const String& sig_hash,
        const CDocHashT& document_ref,
        const CDateT& cDate)
        {
        String signables = getInputOutputSignables(inputs, outputs, sig_hash, document_ref, cDate);
        String hash = ccrypto::keccak256_dbl(signables);      // because of securiy, MUST use double hash
        auto[status, signature_hex, signature] = ccrypto::ECDSAsignMessage(
        private_key,
        hash.midRef(0, constants::SIGN_MSG_LENGTH).to_string());
        CLog::log("The transaction has been signed. signature(" + String::fromStdString(signature_hex) + ") hash(" + hash + ") signables: " + signables, "trx", "info");
        return String::fromStdString(signature_hex);
        }

        bool BasicTxDocument::verifyInputOutputsSignature(
        const String& public_key,
        const String& signature,
        const String& sig_hash,
        const CDateT& cDate
          )
        {
        if (public_key == "")
        {
        CLog::log("Missed public_key!", "app", "error");
        return false;
        }

        if (signature == "")
        {
        CLog::log("Missed signature!", "app", "error");
        return false;
        }

        if (sig_hash == "")
        {
        CLog::log("Missed sig_hash!", "app", "error");
        return false;
        }

        String signables = getInputOutputSignables(sig_hash, cDate);
        signables = ccrypto::keccak256_dbl(signables);      // because of securiy, MUST use double hash
        try {
        bool verify_res = ccrypto::ECDSAVerifysignature(
          public_key,
          signables.midRef(0, constants::SIGN_MSG_LENGTH).to_string(),
          signature);
        return verify_res;

        } catch (std::exception) {
        CLog::log("Failed in transaction signature verify trx(" + m_doc_type + " / " + cutils::hash8c(m_doc_hash) + ")", "trx", "error");
        return false;
        }

        }


        // js name was trxSignatureValidator
        bool BasicTxDocument::validateSignatures(
        const QV2DicT& used_coins_dict,
        const StringList& exclude_coins,
        const CBlockHashT& block_hash)
        {
        //  let block = args.block;
        //  let trx = block.docs[args.docInx];
        JSonArray doc_ext_info = m_doc_ext_info;


        String msg;
        CDateT cDate = application().now();

        // for each input must control if given unlock structutr will be finished in a right(and valid) output address?
        // the order of inputs and ext Info ARE IMPORTANT. the wallet MUST sign and send inputs in order to bip 69
        CLog::log("Signature validating for trx(" + cutils::hash8c(m_doc_hash), "trx", "trace");

        QHash<CInputIndexT, QHash<CSigIndexT, StringList> > the_coins_must_be_signed_by_a_single_sign_set {};
        for (CInputIndexT input_index = 0; input_index < m_inputs.len(); input_index++)
        {
        // for each input must find proper block and bech32 address of output and insert into validate function
        CCoinCodeT coin_code = m_inputs[input_index]->getCoinCode();
        // scape validating invalid inputs(in this case double-spended inputs)
        if (exclude_coins.contains(coin_code))
          continue;

        JSonObject an_unlock_document = doc_ext_info[input_index].toObject();
        JSonObject an_unlock_set = an_unlock_document["uSet"].toObject();

        bool is_valid_unlocker = validateSigStruct(
          an_unlock_set,
          used_coins_dict[coin_code].value("ut_o_address").to_string());

        if (!is_valid_unlocker)
        {
          msg = "Invalid block, because of invalid unlock struture! Block(" + cutils::hash8c(block_hash) + ") transaction(" + cutils::hash8c(get_doc_hash()) + ") input-index(" + String::number(input_index) + " unlock structure: " + cutils::dumpIt(an_unlock_set);
          CLog::log(msg, "trx", "error");
          return false;
        }

        // prepare a signature dictionary
        the_coins_must_be_signed_by_a_single_sign_set[input_index] = {};
        JSonArray sign_sets = an_unlock_set.value("sSets").toArray();
        for (CSigIndexT singature_index = 0; singature_index < sign_sets.len(); singature_index++)
        {
          JSonArray sigInfo = an_unlock_document.value("signatures").toArray()[singature_index].toArray();
          auto[inputs, outputs] = extractInputOutputs(
            m_inputs,
            m_outputs,
            sigInfo[1].to_string());
          the_coins_must_be_signed_by_a_single_sign_set[input_index][singature_index] = StringList {};
          for (auto an_inp: inputs)
            the_coins_must_be_signed_by_a_single_sign_set[input_index][singature_index].push(an_inp->getCoinCode());
        }
        }


        for (CInputIndexT input_index = 0; input_index < m_inputs.len(); input_index++)
        {
        auto the_input = m_inputs[input_index];
        // for each input must find proper block and bech32 address of output and insert into validate function
        CCoinCodeT coin_code = the_input->getCoinCode();

        // scape validating invalid inputs(in this case double-spended inputs)
        if (exclude_coins.contains(coin_code))
          continue;

        JSonObject an_unlock_document = doc_ext_info[input_index].toObject();
        JSonObject an_unlock_set = an_unlock_document["uSet"].toObject();
        JSonArray sign_sets = an_unlock_set.value("sSets").toArray();

        // for each input and proper spending way, must control if the signature is valid?
        for (CSigIndexT singature_index = 0; singature_index < sign_sets.len(); singature_index++)
        {
          JSonArray sigInfo = an_unlock_document.value("signatures").toArray()[singature_index].toArray();
          JSonObject aSignSet = sign_sets[singature_index].toObject();
          bool is_verified = verifyInputOutputsSignature(
            aSignSet.value("sKey").to_string(), //pubKey
            sigInfo[0].to_string(), //signature
            sigInfo[1].to_string(), //sig_hash
            m_doc_creation_date);
          if (!is_verified)
          {
            msg = "Invalid given signature for input(" + String::number(input_index) + ") trx(" + m_doc_type + " / " + cutils::hash8c(m_doc_hash) + ") Block(" + cutils::hash8c(block_hash) + ") ";
            CLog::log(msg, "trx", "error");
            return false;
          }

          if (canHaveTimeLockedInput())
          {
            //// control input timeLocks
            //if (_.has(aSignSet, 'iTLock') && (aSignSet.iTLock > 0)) {
            //  clog.trx.info(`>>>>>>>>>>>>> signed RefLocs By A Single SignSet ${utils.stringify(the_coins_must_be_signed_by_a_single_sign_set)}`);
            //  let iTLock = iutils.convertBigIntToJSInt(aSignSet.iTLock);
            //  for (aRefLoc of the_coins_must_be_signed_by_a_single_sign_set[input_index][singature_index])
            //  {
            //      let inputCreationDate = used_coins_dict[aRefLoc].utRefCreationDate;
            //      let inputRedeemDate = utils.minutesAfter(iTLock, inputCreationDate);

            //      msg = `info::: inputTimeLock compareTime(${cDate}) block(${utils.hash6c(block.blockHash)}) trx(${utils.hash6c(trx.hash)}) `;
            //      msg += `input(${input_index} locked for ${iTLock} Minutes after creation) created on(${inputCreationDate}) `;
            //      msg += `has inputTimeLock and can be redeemed after(${inputRedeemDate}) `;
            //      if (cDate < inputRedeemDate) {
            //          msg = `\ninput is not useable because of ${msg}`;
            //          clog.sec.error(msg);
            //          return { err: true, msg, shouldPurgeMessage: true }
            //      } else {
            //          msg = `\ninput is released ${msg}`;
            //          clog.trx.info(msg);
            //      }
            //  }
            //}
          }
        }
        }

        msg = "All trx have valid signatures Block(" + cutils::hash8c(block_hash) + ") ";
        CLog::log(msg, "trx", "trace");
        return true;
        }

        bool BasicTxDocument::canHaveTimeLockedInput()
        {
        return false;
        // TODO: implement input time lock (both flexible and strict input timelock) ASAP
        return (
        (m_doc_type == constants::document_types::BASIC_TX) &&
        StringList{constants::TRX_CLASSES::SimpleTx, constants::TRX_CLASSES::P4P}.contains(m_doc_class)
        );
        }


        std::tuple<std::vector<TInput*>, std::vector<TOutput*> > BasicTxDocument::extractInputOutputs(String sig_hash)
        {
        if (sig_hash == constants::SIGHASH::ALL)
        return {m_inputs, m_outputs};

        if (sig_hash == constants::SIGHASH::NONE)
        return {m_inputs, {}};

        return {m_inputs, m_outputs};
        }

        std::tuple<std::vector<TInput*>, std::vector<TOutput*> > BasicTxDocument::extractInputOutputs(
        std::vector<TInput*> inputs,
        std::vector<TOutput*> outputs,
        String sig_hash)
        {
        if (sig_hash == constants::SIGHASH::ALL) {
        // change nothing
        } else if (sig_hash == constants::SIGHASH::NONE) {
        outputs = {};
        }
        return { inputs, outputs };
        }

        bool BasicTxDocument::validateGeneralRulsForTransaction()
        {
        String msg;
        //  let localHash = trxHashHandler.doHashTransaction(transaction);
        //  if (localHash != transaction.hash) {
        //    msg = `Mismatch trx hash locally recalculated:${localHash} received: ${transaction.hash} block(${utils.hash6c(blockHash)})`;
        //    return { err: true, msg: msg };
        //  }


        if (cutils::isGreaterThanNow(m_doc_creation_date))
        {
        msg = "Transaction whith future creation date is not acceptable " + get_doc_hash();
        CLog::log(msg, "sec", "error");
        return false;
        }

        return true;
        }

        /**
        * @brief BasicTxDocument::equationCheck
        * @param used_coins_dict
        * @param invalid_coins_dict
        * @return {status, msg, total_inputs_amounts, total_outputs_amounts}
        */
        std::tuple<bool, String, CMPAIValueT, CMPAIValueT> BasicTxDocument::equationCheck(
        const QV2DicT& used_coins_dict,
        const QV2DicT& invalid_coins_dict,
        const CBlockHashT& block_hash)
        {
        String msg;
        CMPAIValueT total_inputs_amounts = 0;
        CMPAIValueT total_outputs_amounts = 0;

        if (m_inputs.len() > 0)
        {
        for (TInput* input: m_inputs)
        {
          CCoinCodeT a_coin_code = input->getCoinCode();
          if (used_coins_dict.keys().contains(a_coin_code))
          {
            if (used_coins_dict[a_coin_code].value("ut_o_value").toDouble() >= MAX_COIN_VALUE)
            {
              msg = "The transaction has input bigger than MAX_SAFE_INTEGER! trx(" + m_doc_type + " / " + cutils::hash8c(m_doc_hash) + ") Block(" + cutils::hash8c(block_hash) + ")  value: " + cutils::microPAIToPAI6(used_coins_dict[a_coin_code].value("ut_o_value").toDouble());
              CLog::log(msg, "sec", "error");
              return {false, msg, 0, 0};
            }
            total_inputs_amounts += used_coins_dict[a_coin_code].value("ut_o_value").toDouble();

          } else {
            /**
            * trx uses already spent outputs! so try invalid_coins_dict
            * probably it is a double-spend case, which will be decided after 12 hours, in importing step
            * BTW ALL trx must have balanced equation (even duoble-spendeds)
            */
            if (invalid_coins_dict.keys().contains(a_coin_code))
            {
              if (invalid_coins_dict[a_coin_code].value("coinGenOutputValue").toDouble() >= MAX_COIN_VALUE)
              {
                msg = "The transaction has inv-input bigger than MAX_SAFE_INTEGER! trx(" + m_doc_type + " / " + cutils::hash8c(m_doc_hash) + ") Block(" + cutils::hash8c(block_hash) + ")  value: " + cutils::microPAIToPAI6(invalid_coins_dict[a_coin_code].value("coinGenOutputValue").toDouble());
                CLog::log(msg, "sec", "error");
                return {false, msg, 0, 0};
              }
              total_inputs_amounts += invalid_coins_dict[a_coin_code].value("coinGenOutputValue").toDouble();

            } else {
              msg = "The input absolutely missed! not in tables neither in DAG! coin(" + a_coin_code + ") trx(" + m_doc_type + " / " + cutils::hash8c(m_doc_hash) + ") Block(" + cutils::hash8c(block_hash) + ")";
              CLog::log(msg, "sec", "error");
              return {false, msg, 0, 0};
            }
          }
        }
        }

        if (m_outputs.len() > 0)
        {
        for (TOutput* output: m_outputs)
        {
          if (output.m_address != cutils::stripOutputAddress(output.m_address))
          {
            msg = "The transaction has not digit charecters in bech 32 address(" + output.m_address + ")! trx(" + m_doc_type + " / " + cutils::hash8c(m_doc_hash) + ") Block(" + cutils::hash8c(block_hash) + ")";
            CLog::log(msg, "sec", "error");
            return {false, msg, 0, 0};
          }

          if (output.m_amount == 0)
          {
            msg = "The transaction has zero output! trx(" + m_doc_type + " / " + cutils::hash8c(m_doc_hash) + ") Block(" + cutils::hash8c(block_hash) + ")";
            CLog::log(msg, "sec", "error");
            return {false, msg, 0, 0};
          }

          if (output.m_amount < 0)
          {
            msg = "The transaction has negative output! trx(" + m_doc_type + " / " + cutils::hash8c(m_doc_hash) + ") Block(" + cutils::hash8c(block_hash) + ")";
            CLog::log(msg, "sec", "error");
            return {false, msg, 0, 0};
          }

          if (output.m_amount >= MAX_COIN_VALUE)
          {
            msg = "The transaction has output bigger than MAX_SAFE_INTEGER! trx(" + m_doc_type + " / " + cutils::hash8c(m_doc_hash) + ") Block(" + cutils::hash8c(block_hash) + ")";
            CLog::log(msg, "sec", "error");
            return {false, msg, 0, 0};
          }

          total_outputs_amounts += output.m_amount;
        }
        }

        return {true, msg, total_inputs_amounts, total_outputs_amounts};
        }

        GenRes BasicTxDocument::customValidateDoc(const Block* block) const
        {
        String msg;
        if (m_doc_class == constants::TRX_CLASSES::P4P)
        {
        if (!constants::SUPPORTS_P4P_TRANSACTION)
        {
          msg = "Network! still doen't support P4P transactions!";
          CLog::log(msg, "trx", "error");
          return {false, msg};
        }
        }

        for (auto an_output: m_outputs)
        {
        if (an_output.m_amount == 0)
        {
          msg = "creating block, the transaction has zero output! trx(" + cutils::hash8c(get_doc_hash()) + ")  ";
          CLog::log(msg, "trx", "error");
          return {false, msg};
        }

        if (an_output.m_amount < 0)
        {
          msg = "creating block, the transaction has negative output! trx(" + cutils::hash8c(get_doc_hash()) + ")  ";
          CLog::log(msg, "trx", "error");
          return {false, msg};
        }

        }

        return {true, ""};
        }

        CMPAISValueT BasicTxDocument::getDocCosts() const
        {
        CMPAISValueT costs = 0;
        for (auto a_cost_index: get_dpis())
        costs += m_outputs[a_cost_index].m_amount;
        return costs;
        }

        std::tuple<JSonArray, JSonArray> BasicTxDocument::extractInputOutputs(
        JSonArray inputs,
        JSonArray outputs,
        String sig_hash)
        {
        if (sig_hash == constants::SIGHASH::ALL) {
        // change nothing
        } else if (sig_hash == constants::SIGHASH::NONE) {
        outputs = {};
        }
        return { inputs, outputs };

        ////TODO: implement CUSTOM
        //case iConsts.SIGHASH.CUSTOM:
        //    let cust = {
        //        inputs: ['fdde3ddwdwedewwdded3:2', 'fdde3ddwdwedewwd656:10', 'fdde3d6yydyyewwdded3:4'],
        //        outputs: ['ALL']
        //    }
        //    cust = {
        //        inputs: 'ALL',
        //        outputs: ['fdde3ddwdwedewwdded3:2', 'fdde3ddwdwedewwd656:10', 'fdde3d6yydyyewwdded3:4']
        //    }
        //    cust = {
        //        inputs: ['fdde3ddwdwedewwdded3:2', 'fdde3ddwdwedewwd656:10', 'fdde3d6yydyyewwdded3:4'],
        //        outputs: ['fdde3ddwdwedewwdded3:2', 'fdde3ddwdwedewwd656:10', 'fdde3d6yydyyewwdded3:4']
        //    }
        //    inputs = args.inputs;
        //    outputs = args.outputs;
        //    break;
        // since they have conflict with BIP69 all are disabled
        // instead, use custom
        // case iConsts.SIGHASH['SINGLE']:
        //     inputs = args.inputs;
        //     outputs = [args.outputs[args.selectedIndex]];
        //     break;
        // case iConsts.SIGHASH['ALL|ANYONECANPAY']:
        //     inputs = [args.inputs[args.selectedIndex]];
        //     outputs = args.outputs;
        //     break;
        // case iConsts.SIGHASH['NONE|ANYONECANPAY']:
        //     inputs = [args.inputs[args.selectedIndex]];
        //     outputs = [];
        //     break;
        // case iConsts.SIGHASH['SINGLE|ANYONECANPAY']:
        //     inputs = [args.inputs[args.selectedIndex]];
        //     outputs = [args.outputs[args.selectedIndex]];
        //     break;

        }



        */
}