use crate::{constants, dlog};

//old_name_was insertNewBlockControlls
pub fn controls_of_new_block_insertion(block_hash:&String) ->(bool,String)
{
  // FIXME: implement it ASAP
  dlog(
    &format!("ScepticalDAGIntegrityControl::controls_of_new_block_insertion({})", block_hash),
    constants::Modules::App,
    constants::SecLevel::Trace);


  // a series of checking to ensure if app succide to import all documents placed in block properly in database?

  // is block hash valid?
  // retrieve block from DAG
//  let regBlock = dagHandler.regenerateBlock(blockHash);
//  if (regBlock.err != false) {
//     msg = `sDIC block(${utils.hash6c(blockHash)}) regenerating failed ${regBlock.msg}`;
//     clog.app.error(msg);
//     return { err: true, msg }
//  }

//  let block = regBlock.block;

//  if (block.blockHash != blockHash) {
//     msg = `sDIC block(${utils.hash6c(blockHash)}) has invalid hash(${utils.hash6c(block.blockHash)})`;
//     clog.app.error(msg);
//     return { err: true, msg }
//  }




  // validate transactions recording
  // validate proposals recording(if exist)
  // validate pledges recording(if exist)
  // validate Ballots recording(if exist)
  // validate ...


  return (true, "".to_string());
}

