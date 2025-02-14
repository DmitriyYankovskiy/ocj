mod server;
mod auth;
mod client;
mod machine;
mod error;
mod file;
mod contest_time;

use error::{OcjError, Result};
use ocj_config::{self as config, solution::Lang};
use tokio::sync::Mutex;

use std::{collections::HashMap, net::IpAddr, sync::Arc, time::Duration};

use config::solution::{self, Solution};

struct App {
    ip: IpAddr,

    auth: auth::Service,
    machine: machine::Service,
    contest_time: contest_time::Service,
    solutions: Mutex<HashMap<solution::Id, client::Id>>,
}

impl App {
    pub async fn init(auth: auth::Service, machine: machine::Service, contest_time: contest_time::Service) -> Result<Self> {
        Ok(Self {
            auth,
            ip: {
                let ip = local_ip_address::local_ip()?;
                println!("server ip: {}", ip);
                ip
            },
            machine,
            contest_time,

            solutions: Mutex::new(HashMap::new()),
        })
    }

    pub async fn judge(&self, solution: &Solution) -> Result<()> {  
        let id = {
            let mut machines_tasks_counters = self.machine.machines_tasks_counters.lock().await;
            let mut machines = self.machine.machines.lock().await;
            let (count, id) = *machines_tasks_counters.first().ok_or(OcjError::NoneMachineFound)?;

            machines_tasks_counters.remove(&(count, id));
            machines_tasks_counters.insert((count + 1, id));
            *machines.get_mut(&id).unwrap().tasks_count.lock().await += 1;
            id
        };

        self.machine.machines.lock().await.get_mut(&id).unwrap().send_solution(&solution);

        Ok(())
    }

    pub async fn update_tests(&self, data: &[u8]) -> Result<()> {
        file::update_tests(data).await?;
        self.machine.broadcast_update_tests(data).await;
        Ok(())
    }

    pub async fn start_contest(&self) -> Result<()> {
        log::info!("contest started");
        Ok(())
    }

    pub async fn finish_contest(&self) -> Result<()> {
        log::info!("contest finished");
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    
    let key = std::env::args().nth(1).ok_or(OcjError::EnvArgsNotFound("key"))?;
    
    let auth = auth::Service::init(&key);
    let machine = machine::Service::init();
    let contest = contest_time::Service::init();

    let app = Arc::new(App::init(auth, machine, contest).await?);
    let app_clone = app.clone();
    _ = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(10)).await;
        for i in 1..10 {
            let r = app_clone.judge(&Solution { 
                code: r#"
                    #include<bits/stdc++.h>
                    using namespace std;

                    int main() {
                        int a, b;
                        cin >> a >> b;

                        cout << a + b + 1 << '\n';
                    }
                "#.to_string(),
                lang: Lang::Cpp,
                problem_number: 1,
                id: i,
            }).await;
            if let Err(e) = r {
                log::error!("{e}");
            }
        }
    });
    server::run(&app).await?;
    Ok(())
}
