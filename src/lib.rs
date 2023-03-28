use routefinder::{Captures, Router as MethodRouter};
use std::collections::HashMap;

type Handler = dyn Fn(Request, Params) -> anyhow::Result<Response>;

pub type Response = http::Response<Option<bytes::Bytes>>;
pub type Request = http::Request<Option<bytes::Bytes>>;

pub type Params = Captures<'static, 'static>;

pub struct Router {
    method_map: HashMap<http::Method, MethodRouter<Box<Handler>>>,
    all_method_router: MethodRouter<Box<Handler>>,
}

struct Selection<'a> {
    pub params: Captures<'static, 'static>,
    pub handler: &'a Handler,
}

impl Router {
    pub fn dispatch(&self, request: Request) -> anyhow::Result<Response> {
        let method = request.method().to_owned();
        let path = request.uri().path().to_owned();
        let Selection { params, handler } = self.route(&path, method);
        handler(request, params)
    }

    fn route(&self, path: &str, method: http::Method) -> Selection<'_> {
        if let Some(m) = self
            .method_map
            .get(&method)
            .and_then(|r| r.best_match(path))
        {
            Selection {
                handler: m.handler(),
                params: m.captures().into_owned(),
            }
        } else if let Some(m) = self.all_method_router.best_match(path) {
            Selection {
                handler: m.handler(),
                params: m.captures().into_owned(),
            }
        } else if method == http::Method::HEAD {
            // If it is a HTTP HEAD request then check if there is a callback in the endpoints map
            // if not then fallback to the behavior of HTTP GET else proceed as usual
            self.route(path, http::Method::GET)
        } else if self
            .method_map
            .iter()
            .filter(|(k, _)| **k != method)
            .any(|(_, r)| r.best_match(path).is_some())
        {
            // If this `path` can be handled by a callback registered with a different HTTP method
            // should return 405 Method Not Allowed
            Selection {
                handler: &method_not_allowed,
                params: Captures::default(),
            }
        } else {
            Selection {
                handler: &not_found,
                params: Captures::default(),
            }
        }
    }

    pub fn add_all(&mut self, path: &str, handler: Box<Handler>) {
        self.all_method_router.add(path, handler).unwrap();
    }

    pub fn add(&mut self, path: &str, method: http::Method, handler: Box<Handler>) {
        self.method_map
            .entry(method)
            .or_insert_with(MethodRouter::new)
            .add(path, handler)
            .unwrap();   
    }

    pub fn new() -> Self {
        Router {
            method_map: HashMap::default(),
            all_method_router: MethodRouter::new(),
        }
    }
}

fn not_found(_req: Request, _params: Params) -> anyhow::Result<Response> {
    Ok(http::Response::builder()
        .status(http::StatusCode::NOT_FOUND)
        .body(None)
        .unwrap())
}

fn method_not_allowed(_req: Request, _params: Params) -> anyhow::Result<Response> {
    Ok(http::Response::builder()
        .status(http::StatusCode::METHOD_NOT_ALLOWED)
        .body(None)
        .unwrap())
}

#[macro_export]
macro_rules! router {
    ($($method:ident $path:literal => $h:path),*) => {
        {
            let mut router = spin_sdk_router::Router::new();
            $(
                router.add($path, http::Method::$method, Box::new($h));
            )*
            move |req: Request| -> anyhow::Result<Response> {
                router.dispatch(req)
            }
        }
    };
}