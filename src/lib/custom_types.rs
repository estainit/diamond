use std::collections::HashMap;
use serde_json::Value;
use crate::lib::block::document_types::document::Document;
use crate::lib::database::abs_psql::{ModelClause, OrderModifier};

pub type CDateT = String;
pub type VString = Vec<String>;
pub type VVString = Vec<Vec<String>>;


#[allow(unused, dead_code)]
pub type CCoinCodeT = String;
pub type CMPAIValueT = u64;

/*
*/
// (+) micro PAI is the smallest unit of accounting for system coins, but normally we use PAI
#[allow(unused, dead_code)]
pub type CMPAISValueT = i64;
// (+-)micro PAI is the smallest unit of accounting for system coins, but normally we use PAI
pub type CBlockHashT = String;
pub type CDocHashT = String;
pub type CAddressT = String;
pub type CDocIndexT = i32;
pub type CInputIndexT = u16;
pub type CSigIndexT = u16;
/*
typedef String  CDocHashT;

// customizing document index maximum number



pub type BlockAncestorsCountT = u16;
// TODO: add max ancestor count control for received blocks
*/
pub type DocLenT = usize;
/*

pub type CVoteT = i16;  // between -100 0 100

// time by hours
 */
pub type TimeByMinutesT = u64;
pub type TimeBySecT = u64;

pub type DoubleDicT = HashMap<String, f64>;
// custom dictionary
/*
pub type floatDicT = HashMap<String, f64>       floatDicT; // custom dictionary
pub type UI16DicT = HashMap<String, CDocIndexT>  UI16DicT; // custom dictionary
pub type HashMap<String, uint32_t>    UI32DicT; // custom dictionary
pub type HashMap<String, uint64_t>    UI64DicT; // custom dictionary
*/
pub type QSDicT = HashMap<String, String>;
// pub type QUDicT = HashMap<String, QUnion>;
// custom dictionary
/*
pub type HashMap<String, VString> QSLDicT; // custom dictionary
pub type HashMap<String, QSDicT>      QS2DicT; // custom dictionary
*/
// pub type QVariant = String    ; // FIXME: implement different QVariant (something like union)!
// custom dictionary
/*
pub type HashMap<String, JSonObject> QJODicT; // custom dictionary
pub type HashMap<String, JSonArray>  QJADicT; // custom dictionary
*/
pub type QVDicT = HashMap<String, String>;
pub type QV2DicT = HashMap<String, QVDicT>;
pub type QVDRecordsT = Vec<HashMap<String, String>>;
pub type JSonObject = Value;
pub type JSonArray = Value;
//Vec<QVDicT>;
/*
pub type Vec<QSDicT>        QSDRecordsT;
pub type Vec<QV2DicT>       QV2DRecordsT;
pub type Vec<JSonObject>   JORecordsT;
pub type Vec<JSonArray>    JARecordsT;
*/
pub type ClausesT<'l> = Vec<ModelClause<'l>>;
pub type OrderT<'l> = Vec<&'l OrderModifier<'l>>;
pub type LimitT = u32;

// Grouped records
pub type GRecordsT = HashMap<String, QVDRecordsT>;
// Grouped Grouped records
#[allow(unused, dead_code)]
pub type G2RecordsT = HashMap<String, GRecordsT>;

/*
pub type Vec<Coin> CoinsT;
typedef String  CCoinCodeT;
typedef uint64_t CMPAIValueT;  // (+) micro PAI is the smallest unit of accounting for system coins, but normally we use PAI
typedef int64_t CMPAISValueT;  // (+-)micro PAI is the smallest unit of accounting for system coins, but normally we use PAI


typedef String  CDateT;
typedef uint32_t CDocIndexT;
typedef uint16_t CInputIndexT; // customizing document index maximum number
typedef uint16_t CSigIndexT;
*/
pub type COutputIndexT = i32;
// customizing document index maximum number
/*

typedef uint16_t DPIIndexT;
pub type DPIIndexT = u16;
*/
//old_name_was DNASharePercentT
pub type SharesPercentT = f64;
//old_name_was DNAShareCountT
pub type SharesCountT = f64;

#[allow(dead_code, unused)]
pub type BlockAncestorsCountT = u16;
// TODO: add max ancestor count control for received blocks
pub type BlockLenT = usize;

/*
typedef int16_t CVoteT;  // between -100 0 100
*/
pub type TimeByHoursT = f64;  // time by hours
/*
typedef uint64_t TimeByMinutesT;
typedef uint64_t TimeBySecT;
typedef HashMap<String, float>       floatDicT; // custom dictionary
typedef HashMap<String, CDocIndexT>  UI16DicT; // custom dictionary
typedef HashMap<String, uint32_t>    UI32DicT; // custom dictionary
typedef HashMap<String, uint64_t>    UI64DicT; // custom dictionary
typedef HashMap<String, String>     QSDicT; // custom dictionary
typedef HashMap<String, VString> QSLDicT; // custom dictionary
typedef HashMap<String, QSDicT>      QS2DicT; // custom dictionary
typedef HashMap<String, QVariant>    QVDicT; // custom dictionary
typedef HashMap<String, JSonObject> QJODicT; // custom dictionary
typedef HashMap<String, JSonArray>  QJADicT; // custom dictionary
typedef HashMap<String, QVDicT>      QV2DicT;

typedef Vec<QVDicT>        QVDRecordsT;
typedef Vec<QSDicT>        QSDRecordsT;
typedef Vec<QV2DicT>       QV2DRecordsT;
typedef Vec<JSonObject>   JORecordsT;
typedef Vec<JSonArray>    JARecordsT;
typedef Vec<ModelClause>   ClausesT;
typedef Vec<OrderModifier> OrderT;

typedef HashMap<String, QVDRecordsT> GRecordsT; // Groupped records
typedef HashMap<String, GRecordsT> G2RecordsT; // Groupped Groupped records

typedef HashMap<String, MerkleNodeData> MNodesMapT;

typedef Vec<Coin> CoinsT;

 */

#[allow(dead_code, unused)]
pub type DocDicT = HashMap<String, Document>;
#[allow(dead_code, unused)]
pub type DocDicVecT = HashMap<String, Vec<Document>>;
