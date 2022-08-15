use std::collections::HashMap;
use serde_json::json;
use crate::{constants, dlog};
use crate::lib::custom_types::JSonObject;
use crate::lib::block::block_types::block::Block;

pub fn load_block_by_db_record(record_row: &HashMap<String, String>) -> (bool, Block)
{
    println!("record_row oooooo: {:?}", record_row);
    let obj: JSonObject = json!({
        "local_receive_date": record_row["b_receive_date"],
        "bNet": constants::SOCIETY_NAME,
        "bType": record_row["b_type"],
        "bConfidence": record_row["b_confidence"],
        "bHash": record_row["b_hash"],
        "bAncestors": record_row["b_ancestors"],
        "signals": record_row["b_signals"],
        "bCDate": record_row["b_creation_date"],
        "bDocsRootHash": record_row["b_docs_root_hash"],
        "bExtHash": record_row["b_ext_root_hash"],
        "bBacker": record_row["b_backer"],
        // "bVer": record_row[""],
        // "bDescriptions": record_row[""],
        // "bLen": record_row[""],
        // "bExtInfo": record_row[""],
        // "bDocs": record_row[""],
        // "bCycle": record_row[""],
        // "bFVotes": record_row[""],
    });
    return load_block(&obj);
}

pub fn load_block(obj: &JSonObject) -> (bool, Block)

{
    let mut block: Block = Block::new();
    let status = block.setByJsonObj(obj);
    if !status {
        println!("Failed in set block by JSON obj: {}", serde_json::to_string(&obj).unwrap());

        dlog(
            &format!("Failed in set block by JSON obj: {}", serde_json::to_string(&obj).unwrap()),
            constants::Modules::CB,
            constants::SecLevel::Error);
    }
    return (status, block);
}


/*


Block* BlockFactory::create(const JSonObject &obj)
{
  String block_type = obj.value("bType").toString();
  if (block_type == CConsts::BLOCK_TYPES::Normal)
  {
    return new NormalBlock(obj);

  }
  else if (block_type == CConsts::BLOCK_TYPES::Coinbase)
  {
    Block *b{new CoinbaseBlock(obj)};
    return b;

  }
  else if (block_type == CConsts::BLOCK_TYPES::RpBlock)
  {
    Block *b{new RepaybackBlock(obj)};
    return b;

  }
  else if (block_type == CConsts::BLOCK_TYPES::FSign)
  {
    Block *b{new FloatingSignatureBlock(obj)};
    return b;

  }
  else if (block_type == CConsts::BLOCK_TYPES::FVote)
  {
    Block *b{new FloatingVoteBlock(obj)};
    return b;

  }
  else if (block_type == CConsts::BLOCK_TYPES::POW)
  {
    Block *b{new POWBlock(obj)};
    return b;

  }
  else if (block_type == CConsts::BLOCK_TYPES::Genesis)
  {
    return new GenesisBlock(obj);
  }

  return new Block(obj);
}

*/