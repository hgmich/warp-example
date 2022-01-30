#![deny(warnings)]

extern crate dotenv;
extern crate futures;
extern crate futures_locks;
extern crate hyper;
extern crate hyper_tls;
extern crate log;
#[macro_use]
extern crate slog;
extern crate slog_async;
#[macro_use]
extern crate slog_scope;
extern crate mysql_async as my;
extern crate serde;
extern crate slog_stdlog;
extern crate slog_term;
#[macro_use]
extern crate serde_json;
extern crate time;
extern crate tokio;
extern crate warp;

mod db;
mod errors;
mod helpers;
mod services;
mod types;

use std::env;

use std::sync::Arc;

pub(crate) use crate::errors::Error;
pub(crate) use crate::types::MyFutureConn;

use futures::Future;
use hyper::client::HttpConnector;
use hyper::header::{HeaderMap, HeaderValue};
use hyper::Client;
use hyper_tls::HttpsConnector;
use std::net::SocketAddr;
use warp::filters::BoxedFilter;
use warp::log::Info;
use warp::{Filter, Reply};

const SERVER_NAME: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

fn database_pool_injector_factory(
) -> BoxedFilter<(impl Future<Item = my::Conn, Error = my::error::Error>,)> {
    let pool =
        db::create_connection_pool(env::var("DATABASE_URL").expect("Must provide DATABASE_URL"));

    warp::any()
        .and_then(move || {
            pool.lock()
                .map(|pool| pool.get_conn())
                .map_err(|_| warp::reject::custom(Error::Database))
        })
        .boxed()
}

fn http_client_injector_factory() -> BoxedFilter<(Arc<Client<HttpsConnector<HttpConnector>>>,)> {
    let https = HttpsConnector::new(2).expect("Failed to init HTTPS connection backend");
    let http_client = Arc::new(Client::builder().build(https));

    warp::any().map(move || http_client.clone()).boxed()
}

fn router() -> BoxedFilter<(impl Reply,)> {
    let database_pool_injector = database_pool_injector_factory();
    let http_client_injector = http_client_injector_factory();

    let mysql_ver = warp::path("mysql_ver")
        .and(database_pool_injector.clone())
        .and_then(crate::services::util::mysql_ver_route);

    let health = warp::path("health").map(crate::services::util::health_check);

    let extern_http = warp::path("extern")
        .and(http_client_injector.clone())
        .and_then(crate::services::util::extern_http);

    let routes = warp::get2().and(mysql_ver.or(health).or(extern_http));

    routes.boxed()
}

fn global_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert("server", HeaderValue::from_static(SERVER_NAME));

    headers
}

fn setup_logging() -> slog_scope::GlobalLoggerGuard {
    use slog::Drain;

    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let log = slog::Logger::root(
        drain,
        slog_o!(
           "version" => env!("CARGO_PKG_VERSION"),
           "app" => env!("CARGO_PKG_NAME")
        ),
    );

    slog_scope::set_global_logger(log)
}

fn request_logger(info: Info) {
    info!("Handled request";
        "method" => info.method().as_str(),
        "path" => info.path(),
        "status" => info.status().as_u16()
    );
}

fn start(listen_addr: SocketAddr) {
    info!("Booting application";
        "started_at" => format!("{}", time::now().rfc3339()),
        "listen_addr" => format!("{}", &listen_addr)
    );

    let routes = router()
        .with(warp::reply::with::headers(global_headers()))
        .with(warp::filters::log::custom(request_logger));

    tokio::run(warp::serve(routes).bind(listen_addr));
}

fn main() {
    use std::net::ToSocketAddrs;

    dotenv::dotenv().expect("Failed to init dotenv");

    let _guard = setup_logging();
    let _stdlog_guard = slog_stdlog::init().unwrap();

    let listen_addr = env::var("LISTEN_HOST")
        .expect("Must provide LISTEN_HOST")
        .to_socket_addrs()
        .expect("Must provide valid HOST:PORT value for LISTEN_HOST")
        .next()
        .expect("Could not resolve hostname in LISTEN_HOST");

    start(listen_addr)
}
