use spin_sdk::{http_component, http::{Request, Response}};
use spin_sdk_router::Params;

#[http_component]
fn handle_simple(req: Request) -> anyhow::Result<Response> {
    // /hello/:planet
    fn hello_planet(_req: Request, params: Params) -> anyhow::Result<Response> {
        let planet = params.get("planet").expect("PLANET");

        Ok(http::Response::builder()
            .status(http::StatusCode::OK)
            .body(Some(format!("{planet}").into()))
            .unwrap())
    }

    // /nested/*
    fn nested_wildcard(_req: Request, params: Params) -> anyhow::Result<Response> {
        let capture = params.wildcard().unwrap_or_default();

        Ok(http::Response::builder()
            .status(http::StatusCode::OK)
            .body(Some(format!("{capture}").into()))
            .unwrap())
    }

    // /*
    fn wildcard(_req: Request, params: Params) -> anyhow::Result<Response> {
        let capture = params.wildcard().unwrap_or_default();
        Ok(http::Response::builder()
            .status(http::StatusCode::OK)
            .body(Some(format!("{capture}").into()))
            .unwrap())
    }

    // /hello/earth
    fn static_route(_req: Request, _params: Params) -> anyhow::Result<Response> {
        Ok(http::Response::builder()
            .status(http::StatusCode::OK)
            .body(None)
            .unwrap())
    }

    (spin_sdk_router::router! {
        GET "/hello/:planet" => hello_planet,
        GET "/nested/*" => nested_wildcard,
        GET "/hello/earth" => static_route,
        GET "/*" => wildcard
    })(req)
}