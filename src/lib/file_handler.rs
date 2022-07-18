// use std::fs::{File, OpenOptions, remove_file};
use std::fs:: {File};
// use std::io::{Write, Read};
use std::io:: Read;



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

pub fn read_(file_full_path: &String) -> (bool, String) {

    println!("file_full_path: {}", file_full_path);

    // let file_full_path = "src/zzz.txt";
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
        },
        // Error handling.
        Err(error) => {
            // println!("Error opening file {}: {}", file_full_path, error);
            return (false, "".to_string());
        },
    }
}