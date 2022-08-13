// use std::fs::{File, OpenOptions, remove_file};
use std::fs::{File};
// use std::io::{Write, Read};
use std::io::Read;
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
use crate::constants;

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
     directory:&String,
     file_name:&String,
     content:&String,
    clone_id:i16)->bool
{
    let mut file_path  = directory.to_string();
    //  if (clone_id>0)
    //    file_path += String::number(clone_id);

    if file_path != "" {
        file_path += "/";
    }

    file_path += file_name;

    return write_(&file_path, content);
}

pub fn write_( file_path:&String,  content:&String)->bool
{
    dlog(
        &format!("wirting file: {}", file_path),
        constants::Modules::App,
        constants::SecLevel::Error);

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
