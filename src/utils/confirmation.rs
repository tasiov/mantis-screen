use std::io;

/// Get confirmation from the user
pub fn get_confirmation(message: &str) -> () {
    println!("{}", message);
    println!("Are you sure you want to proceed? (Y/n)");
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    if input.trim() == "Y" {
        ()
    } else {
        std::process::exit(0);
    }
}
