use std::sync::Arc;

use tonic::{Request, Response, Status};
use tracing::{info, instrument};

use crate::{
    db::{error::DbError, postgres::PostgresDB},
    portfolio::{
        GetPortfolioRequest, GetPortfolioResponse, Position,
        portfolio_service_server::PortfolioService,
    },
};

pub struct GrpcPortfolioService {
    db: Arc<PostgresDB>,
}

#[tonic::async_trait]
impl PortfolioService for GrpcPortfolioService {
    #[instrument(skip(self, request))]
    async fn get_portfolio(
        &self,
        request: Request<GetPortfolioRequest>,
    ) -> Result<Response<GetPortfolioResponse>, Status> {
        let request: GetPortfolioRequest = request.into_inner();
        info!(
            user_id = %request.user_id,
            "Received portfolio request"
        );

        let portfolio = self
            .db
            .get_portfolio(&request.user_id)
            .await
            .map_err(|err| match err {
                DbError::SqlxError(_) => Status::internal("Internal SQL error"),
                _ => Status::internal("Database data error"),
            })?;

        let positions = portfolio
            .into_iter()
            .map(Position::from)
            .collect::<Vec<_>>();

        Ok(Response::new(GetPortfolioResponse {
            user_id: request.user_id,
            positions,
        }))
    }
}

impl GrpcPortfolioService {
    pub fn new(db: Arc<PostgresDB>) -> Self {
        info!("gRPC Portfolio Service created");
        Self { db }
    }
}
