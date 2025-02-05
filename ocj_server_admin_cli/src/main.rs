mod file;
use std::io::Error;

use colored::Colorize;

use ocj_config::{self as config, auth::Token, msg::{admin_to_server as output_msg, ServerToAdmin as InputMsg}};

// #[tokio::main]
fn main() -> std::io::Result<()> {
    let args: Box<[String]> = std::env::args().collect();

    let ip = args.get(1).ok_or(Error::new(std::io::ErrorKind::NotFound, "server ip found"))?;
    let key = args.get(2).ok_or(Error::new(std::io::ErrorKind::NotFound, "key not found"))?;
    let key = &Box::<str>::from(key.clone());
    let client = reqwest::blocking::Client::new();

    let ip = &format!("http://{ip}:{}", config::port::HTTP_FOR_ADMIN);

    let token: Token = match client.get(format!("{ip}/auth/token")).json(key).send().unwrap().json().unwrap() {
        InputMsg::Ok(t) => {
            t
        },
        InputMsg::Err(e) => {
            return Err(Error::new(std::io::ErrorKind::Other, e.as_ref()));            
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
                let msg: output_msg::contest::tests::Update = tests;
                let res: InputMsg<()> = client.patch(format!("{ip}/contest/tests"))
                    .header(config::auth::SECURE_TOKEN_HTTP_HEADER, token.to_string())
                    .json(&msg)
                    .send().unwrap()
                    .json().unwrap();
                match res {
                    InputMsg::Ok(()) => println!("{}", "tests was updated".blue()),
                    InputMsg::Err(e) => println!("{}", e.red()),
                }
            }
            _ => {}
        }
    }
    Ok(())
}
