use std::sync::Arc;

use axum::{Router, http::StatusCode, routing::get};
use tokio::sync::RwLock;

use crate::health::state::{LivenessState, ReadinessState};

/// Create a simple HTTP health router
pub fn create_health_router(
    liveness: Arc<RwLock<LivenessState>>,
    readiness: Arc<RwLock<ReadinessState>>,
) -> Router {
    Router::new()
        .route("/livez", get(move || livez(liveness.clone())))
        .route("/readyz", get(move || readyz(readiness.clone())))
}

async fn readyz(readiness: Arc<RwLock<ReadinessState>>) -> StatusCode {
    if readiness.read().await.is_ready() {
        return StatusCode::OK;
    }
    StatusCode::SERVICE_UNAVAILABLE
}

async fn livez(liveness: Arc<RwLock<LivenessState>>) -> StatusCode {
    if liveness.read().await.is_live() {
        return StatusCode::OK;
    }
    StatusCode::SERVICE_UNAVAILABLE
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use tokio::sync::RwLock;

    fn live_liveness_probe() -> LivenessState {
        let mut state = LivenessState::default();
        state.mark_live();
        state
    }

    fn ready_readiness_probe() -> ReadinessState {
        let mut state = ReadinessState::default();
        state.mark_ready();
        state
    }

    #[tokio::test]
    async fn livez_returns_ok_when_live() {
        let state = Arc::new(RwLock::new(live_liveness_probe()));

        let status = livez(state).await;
        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn livez_returns_service_unavailable_when_not_live() {
        let state = Arc::new(RwLock::new(LivenessState::default()));
        let status = livez(state).await;
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn readyz_returns_ok_when_ready() {
        let state = Arc::new(RwLock::new(ready_readiness_probe()));
        let status = readyz(state).await;
        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn readyz_returns_service_unavailable_when_not_ready() {
        let state = Arc::new(RwLock::new(ReadinessState::default()));
        let status = readyz(state).await;
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    }
}
