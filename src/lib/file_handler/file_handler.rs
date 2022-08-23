use std::fs::{File};
use std::io::{Read, Write};
use crate::lib::constants::{Modules, SecLevel};
use crate::lib::dlog::dlog;
use std::path::Path;
use std::fs;
use substring::Substring;
use crate::{application, ccrypto, constants, machine};

pub fn file_read(
    mut file_path: String,
    file_name: String,
    clone_id: i8) -> (bool, String)
{
    if clone_id > 0 {
        file_path = format!("{file_path}{clone_id}");
    }

    if file_path != "" {
        file_path = format!("{file_path}/");
    }

    let file_path = &get_os_care_path(&format!("{file_path}{file_name}"));

    if !path_exist(file_path) {
        return (false, format!("Path (to read) does not exist! {}", file_path));
    }

    return read_exact_file(file_path);
}


pub fn read_exact_file(file_full_path: &String) -> (bool, String) {
    let file_full_path = get_os_care_path(file_full_path);

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
        Ok(_r) => { true }
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
    clone_id: i8) -> (bool, String)
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

    file_path = get_os_care_path(&format!("{}{}", file_path, file_name));

    return write_exact_file(&file_path, content);
}

pub fn write_exact_file(file_path: &String, content: &String) -> (bool, String)
{
    let file_path = &get_os_care_path(file_path);

    dlog(
        &format!("wirting file: {}", file_path),
        constants::Modules::App,
        constants::SecLevel::Debug);

    let mut file = File::create(file_path)
        .expect("Error encountered while creating file!");
    file.write_all(content.as_ref())
        .expect("Error while writing to file");

    return (true, "File Writed".to_string());
}

pub fn delete_exact_file(file_path: &String) -> bool
{
    let file_path = &get_os_care_path(file_path);
    println!(":::::delete_exact_file file_path: {}", file_path);

    return match fs::remove_file(file_path)
    {
        Ok(_r) =>
            {
                true
            }
        Err(e) => {
            eprintln!("Failed in file delete {}: {}", file_path, e);
            false
        }
    };
}

pub fn list_exact_files(folder_path: &String, filter_by_extension: &str) -> Vec<String> {
    let folder_path = get_os_care_path(folder_path);

    let mut out: Vec<String> = vec![];
    if filter_by_extension == ""
    {
        let paths = fs::read_dir(folder_path).unwrap();

        for path in paths {
            out.push(path.unwrap().path().display().to_string());
        }
        out
    } else {
        // let mut faxvec: Vec<std::path::PathBuf> = Vec::new();
        for element in std::path::Path::new(folder_path.as_str()).read_dir().unwrap() {
            let path = element.unwrap().path();
            if let Some(extension) = path.extension() {
                if extension == filter_by_extension {
                    out.push(path.display().to_string());
                }
            }
        }
        out
    }
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


//  -  -  -  email part
//old_name_was writeEmailAsFile
pub fn write_email_as_file(
    title: &String,
    sender: &String,
    receiver: &String,
    email_body: &String) -> bool
{
    dlog(
        &format!("write Em File args: title({title}) sender({sender}) receiver({receiver})"),
        constants::Modules::App,
        constants::SecLevel::Info);

    let outbox: String = application().outbox_path();
    dlog(
        &format!("email body: {}", email_body),
        constants::Modules::App,
        constants::SecLevel::Trace);

    let mut file_name: String = "".to_string();
    if machine().is_develop_mod()
    {
        file_name = [receiver, sender, &application().get_now_sss(), title, ".txt"].join(",");
    } else {
        file_name = [receiver, sender, &application().get_now_sss(), &ccrypto::get_random_number(5), ".txt"].join(" ");
    }
    dlog(
        &format!("file Name: {}", file_name),
        constants::Modules::App,
        constants::SecLevel::Info);
    dlog(
        &format!("Try to write1: {}/{}", outbox, file_name),
        constants::Modules::App,
        constants::SecLevel::Info);

    let (status, _msg) = write_exact_file(
        &format!("{}/{}", outbox, file_name),
        &email_body);
    status
}
