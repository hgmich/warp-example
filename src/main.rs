#![deny(warnings)]

extern crate dotenv;
extern crate futures;
extern crate futures_locks;
extern crate hyper;
extern crate hyper_tls;
#[macro_use]
extern crate log;
extern crate mysql_async as my;
extern crate pretty_env_logger;
extern crate serde;
#[macro_use]
extern crate serde_json;
extern crate tokio;
extern crate warp;

mod db;
mod errors;
mod services;
mod types;
mod helpers;

use std::env;

use std::sync::Arc;

pub(crate) use crate::errors::Error;
pub(crate) use crate::types::MyFutureConn;

use futures::Future;
use hyper::client::HttpConnector;
use hyper::header::{HeaderMap, HeaderValue};
use hyper::Client;
use hyper_tls::HttpsConnector;
use warp::filters::BoxedFilter;
use warp::{Filter, Reply};

const SERVER_NAME: &str = concat!(
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION")
);

fn database_pool_injector_factory(
) -> BoxedFilter<(impl Future<Item = my::Conn, Error = my::errors::Error>,)> {
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
    use warp::Filter;

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

fn main() {
    use std::net::ToSocketAddrs;

    dotenv::dotenv().expect("Failed to init dotenv");

    pretty_env_logger::init();

    let routes = router()
        .with(warp::reply::with::headers(global_headers()));

    tokio::run(
        warp::serve(routes).bind(
            env::var("LISTEN_HOST")
                .expect("Must provide LISTEN_HOST")
                .to_socket_addrs()
                .expect("Must provide valid HOST:PORT value for LISTEN_HOST")
                .next()
                .expect("Could not resolve hostname in LISTEN_HOST"),
        ),
    );
}
