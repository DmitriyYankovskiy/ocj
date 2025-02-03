use tokio::{net::TcpStream, sync::Mutex};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use futures::{SinkExt, StreamExt};

use std::{net::SocketAddr, sync::Arc};

use crate::{config, machine::{InputMsg, Machine}, App, Result};

async fn ws_connect(stream: TcpStream, app: Arc<App>) -> Result<()> {
    let machine_service = &app.machine;
	let websocket = match accept_async(stream).await {
        Err(e) => {
            log::error!("while conecting with websockets: {e:?}");
            return Err(e.into());
        },
        Ok(r) => r,
    };

	let (sender, mut receiver) = websocket.split();
    let sender = Arc::new(Mutex::new(sender));

    let (i_ch_sender, i_ch_receiver) = tokio::sync::mpsc::unbounded_channel();
    let (o_ch_sender, mut o_ch_receiver) = tokio::sync::mpsc::unbounded_channel();

    let id = rand::random::<u16>();

    machine_service.add_machine(Machine::new(id, o_ch_sender, i_ch_receiver)).await;

    let sender_clone = Arc::clone(&sender);
    let i_task: tokio::task::JoinHandle<Result<()>> = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            if let Ok(msg) = msg {
                match msg {
                    Message::Ping(b) => {
                        sender_clone.lock().await.send(Message::Pong(b)).await?;
                    },
                    Message::Text(data) => {
                        i_ch_sender.send(serde_json::from_str::<InputMsg>(&data.to_string()).unwrap()).unwrap();
                    },
                    _ => {},
                }
            } else {
                log::info!("ws close");
                return Ok(());
            }
        }
        Ok(())
    });

    let o_task: tokio::task::JoinHandle<Result<()>> = tokio::spawn(async move {
        while let Some(msg) = o_ch_receiver.recv().await {
            sender.lock().await.send(Message::Text(serde_json::to_string(&msg).unwrap().into())).await?
        }
        Ok(())
    });

    // app.judge(crate::Solution{code: "code".to_string(), lang: crate::solution::Lang::Cpp, task_number: 12, id: 1332}).await;

    tokio::select! {
        r = i_task => log::error!("input machine task error {:?}", r),
        r = o_task => log::error!("output machine task error {:?}", r),
    }

    machine_service.remove_machine(id).await;

    Ok(())
} 

pub async fn server(app: Arc<App>) -> Result<()> {
    let ws_addr = SocketAddr::new(app.ip, config::port::WS_FOR_MACHINE);
    let listner = match tokio::net::TcpListener::bind(ws_addr).await {
        Ok(l) => { l },
        Err(e) => {
            return Err(e.into());
        }
    };

    loop {
        let (stream, socket_addr) = match listner.accept().await {
            Err(e) => {
                return Err(e.into())
            },
            Ok(r) => r,
        };

        log::debug!("machine ws connect by addr: {socket_addr:?}");
        tokio::spawn(ws_connect(stream, Arc::clone(&app)));
    }
}