use std::sync::Arc;
use tokio::sync::Mutex;

use crate::config;

pub struct Service {
    contest: Mutex<Contest>,
}

impl Service {
    pub fn init() -> Self {
        Self {
            contest: Mutex::new(Contest::InDevelop),
        }
    }
}

pub enum Contest {
    InDevelop,
    Ready{
        time: Arc<Mutex<config::contest::Time>>
    },
    Going {
        time: Arc<Mutex<config::contest::Time>>,
    },
    Finished,
}