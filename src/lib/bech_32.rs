use bech32::{self, ToBase32, Variant}; // FromBase32
// use std::ops::{Bound, RangeBounds};
use substring::Substring;

use crate::lib::constants as CConsts;

//old_name_was ComenIsValid
pub fn comen_is_valid(inp_str: &str) -> bool {
    match bech32::decode(inp_str) {
        Ok(res) => {
            let (hrp, data, variant) = res;
            if hrp != CConsts::SOCIETY_NAME { return false; }
            if variant != Variant::Bech32 { return false; }
            // assert_eq!(Vec::<u8>::from_base32(&data).unwrap(), vec![0x00, 0x01, 0x02]);
        }
        Err(e) => {
            return false;
        }
    }
    return true;
}

//old_name_was ComenEncode
pub fn comen_encode(hex_str: &str, hrp: &str) -> String
{
    let normalized_hex: String = CConsts::BECH32_ADDRESS_VER.to_string() + hex_str;
    let normalized_hex = normalized_hex.substring(0, CConsts::TRUNCATE_FOR_BECH32_ADDRESS as usize);
    let byte_vec: Vec<u8> = normalized_hex.as_bytes().to_vec();
    bech32::encode(hrp, byte_vec.to_base32(), Variant::Bech32).unwrap()

    /*
    Bech32data intVec;
    std::string fullBin = "";
    for (char s : normalized_hex){
        std::string bs = std::bitset< 8 >( int(s) ).to_string();
        fullBin += bs;
    }
    int neddedChar = 5 - (fullBin.length() % 5);
    std::string pd="";
    pd.insert(0, neddedChar, '0');
    fullBin += pd;
    const std::string ff=fullBin;
    stChunk binC = chunkString(ff, 5);
    for (std::string aBin: binC){
        intVec.push_back(std::stoi(aBin, nullptr, 2));
    }
    std::string bechEnc = Encode(hrp, intVec);
    return QString::fromStdString(bechEnc);
     */
}