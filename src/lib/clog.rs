pub fn log (msg: &String, module: &String, level: &String) {
    println!("{}({}): {}", module, level, msg);
}
