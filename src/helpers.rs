use futures::Future;
use hyper::{Body, Response};

pub(crate) fn response_to_json(
    json: impl Future<Item = Response<Body>, Error = warp::Rejection>,
) -> impl Future<Item = serde_json::Value, Error = warp::Rejection> {
    json.and_then(move |res| {
        use futures::Stream;

        res.into_body()
            .concat2()
            .map_err(|e| {
                error!("Error writing response to buffer: {}", e);
                warp::reject::custom(crate::Error::HttpExtern)
            })
            .and_then(|chunk| {
                serde_json::de::from_slice(chunk.as_ref()).map_err(|e| {
                    error!("Error parsing JSON from remote host: {}", e);
                    warp::reject::custom(crate::Error::JsonDecode)
                })
            })
    })
}
