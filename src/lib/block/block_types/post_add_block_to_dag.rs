use crate::{constants, cutils};
use crate::lib::block::block_types::block::Block;
use crate::lib::custom_types::VString;
use crate::lib::dag::dag_walk_through::get_ancestors;
use crate::lib::dag::missed_blocks_handler::remove_from_missed_blocks;
use crate::lib::parsing_q_handler::queue_utils::remove_prerequisites;
use crate::lib::transactions::basic_transactions::coins::coins_handler::inherit_ancestors_visbility;

impl Block {
    //old name was postAddBlockToDAG
    pub fn post_add_block_to_dag(&self) -> bool
    {
        // remove prerequisite, if any block in parsing Q was needed to this block
        remove_prerequisites(&self.m_block_hash);

        // * sometimes (e.g. repayback blocks which can be created by delay and causing to add block to missed blocks)
        // * we need to doublecheck if the block still is in missed blocks list and remove it
        remove_from_missed_blocks(&self.get_block_hash());

        // * inherit coin visibilities of ancestors of newly DAG-added block
        // * current block inherits the visibility of it's ancestors
        // * possibly first level ancestors can be floating signatures(which haven't entry in table trx_utxos),
        // * so add ancestors of ancestors too, in order to being sure we keep good and reliable history in utxos
        if ![constants::block_types::FLOATING_SIGNATURE, constants::block_types::FLOATING_VOTE].contains(&self.m_block_type.as_str())
        {
            let mut ancestors: VString = self.m_block_ancestors.clone();
            ancestors = cutils::array_add(&ancestors, &get_ancestors(&ancestors, 1));
            ancestors = cutils::array_add(&ancestors, &get_ancestors(&ancestors, 1));
            ancestors = cutils::array_unique(&ancestors);
            inherit_ancestors_visbility(
                &ancestors,
                &self.m_block_creation_date,
                &self.get_block_hash());
        }

        return true;
    }
}