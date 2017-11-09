use jsonrpc_core::futures::future;
use jsonrpc_http_server::{self as http, hyper};

use txsched_ui;

#[derive(Default)]
pub struct Ui {
    ui: txsched_ui::Ui,
}

impl http::RequestMiddleware for Ui {
    fn on_request(&self, request: hyper::Request) -> http::RequestMiddlewareAction {
        if *request.method() == hyper::Method::Get {
            let path = if request.path() == "/" {
                "index.html"
            } else {
                &request.path()[1.. ]
            };
            let file = self.ui.files.get(path);
            if let Some(file) = file {
                return http::RequestMiddlewareAction::Respond {
                    should_validate_hosts: false,
                    response: Box::new(future::ok(hyper::Response::new()
                        .with_status(hyper::StatusCode::Ok)
                        .with_body(hyper::Body::from(file.content)))),
                }
            }
        }

        request.into()
    }
}
