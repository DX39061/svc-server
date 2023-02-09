use crate::config::Config;
use crate::service::clone::clone;
use crate::service::pull::pull;
use crate::service::push::push;
use crate::util::{empty, full};
use bytes::Bytes;
use http_body_util::combinators::BoxBody;
use hyper::header::{HeaderValue, UPGRADE};
use hyper::upgrade::Upgraded;
use hyper::{Request, Response, StatusCode};

mod clone;
mod pull;
mod push;

pub async fn preprocess(
    mut req: Request<hyper::body::Incoming>,
) -> Result<Response<BoxBody<Bytes, hyper::Error>>, hyper::Error> {
    // check url path
    let path = req.uri().path().to_string();
    if !["/clone", "/push", "/pull"].contains(&&*path) {
        let mut resp = Response::new(empty());
        *resp.status_mut() = StatusCode::NOT_FOUND;
        return Ok(resp);
    }

    // validate token
    let config = Config::load().expect("failed to load config");
    if let Some(token) = req.headers().get("Authorization") {
        if token.to_str().unwrap_or("") != config.server.token {
            let mut resp = Response::new(full("error: token not match"));
            *resp.status_mut() = StatusCode::FORBIDDEN;
            return Ok(resp);
        }
    } else {
        let mut resp = Response::new(full("error: token needed"));
        *resp.status_mut() = StatusCode::BAD_REQUEST;
        return Ok(resp);
    }

    // check upgrade flag
    if !req.headers().contains_key(UPGRADE) {
        let mut resp = Response::new(empty());
        *resp.status_mut() = StatusCode::BAD_REQUEST;
        return Ok(resp);
    }

    // Setup a future that will eventually receive the upgraded
    // connection and talk a new protocol, and spawn the future
    // into the runtime.
    //
    // Note: This can't possibly be fulfilled until the 101 response
    // is returned below, so it's better to spawn this future instead
    // waiting for it to complete to then return a response.
    tokio::spawn(async move {
        match hyper::upgrade::on(&mut req).await {
            Ok(upgraded_conn) => {
                if let Err(err) = route(upgraded_conn, &path).await {
                    eprintln!("server io error: {}", err);
                }
            }
            Err(err) => {
                eprintln!("upgrade error: {}", err);
            }
        }
    });
    let mut resp = Response::new(empty());
    *resp.status_mut() = StatusCode::SWITCHING_PROTOCOLS;
    resp.headers_mut()
        .insert(UPGRADE, HeaderValue::from_static("websocket"));
    Ok(resp)
}

async fn route(mut upgraded_conn: Upgraded, path: &str) -> Result<(), &'static str> {
    match path {
        "/clone" => clone(&mut upgraded_conn)?,
        "/push" => push(&mut upgraded_conn)?,
        "/pull" => pull(&mut upgraded_conn)?,
        _ => (),
    }
    Ok(())
}
