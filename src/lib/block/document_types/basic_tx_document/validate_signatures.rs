use std::collections::HashMap;
use substring::Substring;
use crate::{ccrypto, constants, cutils, dlog};
use crate::lib::block::block_types::block::Block;
use crate::lib::block::document_types::basic_tx_document::basic_tx_document::BasicTxDocument;
use crate::lib::block::document_types::document::Document;
use crate::lib::block::document_types::document_ext_info::DocExtInfo;
use crate::lib::custom_types::{CBlockHashT, CCoinCodeT, CDateT, CInputIndexT, CSigIndexT, QV2DicT, QVDicT, VString};
use crate::lib::transactions::basic_transactions::coins::coins_handler::extract_coins_owner;
use crate::lib::transactions::basic_transactions::signature_structure_handler::general_structure::validate_sig_struct;
use crate::lib::transactions::basic_transactions::signature_structure_handler::unlock_set::UnlockSet;


impl BasicTxDocument {
    // old name was veridfyDocSignature
    pub fn verify_doc_signature(
        &self,
        doc: &Document,
        block: &Block,
    ) -> bool
    {
        let mut coins_codes: Vec<CCoinCodeT> = vec![];
        for an_inp in &self.m_inputs
        {
            coins_codes.push(an_inp.get_coin_code());
        }

        // retrieve coins owners from blockchain
        let coins_owner_dict =
            extract_coins_owner(&coins_codes);


        let mut used_coins_dict: QV2DicT = HashMap::new();
        for an_inp in &self.m_inputs
        {
            let the_coin_owner = coins_owner_dict[&an_inp.get_coin_code()].clone();
            // retrieve coins owners from blockchain
            if an_inp.m_owner != "".to_string()
            {
                if an_inp.m_owner != the_coin_owner
                {
                    dlog(
                        &format!(
                            "Try to spend someone else coin! {}, pretended owner: {}, real owner: {}, m_inputs: {:#?}",
                            doc.get_doc_identifier(),
                            an_inp.m_owner,
                            the_coin_owner,
                            self.m_inputs),
                        constants::Modules::Sec,
                        constants::SecLevel::Fatal);
                    return false;
                }
            }

            let vv: QVDicT = HashMap::from([
                ("coin_owner".to_string(), the_coin_owner),
                ("coin_value".to_string(), an_inp.m_amount.to_string())]);
            used_coins_dict.insert(an_inp.get_coin_code(), vv);
        }

        return self.validate_tx_signatures(
            doc,
            &used_coins_dict,
            &vec![],
            &block.get_block_hash(),
        );
    }

    // js name was trxSignatureValidator
    // old name was validateSignatures
    pub fn validate_tx_signatures(
        &self,
        doc: &Document,
        used_coins_dict: &QV2DicT,
        exclude_coins: &VString,
        block_hash: &CBlockHashT) -> bool
    {
        let msg: String;

        // for each input must control if given unlock structutr will be finished in a right(and valid) output address?
        // the order of inputs and ext Info ARE IMPORTANT. the wallet MUST sign and send inputs in order to bip 69
        dlog(
            &format!(
                "Signature validating for {}, used_coins_dict: {:#?}",
                doc.get_doc_identifier(),
                used_coins_dict),
            constants::Modules::Trx,
            constants::SecLevel::TmpDebug);
        // panic!("yyyyyy");

        let mut the_coins_must_be_signed_by_a_single_sign_set:
            HashMap<CInputIndexT, HashMap<CSigIndexT, VString>> = HashMap::new();
        let mut vvv: VString;
        let mut input_index: CInputIndexT = 0;
        while input_index < self.m_inputs.len() as CInputIndexT
        {
            // for each input must find proper block and bech32 address of output and insert into validate function
            let coin_code: CCoinCodeT = self.m_inputs[input_index as usize].get_coin_code();
            // scape validating invalid inputs(in this case double-spended inputs)
            if exclude_coins.contains(&coin_code)
            { continue; }

            let an_ext_info: &DocExtInfo = &doc.m_doc_ext_info[input_index as usize];
            let an_unlock_set: &UnlockSet = &an_ext_info.m_unlock_set;

            let is_valid_unlocker: bool = validate_sig_struct(
                an_unlock_set,
                &used_coins_dict[&coin_code]["coin_owner"],
                &HashMap::new(),
            );

            if !is_valid_unlocker
            {
                msg = format!(
                    "Invalid block, because of invalid unlock structure! Block({}) transaction({}) input-index({:?}) unlock structure: {:#?}",
                    cutils::hash8c(block_hash),
                    doc.get_doc_identifier(),
                    input_index,
                    an_unlock_set);
                dlog(
                    &msg,
                    constants::Modules::Trx,
                    constants::SecLevel::Error);
                return false;
            }

            // prepare a signature dictionary
            // the_coins_must_be_signed_by_a_single_sign_set[input_index] = {};
            let mut ggg = HashMap::new();
            let sign_sets = &an_unlock_set.m_signature_sets;
            let mut signature_index: CSigIndexT = 0;
            while signature_index < sign_sets.len() as CSigIndexT
            {
                let sig_info: &VString = &an_ext_info.m_signatures[signature_index as usize];
                let (inputs, _outputs) =
                    BasicTxDocument::extract_input_outputs_based_on_sig_hash(
                        &self.m_inputs,
                        &self.m_outputs,
                        &sig_info[1]);
                vvv = vec![];
                // the_coins_must_be_signed_by_a_single_sign_set[input_index][singature_index] = StringList {};
                for an_inp in inputs
                {
                    // the_coins_must_be_signed_by_a_single_sign_set[input_index][singature_index]
                    //     .push(an_inp.get_coin_code());

                    vvv.push(an_inp.get_coin_code());
                }
                ggg.insert(signature_index, vvv);
                // the_coins_must_be_signed_by_a_single_sign_set[input_index].insert(singature_index, vvv);
                signature_index += 1;
            }
            the_coins_must_be_signed_by_a_single_sign_set.insert(input_index, ggg);

            input_index += 1;
        }

        let mut input_index = 0;
        while input_index < self.m_inputs.len() as CInputIndexT
        {
            // for each input must find proper block and bech32 address of output and insert into validate function
            let coin_code: CCoinCodeT = self.m_inputs[input_index as usize].get_coin_code();

            // scape validating invalid inputs(in this case double-spended inputs)
            if exclude_coins.contains(&coin_code)
            { continue; }

            let an_unlock_document: &DocExtInfo = &doc.m_doc_ext_info[input_index as usize];
            let an_unlock_set = &an_unlock_document.m_unlock_set;
            let sign_sets = &an_unlock_set.m_signature_sets;

            // for each input and proper spending way, must control if the signature is valid?
            let mut signature_index: CSigIndexT = 0;
            while signature_index < sign_sets.len() as CSigIndexT
            {
                let is_verified: bool = self.verify_input_outputs_signature(
                    doc,
                    &sign_sets[signature_index as usize]
                        .m_signature_key, //pubKey
                    &an_unlock_document
                        .m_signatures[signature_index as usize][0], //signature
                    &an_unlock_document
                        .m_signatures[signature_index as usize][1], //sig_hash
                    &doc.m_doc_creation_date);
                if !is_verified
                {
                    msg = format!(
                        "Invalid given signature for signature sig:{}, _index({}) input({}) Block({}) {}",
                        an_unlock_document.m_signatures[signature_index as usize][0],
                        signature_index,
                        input_index,
                        cutils::hash8c(block_hash),
                        doc.get_doc_identifier());
                    dlog(
                        &msg,
                        constants::Modules::Trx,
                        constants::SecLevel::Error);
                    return false;
                }

                if self.can_have_time_locked_inputs()
                {
                    //// control input timeLocks
                    //if (_.has(aSignSet, 'iTLock') && (aSignSet.iTLock > 0)) {
                    //  clog.trx.info(`>>>>>>>>>>>>> signed RefLocs By A Single SignSet ${utils.stringify(the_coins_must_be_signed_by_a_single_sign_set)}`);
                    //  let iTLock = iutils.convertBigIntToJSInt(aSignSet.iTLock);
                    //  for (aRefLoc of the_coins_must_be_signed_by_a_single_sign_set[input_index][singature_index])
                    //  {
                    //      let inputCreationDate = used_coins_dict["coin_code"].utRefCreationDate;
                    //      let inputRedeemDate = utils.minutesAfter(iTLock, inputCreationDate);

                    //      msg = `info::: inputTimeLock compareTime(${c_date}) block(${utils.hash6c(block.blockHash)}) trx(${utils.hash6c(trx.hash)}) `;
                    //      msg += `input(${input_index} locked for ${iTLock} Minutes after creation) created on(${inputCreationDate}) `;
                    //      msg += `has inputTimeLock and can be redeemed after(${inputRedeemDate}) `;
                    //      if (c_date < inputRedeemDate) {
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
                signature_index += 1;
            }

            input_index += 1;
        }

        dlog(
            &format!("All trx have valid signatures Block({}) ", cutils::hash8c(block_hash)),
            constants::Modules::Trx,
            constants::SecLevel::TmpDebug);

        return true;
    }

    // old name was verifyInputOutputsSignature
    pub fn verify_input_outputs_signature(
        &self,
        doc: &Document,
        public_key: &String,
        signature: &String,
        sig_hash: &String,
        c_date: &CDateT,
    ) -> bool
    {
        if public_key == ""
        {
            dlog(
                &format!("Missed public_key!"),
                constants::Modules::App,
                constants::SecLevel::Error);
            return false;
        }

        if signature == ""
        {
            dlog(
                &format!("Missed signature!"),
                constants::Modules::App,
                constants::SecLevel::Error);
            return false;
        }

        if sig_hash == ""
        {
            dlog(
                &format!("Missed sig hash!"),
                constants::Modules::App,
                constants::SecLevel::Error);
            return false;
        }

        let sign_ables_clear: String = self.get_input_output_signables1(doc, sig_hash, c_date);
        let sign_ables_hash = ccrypto::keccak256_dbl(&sign_ables_clear); // because of security, MUST use double hash
        let verify_res: bool = ccrypto::ecdsa_verify_signature(
            public_key,
            &sign_ables_hash.substring(0, constants::SIGN_MSG_LENGTH as usize).to_string(),
            signature);

        dlog(
            &format!("YYY-Sign verify info: verify-res:{} sig-hash: {}, sign-ables-hash: {}, sign-ables-clear: {}, public-key:{}, signature:{}, the doc: {:#?}",
                     verify_res,
                     sig_hash,
                     sign_ables_hash,
                     sign_ables_clear,
                     public_key,
                     signature,
                     doc),
            constants::Modules::App,
            constants::SecLevel::TmpDebug);

        return verify_res;
    }
}
