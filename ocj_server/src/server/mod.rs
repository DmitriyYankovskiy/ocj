pub mod client;
pub mod machine;
pub mod admin;

use std::sync::Arc;

use crate::{App, Result};


pub async fn run(app: &Arc<App>) -> Result<()> {
    let client = tokio::spawn(client::server(Arc::clone(&app)));
    let machine = tokio::spawn(machine::server(Arc::clone(&app)));
    let admin= tokio::spawn(admin::server(Arc::clone(&app)));

    tokio::select! {
        r = client => {
            r?.into()
        },
        r = machine => {
            r?.into()
        },
        r = admin => {
            r?.into()
        },
    }
}