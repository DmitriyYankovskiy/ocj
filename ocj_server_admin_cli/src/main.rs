mod file;
mod ui;
use std::io::{stdin, stdout, Error, Write};

use chrono::{Local, NaiveDateTime, TimeZone};
use colored::Colorize;

use ocj_config::{self as config, auth::Token, msg::{admin_to_server as output_msg, ServerToAdmin as InputMsg}};

fn parse_date(s: Option<Box<str>>) -> Result<chrono::NaiveDate, &'static str> {
    let date = if let Some(d) = s {d} else {
        return Err("argument not found");
    };

    let date = if let Ok(d) = chrono::NaiveDate::parse_from_str(&date, "%d-%m-%Y") { 
        d
    } else if *date == *"today" {
        Local::now().date_naive()
    } else {
        return Err("argument is incorrect");
    };

    Ok(date)
}

fn parse_time(s: Option<Box<str>>) -> Result<chrono::NaiveTime, &'static str> {
    let time = if let Some(t) = s {t} else {
        return Err("argument not found");
    };

    let time = if let Ok(t) = chrono::NaiveTime::parse_from_str(&time, "%H:%M") { 
        t
    } else if *time == *"now" {
        Local::now().time()
    } else {
        return Err("argument is incorrect");
    };

    Ok(time)
}

fn parse_duration(s: Option<Box<str>>) -> Result<Option<std::time::Duration>, &'static str> {
    let duration = if let Some(d) = s {d} else {
        return Err("argument not found");
    };
    let duration: Option<u32> = if let Ok(d) = duration.parse() {
        Some(d)
    } else if duration.as_ref() == "none" {
        None
    } else {
        return Err("argument is incorrect");
    };

    Ok(duration.map(|d| chrono::Duration::minutes(d.into()).to_std().unwrap()))
}

fn main() -> std::io::Result<()> {
    let args: Box<[String]> = std::env::args().collect();

    let ip = args.get(1).ok_or(Error::new(std::io::ErrorKind::NotFound, "server ip found"))?;
    let key = args.get(2).ok_or(Error::new(std::io::ErrorKind::NotFound, "key not found"))?;
    let key = &Box::<str>::from(key.clone());
    let client = reqwest::blocking::Client::new();

    let ip = &format!("http://{ip}:{}", config::port::HTTP_FOR_ADMIN);

    let token: Token = if let Ok(t) = client.get(format!("{ip}/auth/token")).json(key).send() {
        match t.json().unwrap() {
            InputMsg::Ok(t) => {
                t
            },
            InputMsg::Err(e) => {
                return Err(Error::new(std::io::ErrorKind::Other, e.as_ref()));            
            }
        }
    } else {
        println!("{}", "connection failed".red());
        return Ok(());
    };

    println!("[permission {}]", "granted".bright_green().bold());
    
    let stdin = stdin();
    let mut stdout = stdout();
    let mut s_ch = ui::new_command_line_str();
    loop {
        print!("{} ", s_ch.bright_black());
        s_ch = ui::new_command_line_str();
        stdout.flush().unwrap();
        let mut cmd = String::new();
        stdin.read_line(&mut cmd).unwrap();
        let cmd: Box<[Box<str>]> = cmd.split_ascii_whitespace().map(|s| Box::from(s)).collect();
        match cmd.get(0).unwrap_or(&Box::from("")).as_ref() {
            "exit" | "quit" | "q" => {
                break;
            }

            "help" => {
                println!(r#"
                ===::OCJ::===
                
                for date use <dd-mm-yyyy> or <today>
                for time use <hh:mm> or <now>
                "#)
            }
         
            "tests.upd" => {
                let tests = file::get_compressed_tests()?;
                let msg: output_msg::contest::tests::Update = tests;
                let res: InputMsg<()> = if let Ok(r) = client.patch(format!("{ip}/contest/tests"))
                    .header(config::auth::SECURE_TOKEN_HTTP_HEADER, token.to_string())
                    .json(&msg)
                    .send() {r} else {
                        println!("{}", "connection failed".red());
                        continue;
                    }.json().unwrap();
                match res {
                    InputMsg::Ok(()) => println!("{}", "tests was updated".blue()),
                    InputMsg::Err(e) => println!("{}", e.red()),
                }
            }

            "contest.state.ready" => {
                let start_date = match parse_date(cmd.get(1).cloned()) {
                    Ok(d) => d,
                    Err(e) => {
                        println!("{} {}", "start date".red().bold(), e.red());
                        continue;
                    }
                };

                let start_time = match parse_time(cmd.get(2).cloned()) {
                    Ok(t) => t,
                    Err(e) => {
                        println!("{} {}", "start time".red().bold(), e.red());
                        continue;
                    }
                };

                let duration = match parse_duration(cmd.get(3).cloned()) {
                    Ok(o) => o,
                    Err(e) => {
                        println!("{} {}", "duration".red().bold(), e.red());
                        continue;
                    }
                };

                let start = Local.from_local_datetime( &NaiveDateTime::new(start_date, start_time)).unwrap();

                if start + chrono::Duration::seconds(1) < Local::now() {
                    println!("{} {}", "start date_time can't be before".red(), "now".red().bold());
                    continue;
                }

                let res: InputMsg<()> = client.post(format!("{ip}/contest/state/ready"))
                    .header(config::auth::SECURE_TOKEN_HTTP_HEADER, token.to_string())
                    .json(&output_msg::contest::state::SetReady {
                        start: start.into(),
                        duration: duration,
                    })
                    .send().unwrap()
                    .json().unwrap();

                if let InputMsg::Err(e) = res {
                    println!("{}", e.red());
                } else {
                    println!("{} {}", "new contest state:".bright_blue(), "READY".bold().blue());
                }
            }
            
            "" => {
                s_ch = ui::prev_command_line_str();
            }
            _ => {
                println!("{}", "unknow command".red())
            }
        }
    }
    Ok(())
}
