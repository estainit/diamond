use std::collections::HashMap;
use crate::{constants, cutils, dlog};
use crate::lib::block::document_types::basic_tx_document::basic_tx_document::BasicTxDocument;
use crate::lib::block::document_types::document::Document;
use crate::lib::block::document_types::document_ext_info::DocExtInfo;
use crate::lib::custom_types::{CBlockHashT, CCoinCodeT, CInputIndexT, CSigIndexT, QV2DicT, VString};
use crate::lib::transactions::basic_transactions::signature_structure_handler::general_structure::validate_sig_struct;
use crate::lib::transactions::basic_transactions::signature_structure_handler::unlock_set::UnlockSet;


impl BasicTxDocument {
    // js name was trxSignatureValidator
// old name was validateSignatures
    pub fn validate_signatures(
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
            &format!("Signature validating for {}", doc.get_doc_identifier()),
            constants::Modules::Trx,
            constants::SecLevel::TmpDebug);

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
                &used_coins_dict[&coin_code]["ut_o_address"],
                &HashMap::new(),
            );

            if !is_valid_unlocker
            {
                msg = format!(
                    "Invalid block, because of invalid unlock struture! Block({}) transaction({}) input-index({:?}) unlock structure: {:#?}",
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
                        "Invalid given signature for input({:?}) Block({}) {}",
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
                    //      let inputCreationDate = used_coins_dict[aRefLoc].utRefCreationDate;
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
}
