use crate::{config, App};

use axum::{extract::{ConnectInfo, Json, State}, response::IntoResponse, routing::{patch, get}, Router};
use config::msg::admin_to_server as input_msg;
use config::msg::ServerToAdmin as OutputMsg;
use ocj_config::contest::File;

use std::{net::SocketAddr, sync::Arc};

use crate::{Result, OcjError};

mod tests {
    use super::*;
    pub async fn update(State(app): State<Arc<App>>, ConnectInfo(ci): ConnectInfo<SocketAddr>, Json(msg): Json<input_msg::tests::Update>) -> impl IntoResponse  {
        let ip = ci.ip();
        // let ip = std::net::IpAddr::V4(std::net::Ipv4Addr::new(, b, c, d));
        let r = serde_json::to_string(&match app.auth.unwrap_secure::<Box<File>>(&ip, msg).await {
            Ok(msg) => {
                if let Err(e) = app.update_tests(&msg).await {
                    Err(e)
                } else {
                    Ok(OutputMsg::PermissionGranted(()))
                }
            },
            Err(OcjError::Auth(_)) => {
                Ok(OutputMsg::PermissionDenied())
            },
            Err(e) => Err(e)
        }.unwrap_or_else(|e| OutputMsg::InternalError(e.to_string().into()))).unwrap();
        log::debug!("{r}");
        r
    }
}

mod tokens {
    use super::*;
    pub async fn gen(State(app): State<Arc<App>>, ConnectInfo(ci): ConnectInfo<SocketAddr>, Json(msg): Json<input_msg::tokens::Gen>) -> impl IntoResponse {        
        let ip = ci.ip();
        let r = serde_json::to_string(&match app.auth.login(ip, &msg).await {
            Ok(token) => {
                Ok(OutputMsg::PermissionGranted(token))
            },
            Err(OcjError::Auth(_)) => {
                Ok(OutputMsg::PermissionDenied())
            },
            Err(e) => Err(e)
        }.unwrap_or_else(|e| OutputMsg::InternalError(e.to_string().into()))).unwrap();
        log::debug!("{r}");
        r
    }
}

pub fn router(app: Arc<App>) -> Router<()> {
    let contest: Router<_> = Router::new()
        .route("/tests", patch(tests::update))
        .with_state(app.clone());
    let tokens: Router<_> = Router::new()
        .route("/gen", get(tokens::gen))
        .with_state(app.clone());
    Router::new()
        .nest("/contest", contest)
        .nest("/tokens", tokens)
        .with_state(app.clone())
}

pub async fn server(app: Arc<App>) -> Result<()> {
    let addr = SocketAddr::new(app.ip, config::port::HTTP_FOR_ADMIN);
    let listner = tokio::net::TcpListener::bind(addr).await?;
    let router = router(app);


    log::info!("server for admin cli running on port: {}", addr);
    axum::serve(listner, router.into_make_service_with_connect_info::<SocketAddr>()).await?;
    Ok(())
}

