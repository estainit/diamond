use std::fs::{File};
use std::io::{Read, Write};
use crate::lib::constants::{Modules, SecLevel};
use crate::lib::dlog::dlog;
use std::path::Path;
use std::fs;
use substring::Substring;
use crate::{ccrypto, constants, cutils, machine};

pub fn file_read(
    mut file_path: String,
    file_name: String,
    clone_id: i16) -> (bool, String)
{
    if clone_id > 0 {
        file_path = format!("{file_path}{clone_id}");
    }

    if file_path != "" {
        file_path = format!("{file_path}/");
    }

    file_path = get_os_care_path(&format!("{file_path}{file_name}"));

    if !path_exist(&file_path) {
        return (false, format!("Path (to read) does not exist! {}", file_path));
    }

    return read_exact_file(file_path);
}


pub fn read_exact_file(file_full_path: String) -> (bool, String) {
    let file_full_path = get_os_care_path(&file_full_path);

    // Open the file in read-only mode.
    match File::open(file_full_path.clone()) {
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

pub fn path_exist(file_full_path: &String) -> bool {
    let file_full_path = &get_os_care_path(file_full_path);
    if Path::new(file_full_path).exists() {
        return true;
    } else {
        return false;
    }
}

pub fn mkdir(file_full_path: &String) -> bool {
    let file_full_path = &get_os_care_path(file_full_path);
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


pub fn file_write(
    directory: String,
    file_name: String,
    content: &String,
    clone_id: i16) -> (bool,String)
{
    let mut file_path = directory.clone();

    if clone_id > 0 {
        file_path = format!("{file_path}{clone_id}");
    }

    if file_path != "" {
        file_path = format!("{file_path}/");
    }

    file_path = get_os_care_path(&file_path);

    if !path_exist(&file_path) {
        return (false, format!("Path (to write) does not exist! {}", file_path));
    }

    file_path = get_os_care_path(&format!("{file_path}{file_name}"));

    return write_exact_file(&file_path, content);
}

pub fn write_exact_file(file_path: &String, content: &String) -> (bool,String)
{
    let file_path = &get_os_care_path(file_path);

    dlog(
        &format!("wirting file: {}", file_path),
        constants::Modules::App,
        constants::SecLevel::Info);

    let mut file = File::create(file_path)
        .expect("Error encountered while creating file!");
    file.write_all(content.as_ref())
        .expect("Error while writing to file");

    return (true, "File Writed".to_string());
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


    let (status, _msg)=file_write(
        outbox,
        file_name,
        &email_body,
        app_clone_id);
    status
}

pub fn get_os_care_path(the_path: &String) -> String {
    if std::env::consts::OS == "windows" {
        let s1 = the_path.substring(3, the_path.len()).to_string();
        let s2 = s1.replace("/", "\\").replace(":", "_");
        let mut s3 = the_path.substring(0, 3).to_string();
        s3.push_str(&s2);
        return s3;
    }
    return the_path.clone();
}
