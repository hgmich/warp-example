use futures::Future;

use crate::helpers::response_to_json;
use crate::types::{BoxedFutureResponse, HttpClientRef};
use crate::{Error, MyFutureConn};

pub(crate) fn mysql_ver_route(conn: impl MyFutureConn) -> BoxedFutureResponse<(impl warp::Reply)> {
    Box::new(
        crate::db::get_database_version(conn)
            .map(|ver| warp::reply::json(&json!({ "version": ver })))
            .map_err(|db_err| {
                error!("database error: {}", db_err);
                warp::reject::custom(Error::Database)
            }),
    )
}

pub(crate) fn health_check() -> impl warp::Reply {
    warp::reply::json(&json!({"status": "alive"}))
}

pub(crate) fn extern_http(http_client: HttpClientRef) -> BoxedFutureResponse<(impl warp::Reply)> {
    let uri = "https://jsonplaceholder.typicode.com/todos/1"
        .parse()
        .expect("URI should be valid!");

    Box::new(
        response_to_json(http_client.get(uri).map_err(|e| {
            error!("HTTP error: {}", e);
            warp::reject::custom(Error::HttpExtern)
        }))
        .map(|v| warp::reply::json(&v)),
    )
}
