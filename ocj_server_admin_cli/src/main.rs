mod file;
use std::io::Error;

use colored::Colorize;

use ocj_config::{self as config, auth::Token, msg::{admin_to_server as output_msg, Secure, ServerToAdmin as InputMsg}};

// #[tokio::main]
fn main() -> std::io::Result<()> {
    let args: Box<[String]> = std::env::args().collect();

    let ip = args.get(1).ok_or(Error::new(std::io::ErrorKind::NotFound, "server ip found"))?;
    let key = args.get(2).ok_or(Error::new(std::io::ErrorKind::NotFound, "key not found"))?;
    let key = &Box::<str>::from(key.clone());
    let client = reqwest::blocking::Client::new();

    let ip = &format!("http://{ip}:{}", config::port::HTTP_FOR_ADMIN);

    let token: Token = match client.get(format!("{ip}/tokens/gen")).json(&key).send().unwrap().json().unwrap() {
        InputMsg::PermissionGranted(t) => {
            t
        },
        InputMsg::InternalError(e) => {
            return Err(Error::new(std::io::ErrorKind::Other, format!("internal server error {e}")));            
        }
        InputMsg::PermissionDenied() => {
            return Err(Error::new(std::io::ErrorKind::PermissionDenied, "incorrect key"));
        }
    };

    println!("[permission {}]", "granted".bright_green().bold());
    
    loop {
        print!("{}", "#: ".yellow());
        let cmd: String = text_io::read!("{}");
        match cmd.as_str() {
            "exit" | "quit" | "q" => {
                break;
            }
         
            "tests:upd" => {
                let tests = file::get_compressed_tests()?;
                let msg: output_msg::tests::Update = Secure::new(token, tests);
                if let InputMsg::PermissionGranted(()) = client.patch(format!("{ip}/contest/tests")).json(&msg).send().unwrap().json().unwrap() {
                    println!("{}", "tests was updated".blue());
                } else {
                    println!("{}", "error".red());
                }
            }
            _ => {}
        }
    }
    Ok(())
}
