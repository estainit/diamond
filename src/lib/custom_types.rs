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
/*
typedef String  CDocHashT;

// customizing document index maximum number
pub type CInputIndexT = u16;

pub type CSigIndexT = u16;


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
pub type QHash<String, uint32_t>    UI32DicT; // custom dictionary
pub type QHash<String, uint64_t>    UI64DicT; // custom dictionary
*/
pub type QSDicT = HashMap<String, String>;
// pub type QUDicT = HashMap<String, QUnion>;
// custom dictionary
/*
pub type QHash<String, StringList> QSLDicT; // custom dictionary
pub type QHash<String, QSDicT>      QS2DicT; // custom dictionary
*/
// pub type QVariant = String    ; // FIXME: implement different QVariant (something like union)!
// custom dictionary
/*
pub type QHash<String, JSonObject> QJODicT; // custom dictionary
pub type QHash<String, JSonArray>  QJADicT; // custom dictionary
*/
pub type QVDicT = HashMap<String, String>;
pub type QV2DicT = HashMap<String, QVDicT>      ;
pub type QVDRecordsT = Vec<HashMap<String, String>>;
pub type JSonObject = Value;
pub type JSonArray = Value;
//Vec<QVDicT>;
/*
pub type QVector<QSDicT>        QSDRecordsT;
pub type QVector<QV2DicT>       QV2DRecordsT;
pub type QVector<JSonObject>   JORecordsT;
pub type QVector<JSonArray>    JARecordsT;
*/
pub type ClausesT<'l> = Vec<ModelClause<'l>>;
pub type OrderT<'l> = Vec<&'l OrderModifier<'l>>;
pub type LimitT = u32;

/*

pub type QHash<String, QVDRecordsT> GRecordsT; // Groupped records
pub type QHash<String, GRecordsT> G2RecordsT; // Groupped Groupped records

*/
/*
pub type QVector<Coin> CoinsT;
typedef String  CCoinCodeT;
typedef uint64_t CMPAIValueT;  // (+) micro PAI is the smallest unit of accounting for system coins, but normally we use PAI
typedef int64_t CMPAISValueT;  // (+-)micro PAI is the smallest unit of accounting for system coins, but normally we use PAI


typedef String  CDateT;
typedef uint32_t CDocIndexT;
typedef uint16_t CInputIndexT; // customizing document index maximum number
typedef uint16_t CSigIndexT;
*/
pub type COutputIndexT = u16;
// customizing document index maximum number
/*

typedef uint16_t DPIIndexT;
pub type DPIIndexT = u16;
*/
pub type SharesPercentT = f64;
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
typedef QHash<String, float>       floatDicT; // custom dictionary
typedef QHash<String, CDocIndexT>  UI16DicT; // custom dictionary
typedef QHash<String, uint32_t>    UI32DicT; // custom dictionary
typedef QHash<String, uint64_t>    UI64DicT; // custom dictionary
typedef QHash<String, String>     QSDicT; // custom dictionary
typedef QHash<String, StringList> QSLDicT; // custom dictionary
typedef QHash<String, QSDicT>      QS2DicT; // custom dictionary
typedef QHash<String, QVariant>    QVDicT; // custom dictionary
typedef QHash<String, JSonObject> QJODicT; // custom dictionary
typedef QHash<String, JSonArray>  QJADicT; // custom dictionary
typedef QHash<String, QVDicT>      QV2DicT;

typedef QVector<QVDicT>        QVDRecordsT;
typedef QVector<QSDicT>        QSDRecordsT;
typedef QVector<QV2DicT>       QV2DRecordsT;
typedef QVector<JSonObject>   JORecordsT;
typedef QVector<JSonArray>    JARecordsT;
typedef QVector<ModelClause>   ClausesT;
typedef QVector<OrderModifier> OrderT;

typedef QHash<String, QVDRecordsT> GRecordsT; // Groupped records
typedef QHash<String, GRecordsT> G2RecordsT; // Groupped Groupped records

typedef QHash<String, MerkleNodeData> MNodesMapT;

typedef QVector<Coin> CoinsT;

 */

#[allow(dead_code, unused)]
pub type DocDicT = HashMap<String, Document>;
#[allow(dead_code, unused)]
pub type DocDicVecT = HashMap<String, Vec<Document>>;
