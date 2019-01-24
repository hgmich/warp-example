use std::sync::{Arc, Mutex};

use my;
use my::prelude::*;
use my::Opts;

use futures::Future;

use crate::MyFutureConn;

pub(crate) fn create_connection_pool<O: Into<Opts>>(opts: O) -> Arc<Mutex<my::Pool>> {
    Arc::new(Mutex::new(my::Pool::new(opts)))
}

pub(crate) fn get_database_version(future_conn: impl MyFutureConn) -> impl my::MyFuture<String> {
    future_conn
        .and_then(|conn| conn.query("SELECT version()"))
        .and_then(|result| result.collect_and_drop())
        .map(|(_, rows): (my::Conn, Vec<String>)| {
            rows.get(0)
                .cloned()
                .unwrap_or_default()
        })
}
