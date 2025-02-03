use crate::config;
use std::net::IpAddr;

use futures::{stream::StreamExt, SinkExt};

use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

use crate::App;

pub type InputMsg = config::msg::ServerToMachine;
pub type OutputMsg = config::msg::MachineToServer;

pub async fn run(ip: &IpAddr, _app: App, i_sender: UnboundedSender<InputMsg>, mut o_receiver: UnboundedReceiver<OutputMsg>) -> Result<(), ()> {
    let ws_addr = format!("ws://{}:{}", ip.to_string(), config::port::WS_FOR_MACHINE);
    let ws = if let Ok((ws, _)) = connect_async(ws_addr).await {
        ws
    } else {
        log::error!("can't connect to server with websockets");
        return Err(())
    };

    let (mut write, mut read) = ws.split();

    let i_task = tokio::spawn( async move {
        while let Some(msg) = read.next().await {
            let msg = if let Ok(Message::Text(data)) = msg {
                data.to_string()
            } else {
                log::error!("incorrect input websocket message");
                return Err(());
            };
            let msg = if let Ok(msg) = serde_json::from_str::<InputMsg>(&msg) {
                msg
            } else {
                log::error!("incorrect input websocket message");
                return Err(());
            };

            i_sender.send(msg).unwrap();
        }
        Ok(())
    });

    let o_task = tokio::spawn( async move {
        while let Some(msg) = o_receiver.recv().await {
            write.send(Message::Text(serde_json::to_string(&msg).unwrap().into())).await.unwrap();
        }
    });

    tokio::select! {
        _ = o_task => {},
        _ = i_task => {},
    }

    Ok(())
}