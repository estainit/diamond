use crate::cutils::remove_quotes;
use crate::{constants, cutils, dlog};
use crate::lib::block::block_types::block::Block;
use crate::lib::block::block_types::block_genesis::genesis_block::b_genesis::genesis_set_by_json_obj;
use crate::lib::custom_types::{BlockLenT, JSonObject, VString};

impl Block {
    pub fn set_block_by_json_obj(&mut self, obj: &JSonObject) -> bool
    {
        if !obj["local_receive_date"].is_null() {
            self.m_block_receive_date = remove_quotes(&obj["local_receive_date"]);
        }

        if !obj["bNet"].is_null() {
            self.m_block_net = remove_quotes(&obj["bNet"]);
        }

        if !obj["bVer"].is_null() {
            self.m_block_version = remove_quotes(&obj["bVer"]);
        }

        if !obj["bType"].is_null() {
            self.m_block_type = remove_quotes(&obj["bType"]);
        }

        if !obj["bDescriptions"].is_null() {
            self.m_block_descriptions = remove_quotes(&obj["bDescriptions"]);
        }

        // if obj["bConfidence"].to_string() != "" {
        //     println!("iiiiiiiiiiii {}", obj["bConfidence"]);
        //     self.m_block_confidence = remove_quotes(&obj["bConfidence"].to_string().parse4>().unwrap());
        // }

        if !obj["bLen"].is_null() {
            // let b_len = obj["bLen"].to_string().parse::<BlockLenT>();
            let (status, b_len) =
                match remove_quotes(&obj["bLen"])
                    .parse::<BlockLenT>() {
                    Ok(l) => { (true, l) }
                    Err(e) => {
                        dlog(
                            &format!("Invalid bLen! {:?} in received JSon Obj {:?}",
                                     obj["bLen"], e),
                            constants::Modules::App,
                            constants::SecLevel::Error);
                        (false, 0)
                    }
                };
            if !status {
                return false;
            }
            self.m_block_length = b_len;
        }

        if !obj["bHash"].is_null() {
            self.m_block_hash = remove_quotes(&obj["bHash"]);
        }

        if !obj["bAncestors"].is_null() && obj["bAncestors"].is_array()
        {
            let ancestors: &Vec<JSonObject> = &vec![];
            let (status, ancestors) = match obj["bAncestors"].as_array() {
                Some(r) => (true, r),
                _ => {
                    dlog(
                        &format!("Invalid bAncestors {} in  {}",
                                 obj["bAncestors"], self.get_block_identifier()),
                        constants::Modules::App,
                        constants::SecLevel::Error);
                    (false, ancestors)
                }
            };
            if !status
            {
                return false;
            }
            self.m_block_ancestors = ancestors
                .iter()
                .map(|x| remove_quotes(&x))
                .collect::<VString>();
        }

        // if !obj["signals"].toOis_null(len() > 0 {
        //     self.m_signals = remove_quotes(&obj["signals"].toObject());

        if !obj["bCDate"].is_null()
        {
            self.m_block_creation_date = remove_quotes(&obj["bCDate"]);
        }

        if !obj["bDocsRootHash"].is_null()
        {
            self.m_block_documents_root_hash = remove_quotes(&obj["bDocsRootHash"]);
        }

        if !obj["bDocs"].is_null() && obj["bDocs"].is_array()
        {
            let j_documents: &Vec<JSonObject> = &vec![];
            let (status, j_documents) = match obj["bDocs"].as_array() {
                Some(r) => (true, r),
                _ => {
                    dlog(
                        &format!(
                            "Invalid bDocs {} in received JSon Obj {}",
                            self.m_block_type,
                            cutils::controlled_json_stringify(&obj)),
                        constants::Modules::App,
                        constants::SecLevel::Error);
                    (false, j_documents)
                }
            };
            if !status
            { return false; }

            let status = self.create_block_documents(j_documents);
            if !status
            { return false; }
            // self.m_block_documents = remove_quotes(&obj["bDocs"]);
            // if (object_keys.contains("docs"))
            // createDocuments(obj.value("docs"));
        }

        if !obj["bExtHash"].is_null() {
            self.m_block_ext_root_hash = remove_quotes(&obj["bExtHash"]);
        }

        if !obj["bExtInfo"].is_null() {
            // self.m_block_ext_info = remove_quotes(&obj["bExtInfo"].to_);if !obj["bDocs"].is_null() {
            // createDocuments(obj["bDocs"]);
        }

        if !obj["bBacker"].is_null() {
            self.m_block_backer = remove_quotes(&obj["bBacker"]);
        }

        if !obj["bFVotes"].is_null() {
            // self.m_floating_votes = obj["bFVotes"].toArray();
        }


        if self.m_block_type == constants::block_types::NORMAL {
            return true;
        } else if self.m_block_type == constants::block_types::COINBASE {
            return self.m_if_coinbase_block.set_block_by_json_obj(obj);
        } else if self.m_block_type == constants::block_types::REPAYMENT_BLOCK
        {} else if self.m_block_type == constants::block_types::FLOATING_SIGNATURE
        {} else if self.m_block_type == constants::block_types::FLOATING_VOTE
        {} else if self.m_block_type == constants::block_types::POW
        {} else if self.m_block_type == constants::block_types::GENESIS
        {
            return genesis_set_by_json_obj(self, obj);
        }

        println!("Invalid block type1 {:?} in received JSon Obj {:?}",
                 self.m_block_type, cutils::controlled_json_stringify(&obj));
        println!("Invalid block type2 {} in received JSon Obj {}",
                 self.m_block_type, cutils::controlled_json_stringify(&obj));
        dlog(
            &format!("Invalid block type {} in received JSon Obj {}",
                     self.m_block_type, cutils::controlled_json_stringify(&obj)),
            constants::Modules::App,
            constants::SecLevel::Error);
        return false;
    }
}