use std::{collections::{BTreeSet, HashMap}, sync::Arc};

use ocj_config::contest::File;
use tokio::{io::AsyncReadExt, sync::{mpsc::{UnboundedReceiver, UnboundedSender}, Mutex}, task::JoinHandle};

use crate::{config, solution::Solution, file, Result};

pub type Id = u16;
pub type InputMsg = config::msg::MachineToServer;
pub type OutputMsg = config::msg::ServerToMachine;

pub struct Service {
    pub machines: Mutex<HashMap<Id, Arc<Machine>>>,
    pub machines_tasks_counters: Mutex<BTreeSet<(usize, Id)>>,

    pub machine_handles: Mutex<HashMap<Id, JoinHandle<()>>>,
}


impl Service {
    pub fn init() -> Self {
        Self {
            machines: Mutex::new(HashMap::new()),
            machine_handles: Mutex::new(HashMap::new()),
            machines_tasks_counters: Mutex::new(BTreeSet::new()),
        }
    }

    pub async fn add_machine(&self, machine: Machine) {
        let id = machine.id;
        let machine = Arc::new(machine);
        self.machines.lock().await.insert(id, machine.clone());
        self.machines_tasks_counters.lock().await.insert((0, id));
        
        self.machine_handles.lock().await.insert(id, Machine::handle(machine));
    }

    pub async fn remove_machine(&self, id: Id) {
        let mut machines = self.machines.lock().await;
        let mut machines_tasks_counters = self.machines_tasks_counters.lock().await;
        let tasks_count = *machines.get(&id).unwrap().tasks_count.lock().await;
        machines.remove(&id);
        self.machine_handles.lock().await.remove(&id);
        machines_tasks_counters.remove(&(tasks_count, id));
    }

    pub async fn broadcast_update_tests(&self, data: &File) {
        self.broadcast(OutputMsg::UpdateTests(Box::from(data))).await;
    }

    pub async fn broadcast(&self, msg: OutputMsg) {
        let machines = self.machines.lock().await;
        for (_, machine) in &*machines {
            machine.ws_sender.send(msg.clone()).unwrap();
        }
    }
}

pub struct Machine {
    pub id: Id,
    pub tasks_count: Mutex<usize>,

    ws_sender: UnboundedSender<OutputMsg>,
    ws_receiver: Mutex<UnboundedReceiver<InputMsg>>,
}

impl Machine {
    pub fn new(id: Id, ws_sender: UnboundedSender<OutputMsg>, ws_receiver: UnboundedReceiver<InputMsg>) -> Self {
        Self {id, tasks_count: Mutex::new(0), ws_sender, ws_receiver: Mutex::new(ws_receiver)}
    }

    pub fn send_solution(&self, solution: &Solution) {
        self.ws_sender.send(OutputMsg::JudgeSolution(solution.clone())).unwrap();
    }

    pub fn send_tests(&self, file: Box<File>) {
        self.ws_sender.send(OutputMsg::UpdateTests(file)).unwrap()
    }

    pub async fn init(&self) -> Result<()> {
        let mut file = file::get_tests().await.or_else(|e| {
            self.ws_sender.send(OutputMsg::InitFailed).unwrap();
            Err(e)
        })?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).await.unwrap();
        self.send_tests(Box::from(buf));
        Ok(())
    }

    pub fn handle(self: Arc<Self>) -> JoinHandle<()> {
        tokio::spawn(async move {
            while let Some(msg) = self.ws_receiver.lock().await.recv().await {
                match msg {
                    InputMsg::Init => {
                        if let Err(e) = self.init().await {
                            log::error!("{e}");
                        }
                    },
                    _ => ()
                }
            };
        })
    }
}