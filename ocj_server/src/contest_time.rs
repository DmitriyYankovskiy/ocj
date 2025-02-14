use std::{ops::DerefMut, sync::{Arc, Weak}, time::{Duration, SystemTime}};
use tokio::{sync::Mutex, task::JoinHandle};

use crate::{config::{self, contest::{Time, UpdateDuration}}, error::{self, ContestError}, App, Result};

pub struct Service {
    contest: Mutex<State>,
}

impl Service {
    pub fn init() -> Self {
        Self {
            contest: Mutex::new(State::InDevelop),
        }
    }

    pub async fn ready(&self, time: &config::contest::Time, app: Weak<App>) -> Result<()> {
        let mut contest = self.contest.lock().await;
        if let State::InDevelop = *contest {
            *contest = State::Ready { 
                time: time.clone(),
                starter: tokio::spawn(Self::starter(app, time.start)),
            };
            log::info!("contest state: READY");
            Ok(())
        } else {
            Err(error::ContestError::AlreadyReady.into())
        }
    }

    pub async fn update_start_time (&self, start_time: std::time::SystemTime, app: Weak<App>) -> Result<()> {
        let mut contest = self.contest.lock().await;
        match contest.deref_mut() {
            State::Ready {time, starter} => {
                *time = Time {
                    start: start_time.clone(),
                    ..*time
                };
                starter.abort();
                *starter = tokio::spawn(Self::starter(app, start_time));
                Ok(())
            }
            State::InDevelop => {
                Err(ContestError::StillInDevelop.into())
            }
            _ => {
                Err(ContestError::AlreadyGoing.into())
            }
        }
    }

    pub async fn update_duration (&self, op: UpdateDuration, app: Arc<App>) -> Result<()> {
        let mut contest = self.contest.lock().await;
        let app = Arc::downgrade(&app);
        let contest = contest.deref_mut();
        match contest {
            State::Ready {time, ..} | State::Going {time, ..}=> {
                let prev_duration = time.duration;
                *time = Time {
                    duration: match op {
                        UpdateDuration::Add(dur) => {
                            prev_duration.map(|d| d + dur)
                        }
                        UpdateDuration::Sub(dur) => {
                            prev_duration.map(|d| d - dur)
                        }
                        UpdateDuration::Set(dur) => dur,
                    },
                    ..*time
                };
                Result::Ok(())
            }
            State::InDevelop => {
                Err(ContestError::StillInDevelop.into())
            }
            _ => {
                Err(ContestError::AlreadyGoing.into())
            }
        }?;
        if let State::Going { time, finisher } = contest {
            if let Some (d) = time.duration {
                *finisher = Some(tokio::spawn(Self::finisher(app, time.start + d)));
            }
        }
        Ok(())
    }

    pub async fn starter(app: Weak<App>, start_time: std::time::SystemTime) {
        tokio::time::sleep(start_time.duration_since(SystemTime::now()).unwrap_or(Duration::ZERO)).await;
        _ = app.upgrade().unwrap().start_contest().await.or_else(|e| {
            log::error!("error while starting contest: {e:?}");
            Err(e)
        });
        
    }

    pub async fn finisher(app: Weak<App>, end_time: std::time::SystemTime) {
        tokio::time::sleep(end_time.duration_since(SystemTime::now()).unwrap_or(Duration::ZERO)).await;
        _ = app.upgrade().unwrap().finish_contest().await.or_else(|e| {
            log::error!("error while finishing contest: {e:?}");
            Err(e)
        });
    }
}



pub enum State { 
    InDevelop,
    Ready {
        time: Time,
        starter: JoinHandle<()>,
    },
    Going {
        time: Time,
        finisher: Option<JoinHandle<()>>,
    },
    Finished,
}