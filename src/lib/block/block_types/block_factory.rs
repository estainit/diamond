use crate::{constants, dlog};
use crate::lib::custom_types::JSonObject;
use crate::lib::block::block_types::block::Block;

pub fn load_block(obj: &JSonObject) -> (bool, Block)
{
    let mut block: Block = Block::new();
    let status = block.setByJsonObj(obj);
    if !status{
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