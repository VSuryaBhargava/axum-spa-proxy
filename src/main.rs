use axum::body::Body;
use axum::extract::{Path, Query, Request, State};

use axum::response::Response;
use axum::routing::get_service;
use axum::Json;
use axum::{response::IntoResponse, routing::get, Router};
use axum_server::tls_rustls::RustlsConfig;
use hyper::Version;
use hyper::{body::Incoming, StatusCode, Uri};
use hyper_tls::HttpsConnector;
use hyper_util::{client::legacy::Client as HyperClient, rt::TokioExecutor};
use serde_json::json;
use std::net::SocketAddr;
use tower_http::services::{ServeDir, ServeFile};

mod args;
mod config;

// ❯ cargo watch -q -c -w src/ -x run

#[derive(serde::Deserialize, Debug)]
struct QueryParams {
    username: Option<String>,
}

type HttpsClient = HyperClient<
    HttpsConnector<hyper_util::client::legacy::connect::HttpConnector>,
    axum::body::Body,
>;

#[derive(Clone)]
struct ProxyState {
    https_client: HttpsClient,
}

#[derive(serde::Deserialize, Debug)]
struct PathParams {
    username: String,
    age: Option<u32>,
}

#[tokio::main]
async fn main() {
    let args = args::Args::new();
    let config_path_file_path = &args.config;
    let server_config = config::Config::from_file(config_path_file_path);

    let state = ProxyState {
        https_client: HyperClient::builder(TokioExecutor::new()).build(HttpsConnector::new()),
    };

    let mut hello_route = Router::new()
        .route("/user/:username", get(greet_user))
        .route("/user/:username/age/:age", get(greet_user));

    if let Some(proxies) = &server_config.proxies {
        for proxy in proxies {
            let target = proxy.target.to_owned();
            let log_requests = args.log_requests;
            let log_responses = args.log_responses;

            let handler = move |s: State<ProxyState>, req: Request<Body>| {
                proxy_handler(target, log_requests, log_responses, s, req)
            };

            hello_route = hello_route.route(
                &proxy.route,
                get(handler.clone())
                    .post(handler.clone())
                    .delete(handler.clone())
                    .put(handler.clone()),
            );
        }
    }

    if let Some(file_server) = &server_config.file_server {
        hello_route = hello_route.fallback_service(
            Router::new().nest_service(
                file_server.route_path.as_str(),
                get_service(
                    ServeDir::new(&file_server.file_path)
                        .not_found_service(ServeFile::new(&file_server.fallback_file)),
                ),
            ),
        );
    }

    let addr = SocketAddr::from(([0, 0, 0, 0], server_config.port));

    println!(
        "Starting server on port {}://localhost:{}/",
        if server_config.https_config.is_some() {
            "https"
        } else {
            "http"
        },
        server_config.port
    );

    match server_config.https_config {
        Some(ref https_config) => {
            let cert_and_key_files =
                RustlsConfig::from_pem_file(&https_config.cert, &https_config.key)
                    .await
                    .expect("unable to read cert and key files");
            axum_server::bind_rustls(addr, cert_and_key_files)
                .serve(hello_route.with_state(state).into_make_service())
                .await
                .unwrap_or_else(|_| {
                    panic!("Unable to start server on port {}", server_config.port)
                });
        }
        None => axum_server::bind(addr)
            .serve(hello_route.with_state(state).into_make_service())
            .await
            .unwrap_or_else(|_| panic!("Unable to start server on port {}", server_config.port)),
    };
}

async fn proxy_handler(
    target: String,
    log_requests: bool,
    log_responses: bool,
    State(state): State<ProxyState>,
    mut req: Request<Body>,
) -> Result<Response<Incoming>, StatusCode> {
    let path_query = req
        .uri()
        .path_and_query()
        .map_or_else(|| req.uri().path(), |v| v.as_str());

    let uri = format!("{target}{path_query}");

    *req.uri_mut() = Uri::try_from(uri).unwrap();
    *req.version_mut() = Version::HTTP_11;

    if log_requests {
        println!("Request: {:?}", req);
    }

    match state.https_client.request(req).await {
        Ok(res) => {
            if log_responses {
                println!("Response: {:?}", res);
            }
            Ok(res)
        }
        Err(e) => {
            println!("error: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn greet_user(
    Path(path_params): Path<PathParams>,
    Query(query_params): Query<QueryParams>,
) -> impl IntoResponse {
    Json(
        json!({ "name": path_params.username, "age": path_params.age, "query": match query_params.username {
            Some(username) => username,
            None => "no username".to_string()
        }}),
    )
}
