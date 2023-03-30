use routefinder::{Captures, Router as MethodRouter};
use std::collections::HashMap;

type Handler = dyn Fn(Request, Params) -> anyhow::Result<Response>;

pub type Response = http::Response<Option<bytes::Bytes>>;
pub type Request = http::Request<Option<bytes::Bytes>>;

pub type Params = Captures<'static, 'static>;

pub struct Router {
    methods_map: HashMap<http::Method, MethodRouter<Box<Handler>>>,
    all_methods: MethodRouter<Box<Handler>>,
}

struct RouteMatch<'a> {
    params: Captures<'static, 'static>,
    handler: &'a Handler,
}

impl Router {
    pub fn handle(&self, request: Request) -> anyhow::Result<Response> {
        let method = request.method().to_owned();
        let path = request.uri().path().to_owned();
        let RouteMatch { params, handler } = self.find(&path, method);
        handler(request, params)
    }

    fn find(&self, path: &str, method: http::Method) -> RouteMatch<'_> {
        let best_match = self
            .methods_map
            .get(&method)
            .and_then(|r| r.best_match(path));

        if let Some(m) = best_match {
            let params = m.captures().into_owned();
            let handler = m.handler();
            return RouteMatch { handler, params };
        }

        let best_match = self.all_methods.best_match(path);

        match best_match {
            Some(m) => {
                let params = m.captures().into_owned();
                let handler = m.handler();
                RouteMatch { handler, params }
            }
            None if method == http::Method::HEAD => {
                // If it is a HTTP HEAD request then check if there is a callback in the methods map
                // if not then fallback to the behavior of HTTP GET else proceed as usual
                self.find(path, http::Method::GET)
            }
            None => {
                let not_allowed = self
                    .methods_map
                    .iter()
                    .filter(|(k, _)| **k != method)
                    .any(|(_, r)| r.best_match(path).is_some());

                if not_allowed {
                    // If this `path` can be handled by a callback registered with a different HTTP method
                    // should return 405 Method Not Allowed
                    RouteMatch {
                        handler: &method_not_allowed,
                        params: Captures::default(),
                    }
                } else {
                    RouteMatch {
                        handler: &not_found,
                        params: Captures::default(),
                    }
                }
            }
        }
    }

    pub fn all<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(Request, Params) -> anyhow::Result<Response> + 'static,
    {
        self.all_methods.add(path, Box::new(handler)).unwrap();
    }

    pub fn add<F>(&mut self, path: &str, method: http::Method, handler: F)
    where
        F: Fn(Request, Params) -> anyhow::Result<Response> + 'static,
    {
        self.methods_map
            .entry(method)
            .or_insert_with(MethodRouter::new)
            .add(path, Box::new(handler))
            .unwrap();
    }

    pub fn get<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(Request, Params) -> anyhow::Result<Response> + 'static,
    {
        self.add(path, http::Method::GET, handler)
    }

    pub fn post<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(Request, Params) -> anyhow::Result<Response> + 'static,
    {
        self.add(path, http::Method::POST, handler)
    }

    pub fn delete<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(Request, Params) -> anyhow::Result<Response> + 'static,
    {
        self.add(path, http::Method::DELETE, handler)
    }

    pub fn put<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(Request, Params) -> anyhow::Result<Response> + 'static,
    {
        self.add(path, http::Method::PUT, handler)
    }

    pub fn patch<F>(&mut self, path: &str, handler: F)
    where
        F: Fn(Request, Params) -> anyhow::Result<Response> + 'static,
    {
        self.add(path, http::Method::PATCH, handler)
    }

    pub fn new() -> Self {
        Router {
            methods_map: HashMap::default(),
            all_methods: MethodRouter::new(),
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
    ($($method:tt $path:literal => $h:expr),*) => {
        {
            let mut router = spin_sdk_router::Router::new();
            $(
                spin_sdk_router::router!(@build router $method $path => $h);
            )*
            move |req: Request| -> anyhow::Result<Response> {
                router.handle(req)
            }
        }
    };
    (@build $r:ident HEAD $path:literal => $h:expr) => {
        $r.add($path, http::Method::HEAD, $h);
    };
    (@build $r:ident GET $path:literal => $h:expr) => {
        $r.add($path, http::Method::GET, $h);
    };
    (@build $r:ident PUT $path:literal => $h:expr) => {
        $r.add($path, http::Method::PUT, $h);
    };
    (@build $r:ident POST $path:literal => $h:expr) => {
        $r.add($path, http::Method::POST, $h);
    };
    (@build $r:ident PATCH $path:literal => $h:expr) => {
        $r.add($path, http::Method::PATCH, $h);
    };
    (@build $r:ident DELETE $path:literal => $h:expr) => {
        $r.add($path, http::Method::DELETE, $h);
    };
    (@build $r:ident POST $path:literal => $h:expr) => {
        $r.add($path, http::Method::OPTIONS, $h);
    };
    (@build $r:ident _ $path:literal => $h:expr) => {
        $r.all($path, $h);
    };
}

#[cfg(test)]
mod tests {}
