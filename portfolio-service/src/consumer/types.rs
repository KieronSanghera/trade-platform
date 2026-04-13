use crate::consumer::ConsumerError;
use shared::TradeExecuted;
use std::future::Future;
use std::pin::Pin;

// Handler returns a future which is a result or error.
type HandlerFutureResponse = Pin<Box<dyn Future<Output = Result<(), ConsumerError>> + Send>>;

// Handler is a function that takes a TradeExecuted and responds with the above
pub type Handler = Box<dyn Fn(TradeExecuted) -> HandlerFutureResponse + Send + Sync>;
