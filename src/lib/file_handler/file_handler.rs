// use std::fs::{File, OpenOptions, remove_file};
use std::fs::{File};
// use std::io::{Write, Read};
use std::io::{Read, Write};
use crate::lib::constants::{Modules, SecLevel};
use crate::lib::dlog::dlog;
use std::path::Path;

pub fn read(
    file_path: &mut String,
    file_name: &String,
    clone_id: i8) -> (bool, String)
{
    if clone_id > 0 {
        file_path.push_str(&clone_id.to_string());
    }

    if file_path != "" {
        file_path.push_str("/");
    }

    file_path.push_str(file_name);

    return read_(file_path);
}

pub fn path_exist(file_full_path: &String) -> bool {
    if Path::new(file_full_path).exists() {
        return true;
    } else {
        return false;
    }
}

use std::fs;
use crate::{ccrypto, constants, cutils, machine};

pub fn mkdir(file_full_path: &String) -> bool {
    return match fs::create_dir(file_full_path) {
        Ok(r) => { true }
        Err(e) => {
            dlog(
                &format!("make dir failed: {}", e),
                constants::Modules::App,
                constants::SecLevel::Error);
            false
        }
    };
}

pub fn read_(file_full_path: &String) -> (bool, String) {
    if !path_exist(file_full_path) {
        return (false, "".to_string());
    }

    // Open the file in read-only mode.
    match File::open(file_full_path) {
        // The file is open (no error).
        Ok(mut file) => {
            let mut content = String::new();

            // Read all the file content into a variable (ignoring the result of the operation).
            file.read_to_string(&mut content).unwrap();

            // *contents = content.clone();
            // The file is automatically closed when is goes out of scope.
            return (true, content);
        }
        // Error handling.
        Err(error) => {
            let err_msg = format!("Error opening file {}: {}", file_full_path, error);
            dlog(&err_msg, Modules::App, SecLevel::Warning);
            return (false, "".to_string());
        }
    }
}


pub fn f_write(
    directory: &String,
    file_name: &String,
    content: &String,
    clone_id: i16) -> bool
{
    let mut file_path = directory.to_string();
    //  if (clone_id>0)
    //    file_path += String::number(clone_id);

    if file_path != "" {
        file_path += "/";
    }

    file_path += file_name;

    return write_(&file_path, content);
}

pub fn write_(file_path: &String, content: &String) -> bool
{
    dlog(
        &format!("wirting file: {}", file_path),
        constants::Modules::App,
        constants::SecLevel::Info);

    let mut file = File::create(file_path)
        .expect("Error encountered while creating file!");
    file.write_all(content.as_ref())
        .expect("Error while writing to file");

    /*
    QFile f(file_path);
    if (!f.open(QIODevice::WriteOnly | QIODevice::Text))
    {
    CLog::log( "Unable to write file" + file_path, "app", "fatal");
    return false;
    }

    QTextStream out(&f);
    out << content;

    f.close();
     */
    return true;
}


//  -  -  -  email part
pub fn writeEmailAsFile(
    title: &String,
    sender: &String,
    receiver: &String,
    email_body: &String) -> bool
{
    dlog(
        &format!("write Em File args: title({title}) sender({sender}) receiver({receiver})"),
        constants::Modules::App,
        constants::SecLevel::Trace);

    // let mut to_send_message: String = message.clone();
    // if is_custom
    // {
    //     to_send_message = constants::message_tags::customStartEnvelope.to_string() + &to_send_message + &constants::message_tags::customEndEnvelope;
    // }

    let outbox: String = machine().get_outbox_path();
    let app_clone_id = machine().get_app_clone_id();
//    if (app_clone_id > 0)
//        outbox = outbox + app_clone_id;

    // let mut email _body: String = cutils::get_now() + constants::NL;
    // email _body += &*("time: ".to_owned() + &cutils::get_now_sss() + &constants::NL);
    // email _body += constants::message_tags::senderStartTag + sender + constants::message_tags::senderEndTag + constants::NL;
    // email_ body += constants::message_tags::receiverStartTag + receiver + constants::message_tags::receiverEndTag + constants::NL;
    // email _body += &*(to_send_message.clone() + &constants::NL);
    // let email hash: String = cutils::hash16c(&ccrypto::keccak256(&(sender + receiver + to_send_message)));
    // email_ body += constants::message_tags::hashStartTag + email _hash.clone() + constants::message_tags::hashEndTag + constants::NL;
    dlog(
        &format!("email body: {}", email_body),
        constants::Modules::App,
        constants::SecLevel::Trace);

    let mut file_name: String = "".to_string();
    if machine().is_develop_mod()
    {
        file_name = [&cutils::get_now_sss(), sender, receiver, title, ".txt"].join(",");
    } else {
        file_name = [receiver, &cutils::get_now_sss(), &ccrypto::get_random_number(5), ".txt"].join(" ");
    }
    dlog(
        &format!("file Name: {}", file_name),
        constants::Modules::App,
        constants::SecLevel::Trace);
    dlog(
        &format!("Try to write1: {}/{}", outbox, file_name),
        constants::Modules::App,
        constants::SecLevel::Trace);


    return f_write(
    &(outbox + &"/"),
    &file_name,
    &email_body,
    app_clone_id);
}
