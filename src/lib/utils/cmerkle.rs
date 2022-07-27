use std::collections::HashMap;
use substring::Substring;

// use crate::lib::constants as cconsts;
use crate::lib::custom_types::{VString, VVString};
use crate::lib::utils::cutils as cutils;
use crate::lib::ccrypto;
use crate::lib::constants as cconsts;
use crate::lib::dlog::dlog;
use crate::lib::utils::dumper;

pub const MERKLE_VERSION: &str = "0.1.0";

#[derive(Clone)]
pub struct MerkleNodeData {
    pub m_proof_keys: VString,
    pub m_parent: String,
    pub m_left_hash: String,
    pub m_merkle_proof: VString,
//  uint16_t leaves;
}

pub trait MerkleNodeDataTrait {
    fn new() -> MerkleNodeData;
    fn mynew() -> MerkleNodeData;
    fn clone_me(&self) -> MerkleNodeData;
    fn set_m_merkle_proof(&mut self, proof: VString);
    fn push_m_merkle_proof(&mut self, proof: &String);
}

impl MerkleNodeDataTrait for MerkleNodeData {
     fn new() -> MerkleNodeData {
        let o: MerkleNodeData = MerkleNodeData {
            m_proof_keys: vec![],
            m_parent: "".to_string(),
            m_left_hash: "".to_string(),
            m_merkle_proof: vec![],
        };
        return o;
    }

     fn mynew() -> MerkleNodeData {
        let o: MerkleNodeData = MerkleNodeData {
            m_proof_keys: vec![],
            m_parent: "".to_string(),
            m_left_hash: "".to_string(),
            m_merkle_proof: vec![],
        };
        return o;
    }

    fn clone_me(&self) -> MerkleNodeData {
        let mut o: MerkleNodeData = MerkleNodeData::new();
        o.m_proof_keys = cutils::clone_vec(&self.m_proof_keys);
        o.m_parent = self.m_parent.clone();
        o.m_left_hash = self.m_left_hash.clone();
        o.m_merkle_proof = cutils::clone_vec(&self.m_merkle_proof);
        return o;
    }

    fn set_m_merkle_proof(&mut self, proof: VString) {
        self.m_merkle_proof = proof;
    }

    fn push_m_merkle_proof(&mut self, proof: &String) {
        self.m_merkle_proof.push(proof.clone());
    }
}

pub type MNodesMapT = HashMap<String, MerkleNodeData>;

pub fn generate(
    elms: &VString,
    input_type: &String,
    hash_algorithm: &String,
    version_: &String) -> (String, MNodesMapT, String, i8, i8)
{
    let mut version = version_.to_string();
    if version == "".to_string() {
        version = MERKLE_VERSION.to_string();
    }

    let p: MNodesMapT = HashMap::new();

    if elms.len() == 0 {
        return (
            "".to_string(), //root
            p, //proofs
            version.to_string(),
            0,  //levels
            0  //leaves
        );
    }

    if elms.len() == 1 {
        let root_hash: String;
        if input_type == "hashed" {
            root_hash = elms[0].to_string()
        } else {
            root_hash = do_hash_a_node(&elms[0].to_string(), hash_algorithm);
        }
        return (
            root_hash,
            p, // proofs
            version,
            1,  //levels
            1  // leaves
        );
    }

    // fitting leaves conut
    let mut needed_leaves: i32 = 2;
    let mut inx = elms.len();
    needed_leaves = match inx {
        1..=2 => 2,
        3..=4 => 4,
        5..=8 => 8,
        9..=16 => 16,
        17..=32 => 32,
        33..=64 => 64,
        65..=128 => 128,
        129..=256 => 256,
        257..=512 => 512,
        513..=1024 => 1024,
        1025..=2048 => 2048,
        2049..=4096 => 4096,
        4097..=8192 => 8192,
        _ => panic!("Invalid needed_leaves: {}", needed_leaves)
    };
    let mut elms_ = cutils::clone_vec(&elms);
    while inx < needed_leaves as usize {
        elms_.push("leave_".to_owned() + &format!("{}", inx + 1));
        inx += 1;
    }

    let (root,
        verifies,
        leaves,
        levels) =
        inner_merkle(&elms_, input_type, hash_algorithm, &"".to_string());

    let mut final_verifies: MNodesMapT = HashMap::new();
    let mut keys: Vec<&String> = verifies.keys().collect();

    keys.sort();  // FIXME: the true order is reverse sort, also consider the version 0.1.0 in order to solve "fake right leave" problem
    for key in keys {
        let a_proof: MerkleNodeData = verifies.get(&key.clone()).unwrap().clone_me();
        let the_key = key.clone()[2..].to_string();
        if !final_verifies.contains_key(&the_key) {
            final_verifies.insert(the_key.clone(), a_proof);
        }
        let mut the_element = final_verifies.get(&the_key).unwrap().clone_me();
        // the_element.m_merkle_proof = cutils::clone_vec(&verifies.get(&key.clone()).unwrap().m_proof_keys);
        // the_element.m_left_hash = verifies.get(&key.clone()).unwrap().m_left_hash.clone();
    }
    return (
        root,
        final_verifies,
        version,
        levels as i8,
        leaves as i8
    );
}


//old_name_was doHashANode
pub fn do_hash_a_node(node_value: &String, hash_algorithm: &String) -> String {
    if (hash_algorithm == "keccak256") | (hash_algorithm == "") {
        return ccrypto::keccak256(node_value);
    } else if hash_algorithm == "noHash" {
        return node_value.to_string();
    } else if hash_algorithm == "aliasHash" {
        return "h(".to_owned() + node_value + ")";
    } else {
        let err_msg = format!("Invalid hash algorithm for merkle!: {}", hash_algorithm);
        dlog(&err_msg, cconsts::Modules::App, cconsts::SecLevel::Fatal);
        return err_msg.to_string();
    }
}

//old_name_was innerMerkle
pub fn inner_merkle(elms_: &VString, input_type: &String, hash_algorithm: &String, _version: &String)
                    -> (String, MNodesMapT, i32, i32)
{
    let mut elms = cutils::clone_vec(elms_);
    if input_type == "string" {
        let mut hashed_elements: VString = vec![];
        for element in elms {
            hashed_elements.push(do_hash_a_node(&element, hash_algorithm));
        }
        elms = cutils::clone_vec(&hashed_elements);
    }

    let mut verifies: MNodesMapT = HashMap::new();
    let leaves = elms.len();
    let mut level: i32 = 0;
    let mut parent: String;
    let l_key: String;
    let r_key: String;
    let l_child: String;
    let r_child: String;
    while level < 100_000 {
        level += 1;

        if elms.len() == 1 {
            return
                (
                    elms[0].to_string(), //root:
                    verifies,
                    leaves as i32,
                    level
                );
        }

        if elms.len() % 2 == 1 {
            let err_msg = format!("FATAL ERROR ON MERKLE GENERATING: {}", dumper::dump_it(elms));
            dlog(&err_msg, cconsts::Modules::App, cconsts::SecLevel::Fatal);
            panic!("{}", err_msg);
            // // adding parity right fake hash
            // QString the_hash = elms[elms.len() - 1];
            // if (version > "0.0.0")
            //     the_hash = CConsts::FAKE_RIGHT_HASH_PREFIX + the_hash;
            // elms.push_back(the_hash);
        }

        let mut chunks: VVString = cutils::chunk_to_vvstring(&elms, 2);
        elms = vec![]; // emptying elements

        for chunk in chunks {
            parent = do_hash_a_node(&(chunk[0].clone().to_string() + &chunk[1].to_string()), hash_algorithm);
            elms.push(parent.clone());
            if level == 1 {
                let l_key = format!("l_{}", chunk[0].clone());
                let r_key = "r_".to_owned() + &chunk[1].to_string();
                let val = MerkleNodeData {
                    m_proof_keys: vec![],
                    m_parent: parent.clone(),
                    m_left_hash: "".to_string(),
                    m_merkle_proof: vec![],
                };
                verifies.insert(l_key.clone(), val.clone_me());
                let val = MerkleNodeData {
                    m_proof_keys: vec![],
                    m_parent: parent.clone(),
                    m_left_hash: chunk[0].clone().to_string(),
                    m_merkle_proof: vec![],
                };
                verifies.insert(r_key.clone(), val.clone_me());

                let mut veri = verifies.get(&l_key.clone()).unwrap().clone_me();
                veri.push_m_merkle_proof(&("r.".to_owned() + &chunk[1].to_string()));
                verifies.remove(&l_key);
                verifies.insert(l_key.clone(), veri);
            } else {
                // find alter parent cild
                let mut tmp_verifies: MNodesMapT = HashMap::new();

                for (key, the_verify) in verifies.iter() {
                    let mut mut_verify: MerkleNodeData = the_verify.clone_me();
                    if chunk[0] == the_verify.m_parent {
                        mut_verify.push_m_merkle_proof(&("r.".to_owned() + &chunk[1]));
                        mut_verify.m_parent = parent.clone();
                    }
                    if chunk[1] == the_verify.m_parent {
                        mut_verify.push_m_merkle_proof(&("l.".to_owned() + &chunk[0]));
                        mut_verify.m_parent = parent.clone();
                    }
                    tmp_verifies.insert(key.clone(), mut_verify.clone_me());
                }
                // replace changes
                for (key, new_verify) in tmp_verifies.iter() {
                    verifies.remove(key);
                    verifies.insert(key.clone(), new_verify.clone_me());
                }
            }
        }
    }

    return (
        "".to_string(), //root:
        HashMap::new(),
        0,
        0
    );
}

//old_name_was getRootByAProve
pub fn get_root_by_a_prove(
    leave_: &String,
    proof_hashes: &VString,
    l_hash: &String,
    input_type_: &String,
    hash_algorithm_: &String) -> String
{
    let mut input_type: String = input_type_.to_string();
    if input_type == "" {
        input_type = "hashed".to_string();
    }
    let mut hash_algorithm: String = hash_algorithm_.to_string();
    if hash_algorithm == "" {
        hash_algorithm = "keccak256".to_string();
    }
    let mut leave: String = leave_.to_string();
    if input_type == "string" {
        leave = do_hash_a_node(&leave, &hash_algorithm);
    }

    let mut proof: String = "".to_string();
    if l_hash != "" {
        proof = do_hash_a_node(&(l_hash.to_string() + &leave), &hash_algorithm);
    } else {
        proof = leave;
    }

    if proof_hashes.len() > 0 {
        for element in proof_hashes {
            let pos: String = element.substring(0, 1).to_string();
            let val: String = element.substring(2, element.len()).to_string();
            if pos == "r" {
                proof = do_hash_a_node(&(proof + &val), &hash_algorithm);
            } else {
                proof = do_hash_a_node(&(val + &proof), &hash_algorithm);
            }
        }
    }
    return proof;
}

/*
std::tuple<QString, MNodesMapT, QString, int, int> generate(QStringList elms, QString inputType = "hashed", QString hash_algorithm = "keccak256", QString version = MERKLE_VERSION);

QString do_hash_a_node(const QString & node_value, const QString & hash_algorithm = "keccak256");

std::tuple<QString, MNodesMapT, int, int> innerMerkle(
QStringList elms,
QString inputType = "hashed",
QString hashAlgorithm = "keccak256",
QString version = MERKLE_VERSION);



QString dumpAProof(const MerkleNodeData & a_proof);
QString dumpProofs(const MNodesMapT & proofs);

};



QString dumpAProof(const MerkleNodeData& a_proof)
{
  QString out = "";
  out += "\n Parent: " + a_proof.m_parent;
  out += "\n Proof Keys: " + a_proof.m_proof_keys.join(", ");
  out += "\n Left Hash: " + a_proof.m_left_hash;
  out += "\n Merkle Proof: " + a_proof.m_merkle_proof.join(", ");
  return out;
}

QString dumpProofs(const MNodesMapT& proofs)
{
  QString out = "";
  for (QString a_leave_hash: proofs.into_keys().collect())
  {
    out += "\n\n The Proof for leave hash: " + a_leave_hash;
    out += dumpAProof(proofs[a_leave_hash]);
  }
  return out;
}

//static merklePresenter(m) {
//    let out = `Root: ${m.root} \nLeaves: ${m.levels} \nLevels: ${m.leaves} \nProofs:`;

//    let inx = 0;
//    _.forOwn(m.proofs, function (value, key) {
//        out += `\n\tleave(${inx}): ${key} => ` + ((utils._notNil(value.m_left_hash)) ? `\n\t\tleftHsh: ${value.m_left_hash}` : ``)
//        out += `\n\t\thashes: `
//        value.m_merkle_proof.forEach(element => {
//            out += element + ` `
//        });
//        inx += 1;
//    });
//    return out;
//}







 */