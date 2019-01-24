use std::sync::Arc;

use my;

use futures::Future;
use hyper::client::HttpConnector;
use hyper::Client;
use hyper_tls::HttpsConnector;

pub(crate) trait FutureResponse<T>: Future<Item = T, Error = warp::Rejection> {}

impl<T, F> FutureResponse<T> for F where F: Future<Item = T, Error = warp::Rejection> {}

pub(crate) trait MyFutureConn: my::MyFuture<my::Conn> {}

impl<F> MyFutureConn for F where F: my::MyFuture<my::Conn> {}

pub(crate) type HttpClientRef = Arc<Client<HttpsConnector<HttpConnector>>>;
pub(crate) type BoxedFutureResponse<T> = Box<(Future<Item = T, Error = warp::Rejection> + Send)>;
