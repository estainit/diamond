use std::collections::HashMap;
use crate::{constants, cutils, dlog};
use crate::cutils::controlled_str_to_json;
use crate::lib::custom_types::JSonObject;
use crate::lib::block::block_types::block::Block;
use crate::lib::block_utils::unwrap_safed_content_for_db;

impl Block {
    pub fn load_block(obj: &JSonObject) -> (bool, Block)
    {
        let mut block: Block = Self::new();
        let status = block.set_block_by_json_obj(obj);
        if !status {
            println!("Failed in set block by JSON obj: {}", cutils::controlled_json_stringify(&obj));

            dlog(
                &format!("Failed in set block by JSON obj: {}", cutils::controlled_json_stringify(&obj)),
                constants::Modules::CB,
                constants::SecLevel::Error);
        }
        return (status, block);
    }

    pub fn load_block_by_db_record(record_row: &HashMap<String, String>) -> (bool, Block)
    {
        let (status, _sf_ver, serialized_block_body) = unwrap_safed_content_for_db(&record_row["b_body"].to_string());
        if !status
        {
            dlog(
                &format!(
                    "Failed in unwrap safe content: {}",
                    &record_row["b_body"].to_string()),
                constants::Modules::App,
                constants::SecLevel::Error);
            return (false, Self::new());
        }
        return Self::load_block_by_serialized_content(&serialized_block_body);
    }

    pub fn load_block_by_serialized_content(serialized_block_body: &String) -> (bool, Block)
    {
        let (status, j_obj) = controlled_str_to_json(&serialized_block_body);
        if !status
        {
            dlog(
                &format!(
                    "Failed in deser unwrapped safe content: {}",
                    &serialized_block_body),
                constants::Modules::App,
                constants::SecLevel::Error);
            return (false, Self::new());
        }

        return Self::load_block(&j_obj);
    }

    #[allow(dead_code)]
    pub fn from_str(content: &String) -> (bool, Block)
    {
        let mut status = true;
        let block: Block = match serde_json::from_str(&content)
        {
            Ok(r) => r,
            Err(e) => {
                status = false;
                dlog(
                    &format!(
                        "Failed in deserializing block {} {}",
                        e, content
                    ),
                    constants::Modules::App,
                    constants::SecLevel::Error);
                Self::new()
            }
        };
        (status, block)
    }
}

/*


Block* BlockFactory::create(const JSonObject &obj)
{
  String block_type = obj.value("bType").to_string();
  if (block_type == constants::BLOCK_TYPES::Normal)
  {
    return new NormalBlock(obj);

  }
  else if (block_type == constants::block_types::COINBASE)
  {
    Block *b{new CoinbaseBlock(obj)};
    return b;

  }
  else if (block_type == constants::BLOCK_TYPES::RpBlock)
  {
    Block *b{new RepaybackBlock(obj)};
    return b;

  }
  else if (block_type == constants::BLOCK_TYPES::FSign)
  {
    Block *b{new FloatingSignatureBlock(obj)};
    return b;

  }
  else if (block_type == constants::BLOCK_TYPES::FVote)
  {
    Block *b{new FloatingVoteBlock(obj)};
    return b;

  }
  else if (block_type == constants::BLOCK_TYPES::POW)
  {
    Block *b{new POWBlock(obj)};
    return b;

  }
  else if (block_type == constants::BLOCK_TYPES::Genesis)
  {
    return new GenesisBlock(obj);
  }

  return new Block(obj);
}

*/