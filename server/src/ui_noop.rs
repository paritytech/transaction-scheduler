use jsonrpc_http_server as http;
use jsonrpc_http_server::hyper;

#[derive(Default)]
pub struct Ui;

impl http::RequestMiddleware for Ui {
    fn on_request(&self, request: hyper::Request) -> http::RequestMiddlewareAction {
        request.into()
    }
}
