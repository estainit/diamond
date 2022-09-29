use crate::lib::custom_types::VString;

//old_name_was controlCoinsVisibilityInGraphHistory
pub fn control_coins_visibility_in_graph_history(
 _block_used_coins:&VString,
 _ancestors:&VString,
 _block_hash:&String)->bool
{

//  /**
//  * since first level of ancestors(or maybe some other levels in between) blocks
//  * could be floating signatures for which in trx_utxos there is no entry,
//  * so looking for ancestors of ancestors too(4 level up)
//  */
//  ancestors = utils.arrayUnique(utils.arrayAdd(ancestors, AddBlockHandler._super.walkThrough.getAncestors(ancestors)));
//  ancestors = utils.arrayUnique(utils.arrayAdd(ancestors, AddBlockHandler._super.walkThrough.getAncestors(ancestors)));
//  ancestors = utils.arrayUnique(utils.arrayAdd(ancestors, AddBlockHandler._super.walkThrough.getAncestors(ancestors)));
//  ancestors = utils.arrayUnique(utils.arrayAdd(ancestors, AddBlockHandler._super.walkThrough.getAncestors(ancestors)));
//  let visibleCoins = [];
//  clog.trx.info(`the reflocs(${block_used_coins.map(x => iutils.shortCoinRef(x))}) must be visible by at least one of: ${ancestors.map(x => utils.hash6c(x))}`);
//  for (let ancestor of ancestors) {
//   let tmpVisibleCoins = utxoHandler.searchInSpendableCoins({
//       coins: block_used_coins,
//       visibleBy: ancestor
//   })
//   if (tmpVisibleCoins.length > 0)
//       visibleCoins = utils.arrayAdd(visibleCoins, tmpVisibleCoins.map(x => x.ut_coin));
//  };
//  visibleCoins = utils.arrayUnique(visibleCoins);
//  clog.trx.info(`Existed visible coins: ${utils.stringify(visibleCoins)}`);

//  /**
//  * invisible Coins means the coins which are not visible by this block,
//  * could be because they are not in block hirarchy history or because they are already spended
//  */
//  let invisibleCoins = utils.arrayDiff(block_used_coins, visibleCoins);
//  clog.trx.info(`invisible Coins: ${utils.stringify(invisibleCoins)}`);

//  // controll if they are in DAG but not in block's history
//  if (invisibleCoins.length > 0) {
//   let otherHistory = []
//   for(let coin of invisibleCoins){
//       if (_.has(usedCoinsDict, coin))
//           otherHistory.push(usedCoinsDict[coin]);
//   };
//   if (otherHistory.length > 0) {
//       msg = `block(${utils.hash6c(block_hash)}) can not spend refLocs(${utils.stringify(otherHistory)}) which are not in it's history`;
//       clog.trx.warn(msg)
//       return { err: true, msg, shouldPurgeMessage: true }
//   }

//  }

return true;
}
