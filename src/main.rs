use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    routing::get,
    Json, Router,
};
use gfp::{
    cache_map::ExpiringHashMap,
    grafana::{self, Grafana},
};
use serde::Serialize;
use std::{env, sync::Arc, time::Duration};
use tokio::sync::Mutex;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

struct AppState {
    grafana: Grafana,
    public_dashboards: ExpiringHashMap<String, String>,
}

#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .pretty()
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Setting default subscriber failed");

    let grafana_domain = env::var("GFP_GRAFANA_DOMAIN").unwrap_or_else(|_| {
        tracing::error!("GFP_GRAFANA_DOMAIN not set");
        panic!("GFP_GRAFANA_DOMAIN is required.");
    });
    let username = env::var("GFP_GRAFANA_USERNAME").unwrap_or_else(|_| {
        tracing::error!("GFP_GRAFANA_USERNAME not set");
        panic!("GFP_GRAFANA_USERNAME is required.");
    });
    let password = env::var("GFP_GRAFANA_PASSWORD").unwrap_or_else(|_| {
        tracing::error!("GFP_GRAFANA_PASSWORD not set");
        panic!("GFP_GRAFANA_PASSWORD is required.");
    });
    let port = env::var("GFP_PORT")
        .and_then(|value| {
            value.parse::<u16>().map_err(|_| {
                tracing::error!("GFP_PORT is not a valid number");
                panic!("GFP_PORT is not a valid number");
            })
        })
        .unwrap_or_else(|_| {
            tracing::warn!("GFP_PORT environment variable not set, using default");
            8080
        });
    let cache_duration_seconds = env::var("GFP_CACHE_SECONDS")
        .and_then(|value| {
            value.parse::<u64>().map_err(|_| {
                tracing::error!("GFP_CACHE_SECONDS is not a valid number");
                panic!("GFP_CACHE_SECONDS is not a valid number");
            })
        })
        .unwrap_or_else(|_| {
            tracing::warn!("GFP_CACHE_SECONDS environment variable not set, using default");
            300
        });

    let grafana = grafana::Grafana::new(grafana_domain, username, password);

    let shared_state = Arc::new(Mutex::new(AppState {
        grafana,
        public_dashboards: ExpiringHashMap::new(Duration::from_secs(cache_duration_seconds)),
    }));

    let app = Router::new()
        .route("/provision/:id", get(provision))
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .unwrap();

    tracing::info!("Listening on port {port}");

    axum::serve(listener, app).await.unwrap();
}

async fn provision(
    Path(dashboard_id): Path<String>,
    State(state): State<Arc<Mutex<AppState>>>,
) -> Result<Redirect, AppError> {
    tracing::info!("Received uid: {dashboard_id}");

    let mut state = state.lock().await;

    if let Some(url) = state.public_dashboards.get(&dashboard_id) {
        tracing::info!("Redirecting to {url} found in cache");
        return Ok(Redirect::to(url));
    }
    let grafana = &state.grafana;
    let url = grafana.public_url_of_dashboard(&dashboard_id).await?;
    state.public_dashboards.insert(dashboard_id, url.clone());

    tracing::info!("Redirecting to: {url}");
    Ok(Redirect::to(&url))
}

enum AppError {
    Network,
    Api,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        #[derive(Serialize)]
        struct ErrorResponse {
            message: String,
        }

        let (status, message) = match self {
            AppError::Network => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Network error".to_string(),
            ),
            AppError::Api => (StatusCode::INTERNAL_SERVER_ERROR, "API error".to_string()),
        };

        (status, Json(ErrorResponse { message })).into_response()
    }
}

impl From<grafana::Error> for AppError {
    fn from(err: grafana::Error) -> Self {
        match err {
            grafana::Error::Network => AppError::Network,
            grafana::Error::Api(_) => AppError::Api,
        }
    }
}
