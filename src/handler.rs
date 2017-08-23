use std::fmt;

use futures::{Future, IntoFuture};

use response::{BoxFutureResponse, Response};
use context::Context;
use StatusCode;

#[cfg_attr(feature = "cargo-clippy", allow(needless_pass_by_value, use_debug))]
pub(crate) fn default_catch<E: fmt::Debug + Send>(err: E) -> Response {
    // TODO: Support definable error catchers
    /*
        Idea of syntax:

        Shio::new().catch(|err: Error| { ... })
    */

    // Default "error catcher" just logs with error! and responds with a 500
    error!("{:?}", err);

    Response::with(StatusCode::InternalServerError)
}

pub trait Handler: Send + Sync
where
    <Self::Result as IntoFuture>::Error: fmt::Debug + Send,
{
    type Result: IntoFuture<Item = Response>;

    fn call(&self, context: Context) -> Self::Result;

    #[inline]
    fn boxed(self) -> BoxHandler
    where
        Self: Sized + 'static,
    {
        Box::new(move |ctx: Context| -> BoxFutureResponse {
            Box::new(self.call(ctx).into_future().or_else(default_catch))
        })
    }
}

#[cfg_attr(feature = "cargo-clippy", allow(stutter))]
pub type BoxHandler = Box<Handler<Result = BoxFutureResponse>>;

impl<TError, TFuture, TFn> Handler for TFn
where
    TError: fmt::Debug + Send,
    TFuture: IntoFuture<Item = Response, Error = TError>,
    TFn: Send + Sync,
    TFn: Fn(Context) -> TFuture,
{
    type Result = TFuture;

    #[inline]
    fn call(&self, context: Context) -> Self::Result {
        (*self)(context)
    }
}
