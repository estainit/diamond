use crate::lib::block::block_types::block::Block;
use crate::lib::block::document_types::document::Document;
use crate::lib::custom_types::{CDocIndexT, JSonObject};

// pub fn create_new_document(
//     doc_type: String,
// ) -> Document
// {
//     let mut doc: Document = Document::new();
//     doc.m_doc_type = doc_type;
//
//     return doc;
// }

pub fn load_document(
    obj: &JSonObject,
    block: &Block,
    doc_index: CDocIndexT) -> Document
{
    let mut doc: Document = Document::new();
    let  _status = doc.set_by_json_obj(obj,block,doc_index);

    return doc;
}

