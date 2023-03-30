use spin_sdk::{
    http::{Request, Response},
    http_component,
};
use spin_sdk_router::{Params, Router};

#[http_component]
fn handle_example(req: Request) -> anyhow::Result<Response> {
    let mut router = Router::new();
    router.get("/hello/:planet", api::hello_planet);
    router.add_all("/*", |_req, params| {
        let capture = params.wildcard().unwrap_or_default();
        Ok(http::Response::builder()
            .status(http::StatusCode::OK)
            .body(Some(format!("{capture}").into()))
            .unwrap())
    });

    router.handle(req)
}

mod api {
    use super::*;

    // /hello/:planet
    pub fn hello_planet(_req: Request, params: Params) -> anyhow::Result<Response> {
        let planet = params.get("planet").expect("PLANET");

        Ok(http::Response::builder()
            .status(http::StatusCode::OK)
            .body(Some(format!("{planet}").into()))
            .unwrap())
    }
}

