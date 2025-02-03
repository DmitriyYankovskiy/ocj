mod server;
mod judge;
mod file;

use ocj_config::{self as config, solution::{JudgeResult, Verdict}};

use std::{net::IpAddr, str::FromStr, sync::Arc};
use tokio::{io::AsyncWriteExt, sync::{mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender}, Mutex}};

use server::{InputMsg, OutputMsg};

#[derive(Clone)]
struct App {
    statements_exist: Arc<Mutex<bool>>,
    ws_receiver: Arc<Mutex<UnboundedReceiver<InputMsg>>>,
    ws_sender: UnboundedSender<OutputMsg>,
}

impl App {
    pub fn init() -> (App, UnboundedSender<InputMsg>, UnboundedReceiver<OutputMsg>) {
        let (i_sender, i_receiver) = unbounded_channel();
        let (o_sender, o_receiver) = unbounded_channel();
        (
            Self {
                statements_exist: Arc::new(Mutex::new(true)),
                ws_sender: o_sender,
                ws_receiver: Arc::new(Mutex::new(i_receiver)),
            },
            i_sender,
            o_receiver,
        )
    }

    pub async fn run(&mut self) {
        self.ws_sender.send(OutputMsg::Init).unwrap();
        loop {
            let msg = self.ws_receiver.lock().await.recv().await;
            let msg = if let Some(msg) = msg {
                msg
            } else {
                log::info!("server was closed");
                break;
            };
            match msg {
                InputMsg::UpdateTests(bytes) => {
                    log::info!("tests was updated");
                    let mut tests = tokio::fs::File::create(format!("{}.tar.gz", config::file::TESTS)).await.unwrap();
                    tests.write(&bytes).await.unwrap();
                    file::decompress_tests().await;
                },
                InputMsg::JudgeSolution(solution) => {
                    log::info!("judge solution [{}]", solution.id);
                    let solution_id = solution.id;
                    let problem_number = solution.problem_number;
                    let verdict = crate::judge::judge(solution).await;
                    let verdict = if let Ok(v) = verdict {
                        v
                    } else {
                        continue;
                    };
                    self.ws_sender.send(OutputMsg::JudgeResult(JudgeResult {
                        score: if let Verdict::Ok = verdict {100} else {0},
                        solution_id,
                        verdict,
                        problem_number,
                    })).unwrap();
                }
                ocj_config::msg::ServerToMachine::InitFailed => todo!(),
            };
        };
    }
}

#[tokio::main]
async fn main() -> Result<(), ()>{
    env_logger::init();
    file::init().await;

    let mut args = std::env::args();
    let ip = args.nth(1);
    if let None = ip {
        log::error!("arg 1 (server address) not found");
        return Err(());
    }
    let ip = match IpAddr::from_str(&ip.unwrap()) {
        Ok(ip) => ip,
        Err(_) => {
            log::error!("can't parse arg1 to ip address");
            return Err(());
        }
    };

    let (app, i_sender, o_receiver) = App::init();
    let mut app_clone = app.clone();
    let _app_task = tokio::spawn(async move {app_clone.run().await});

    server::run(&ip, app, i_sender, o_receiver).await?;

    Ok(())
}
