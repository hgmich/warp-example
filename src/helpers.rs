use futures::Future;
use hyper::{Body, Response};
use my::error::Error as MysqlError;

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

pub(crate) fn log_database_error(error: MysqlError) {
    match error {
        MysqlError::Driver(drv_err) => {
            error!("database driver error";
                "fault" => "database",
                "source" => "driver",
                "message" => format!("{}", drv_err),
                "object" => format!("{:?}", drv_err)
            );
        }
        MysqlError::Io(io_err) => {
            io_err
                .raw_os_error()
                .map(|errno| {
                    error!("database connection I/O error (system)";
                        "fault" => "database",
                        "source" => "io",
                        "message" => format!("{}", &io_err),
                        "object" => format!("{:?}", &io_err),
                        "io_err_kind" => format!("{:?}", io_err.kind()),
                        "errno" => errno
                    );
                })
                .or_else(|| {
                    error!("unknown database connection I/O error";
                        "fault" => "database",
                        "source" => "io",
                        "message" => format!("{}", io_err),
                        "object" => format!("{:?}", io_err)
                    );

                    None
                });
        }
        _ => error!("database error";
            "fault" => "database",
            "source" => "unknown",
            "message" => format!("{}", error),
            "object" => format!("{:?}", error)
        ),
    }
}
