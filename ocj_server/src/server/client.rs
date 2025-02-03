use crate::{config, App, Result};

use axum::Router;

use std::{net::SocketAddr, sync::Arc};


pub async fn server(app: Arc<App>) -> Result<()> {
    let addr = SocketAddr::new(app.ip, config::port::HTTP_FOR_CLIENT);
    let listner = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => { l },
        Err(e) => {
            log::error!("can't bind TcpListner to port {}: {}", addr, e);
            return Err(e.into());
        }
    };

    let router = Router::new();

    log::info!("server running on port: {}", addr);
    if let Err(e) = axum::serve(listner, router).await {
        log::error!("server running with error {:?}", e);
        Err(e.into())
    } else {
        Ok(())
    }
}