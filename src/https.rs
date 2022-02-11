use actix_web::{
    body::BoxBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    http::header,
    Error, HttpResponse,
};
use futures::future::{ok, Either, Ready};

pub struct Https {}

impl Https {
    pub fn new() -> Self {
        Https {}
    }
}

impl<S> Transform<S, ServiceRequest> for Https
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;

    type Error = Error;

    type Transform = HttpsMiddleware<S>;

    type InitError = ();

    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(HttpsMiddleware { service })
    }
}

pub struct HttpsMiddleware<S> {
    service: S,
}

impl<S> Service<ServiceRequest> for HttpsMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;

    type Error = Error;

    type Future = Either<S::Future, Ready<Result<ServiceResponse<BoxBody>, Self::Error>>>;

    fn poll_ready(
        &self,
        ctx: &mut core::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        if req.connection_info().scheme() == "https" {
            Either::Left(self.service.call(req))
        } else {
            let host = req.connection_info().host().to_owned();
            let uri = req.uri().to_owned();

            Either::Right(ok(req.into_response(
                HttpResponse::PermanentRedirect()
                    .append_header((header::LOCATION, format!("https://{}{}", host, uri)))
                    .finish(),
            )))
        }
    }
}
