use crate::consumer::ConsumerError;
use shared::TradeExecuted;
use std::future::Future;
use std::pin::Pin;

type HandlerFutureResponse = Pin<Box<dyn Future<Output = Result<(), ConsumerError>> + Send>>;

pub type Handler = Box<dyn Fn(TradeExecuted) -> HandlerFutureResponse + Send + Sync>;
