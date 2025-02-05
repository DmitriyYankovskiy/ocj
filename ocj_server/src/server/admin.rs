use crate::{config, App};

use axum::{extract::{ConnectInfo, Json, Request, State}, http::StatusCode, middleware::{self, Next}, response::{IntoResponse, Response}, routing::{get, patch}, Router};
use config::msg::admin_to_server as input_msg;
use config::msg::ServerToAdmin as OutputMsg;
use ocj_config::{auth::Token, contest::File};

use std::{net::SocketAddr, sync::Arc};

use crate::{Result, OcjError};

async fn auth_mw(State(app): State<Arc<App>>, ConnectInfo(ci): ConnectInfo<SocketAddr>, req: Request, next: Next) -> std::result::Result<Response, StatusCode> {
    let token: Token = req.headers()
        .get(config::auth::SECURE_TOKEN_HTTP_HEADER).ok_or(StatusCode::LOCKED)?
        .to_str().or(Err(StatusCode::LOCKED))?
        .parse().or(Err(StatusCode::LOCKED))?;
    let ip = ci.ip();
    if let Err(e) = app.auth.check_token(&ip, &token).await {
        Err(if let OcjError::Auth(_) = e {
            StatusCode::LOCKED
        } else {
            StatusCode::INTERNAL_SERVER_ERROR
        })
    } else {
        log::debug!("ADMIN PERMISSION GRANTED");
        Ok(next.run(req).await)
    }
}

mod contest {
    use super::*;
    pub mod tests {
        use super::*;
        pub async fn update(State(app): State<Arc<App>>, Json(msg): Json<input_msg::contest::tests::Update>) -> impl IntoResponse  {            
            if let Err(e) = app.update_tests(&msg).await {
                Json::from(OutputMsg::<()>::Err(e.to_string().into()))
            } else {
                Json::from(OutputMsg::Ok(()))
            }
        }
    }

    // pub mod time {
    //     use super::*;
    //     pub async fn update(State(app): State<Arc<App>>, ConnectInfo(ci): ConnectInfo<SocketAddr>, Json(msg): Json<input_msg::contest::tests::Update>) -> impl IntoResponse  {
    //         let ip = ci.ip();
    //         // let ip = std::net::IpAddr::V4(std::net::Ipv4Addr::new(, b, c, d));
    //         let r = serde_json::to_string(&match app.auth.unwrap_secure::<Box<File>>(&ip, msg).await {
    //             Ok(msg) => {
    //                 if let Err(e) = app.update_tests(&msg).await {
    //                     Err(e)
    //                 } else {
    //                     Ok(OutputMsg::PermissionGranted(()))
    //                 }
    //             },
    //             Err(OcjError::Auth(_)) => {
    //                 Ok(OutputMsg::PermissionDenied())
    //             },
    //             Err(e) => Err(e)
    //         }.unwrap_or_else(|e| OutputMsg::InternalError(e.to_string().into()))).unwrap();
    //         log::debug!("{r}");
    //         r
    //     }
    // }
}

mod auth {
    use super::*;
    pub async fn token(State(app): State<Arc<App>>, ConnectInfo(ci): ConnectInfo<SocketAddr>, Json(msg): Json<input_msg::tokens::Get>) -> impl IntoResponse {        
        log::debug!("nt");
        let ip = ci.ip();
        let r = match app.auth.login(ip, &msg).await {
            Ok(token) => {
                OutputMsg::Ok(token)
            },
            Err(OcjError::Auth(_)) => {
                log::warn!("try get accept by ip: {:?}", ip);
                return StatusCode::LOCKED.into_response();
            },
            Err(e) => OutputMsg::Err(e.to_string().into())
        };
        log::debug!("{:?}", r);
        Json::from(r).into_response()
    }
}

pub fn router(app: Arc<App>) -> Router<()> {
    let contest: Router<_> = Router::new()
        .route("/tests", patch(contest::tests::update))
        // .route("/time", patch(contest::time::update))
        .layer(middleware::from_fn_with_state(app.clone(),auth_mw))
        .with_state(app.clone());
    let auth: Router<_> = Router::new()
        .route("/token", get(auth::token))
        .with_state(app.clone());
    Router::new()
        .nest("/contest", contest)
        .nest("/auth", auth)
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

