use crate::dlog;
use crate::lib::address::basic_addresses::createANewBasicAddress;
use crate::lib::address::strict_address::create_a_new_strict_address;
use crate::lib::constants;
use crate::lib::transactions::basic_transactions::signature_structure_handler::unlock_document::UnlockDocument;

//old_name_was createANewAddress
pub fn create_a_new_address<'a>(
    signature_type: &'a str,
    signature_mod: &'a str, // m of n
    signature_version: &'a str) -> (bool, UnlockDocument)
{
    if signature_type == constants::signature_types::Basic
    {
        if signature_mod == "Complex"
        {
//      return complexAddressHandler.createANewComplexModAddress(args);
        } else {
            return createANewBasicAddress(signature_mod, signature_version);
        }
    } else if (signature_type == constants::signature_types::Strict)
    {
        return create_a_new_strict_address(signature_mod, signature_version);
    } else if (signature_type == constants::signature_types::Bitcoin)
    {
//    return bitcoinAddressHandler.createANewBitcoinAddress(args);
    }

    dlog(
        &format!("Unknown address signatureType({signature_type}) to create!"),
        constants::Modules::App,
        constants::SecLevel::Fatal);

    return (false, UnlockDocument::get_null());
}

