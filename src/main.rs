use std::sync::Arc;

use axum::Router;
use backend_rust::{
    config::Settings,
    database::{postgres::create_pool, redis::create_redis_client},
    middleware::{cors::build_cors, logging::build_trace},
    routes::api::build_api_router,
    services::{ai_client::AiClient, auth_service::AuthService, file_storage::StorageProvider},
    solana::{client::SolanaClient, token::TokenService},
    state::AppState,
};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("info,backend_rust=debug")
        .init();

    let settings = Settings::from_env()?;

    // -----------------------------------------------------------------------
    // Database — connect and run pending migrations in background.
    // We run migrations in a detached task so the HTTP server (and `/health`)
    // becomes available even if the DB is temporarily unreachable during
    // deployment. Migration failures are logged but do not crash the process.
    // -----------------------------------------------------------------------
    let db = create_pool(&settings).await?;

    let db_for_migrate = db.clone();
    tokio::spawn(async move {
        tracing::info!("Running database migrations in background with retry...");
        let mut attempt: u32 = 0;
        let max_attempts: u32 = 10;
        loop {
            attempt += 1;
            match sqlx::migrate!("./migrations").run(&db_for_migrate).await {
                Ok(_) => {
                    tracing::info!("Migrations applied successfully on attempt {}.", attempt);
                    break;
                }
                Err(e) => {
                    if attempt >= max_attempts {
                        tracing::error!("Database migration failed after {} attempts: {:?}. Giving up.", attempt, e);
                        break;
                    } else {
                        let backoff = std::time::Duration::from_secs(5 * attempt as u64);
                        tracing::warn!("Migration attempt {} failed: {:?}. Retrying in {}s...", attempt, e, backoff.as_secs());
                        tokio::time::sleep(backoff).await;
                    }
                }
            }
        }
    });

    // -----------------------------------------------------------------------
    // Other services
    // -----------------------------------------------------------------------
    let redis = match create_redis_client(&settings).await {
        Ok(r) => {
            Some(r)
        }
        Err(e) => {
            tracing::warn!(
                "Redis unavailable ({}). Rate limiting disabled. \
                 Start Redis with: sudo service redis-server start",
                e
            );
            None
        }
    };
    let auth = AuthService::new(&settings);
    let ai_client = AiClient::new(&settings);
    let storage = StorageProvider::from_settings(&settings);

    // -----------------------------------------------------------------------
    // Solana — optional. Falls back to DB-only mode when keys are placeholders.
    // -----------------------------------------------------------------------
    let solana = if is_solana_configured(&settings) {
        match SolanaClient::from_settings(&settings) {
            Ok(client) => {
                tracing::info!(
                    "Solana client initialised. RPC={} Mint={}",
                    settings.solana_rpc_url,
                    settings.solana_token_mint_address
                );
                Some(TokenService::new(client))
            }
            Err(e) => {
                tracing::warn!(
                    "Solana client failed to initialise ({}). Running in DB-only mode.",
                    e
                );
                None
            }
        }
    } else {
        tracing::info!(
            "Solana keys are not configured. Running in DB-only mode. \
             Set SOLANA_WALLET_PRIVATE_KEY and SOLANA_TOKEN_MINT_ADDRESS to enable on-chain transactions."
        );
        None
    };

    let state = Arc::new(AppState {
        settings: settings.clone(),
        db,
        redis,
        auth,
        ai_client,
        storage,
        solana,
    });

    let app: Router = build_api_router(state)
        .layer(build_cors(&settings))
        .layer(build_trace());

    let listener = TcpListener::bind(format!(
        "0.0.0.0:{}",
        std::env::var("PORT").unwrap_or_else(|_| "3000".to_string())
    ))
    .await?;
    tracing::info!("backend-rust listening on port {}", listener.local_addr()?.port());
    axum::serve(listener, app).await?;
    Ok(())
}

/// Returns true only when the Solana env vars look like real values
/// (not the placeholder strings from .env.example).
fn is_solana_configured(settings: &Settings) -> bool {
    let key = &settings.solana_wallet_private_key;
    let mint = &settings.solana_token_mint_address;

    !key.is_empty()
        && key != "your_private_key_base58"
        && !mint.is_empty()
        && mint != "your_token_mint_address"
}
