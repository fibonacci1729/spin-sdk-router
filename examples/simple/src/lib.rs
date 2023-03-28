use spin_sdk::{http_component, http::{Request, Response}};

#[http_component]
fn handle_simple(req: Request) -> anyhow::Result<Response> {
    (spin_sdk_router::router! {
        GET "/hello/:planet" => api::h1,
        GET "/*" => api::h2,
        GET "/nested/*" => api::h3,
        GET "/hello/earth" => api::h4,
        GET "/" => |ctx| { todo!("WHAT") }
    })(req)
}

mod api {
    use spin_sdk_router::{Context, Response};

    // /hello/:planet
    pub fn h1(Context { request: _, params }: Context) -> anyhow::Result<Response> {
        let planet = params.get("planet").expect("PLANET");
        println!("{planet}");

        Ok(http::Response::builder()
            .status(http::StatusCode::OK)
            .body(None)
            .unwrap())
    }

    // /*
    pub fn h2(_ctx: Context) -> anyhow::Result<Response> { todo!("h2") }
    
    // /nested/*
    pub fn h3(_ctx: Context) -> anyhow::Result<Response> { todo!("h3") }
    
    // /hello/earth
    pub fn h4(_ctx: Context) -> anyhow::Result<Response> { todo!("h4") }
}