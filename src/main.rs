use dotenvy::dotenv;
use hid_oauth_lib::types::GatewayConfig;
use hid_oauth_lib::{config, routes, state::AppState};
use std::sync::Arc;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing::{Level, debug, error, info};
use {hid_oauth_lib::management::db_based_config_integration, sqlx::PgPool, std::time::Duration};

#[allow(dead_code)]
const DEFAULT_CONFIG_PATH: &str = "config.yaml";
const DEFAULT_PORT: &str = "3000";
const DEFAULT_MANAGEMENT_PORT: &str = "8080";
const DEFAULT_DB_POLL_INTERVAL_SECONDS: u64 = 30;

const MAX_CONSECUTIVE_FAILURES: u32 = 5;
const MAX_BACKOFF_SECONDS: u64 = 300;

#[derive(Debug, Clone)]
pub enum ConfigMode {
    Yaml { path: String },
    Database { pool: PgPool },
}

type ConfigProvider =
    Arc<hid_oauth_lib::management::services::config_provider_service::ConfigProviderService>;

async fn determine_config_mode() -> anyhow::Result<ConfigMode> {
    match std::env::var("HID_OAUTH_MODE").as_deref() {
        Ok("database") => {
            debug!("HID_OAUTH_MODE=database detected. Initializing database mode.");
            let database_url = std::env::var("DATABASE_URL")
                .map_err(|e| anyhow::anyhow!("DATABASE_URL not set for database mode: {e}"))?;

            debug!("Connecting to database: {}", database_url);

            let pool = PgPool::connect(&database_url).await.map_err(|e| {
                anyhow::anyhow!("Failed to connect to database at {database_url}: {e}")
            })?;

            info!("Database connection established successfully.");
            Ok(ConfigMode::Database { pool })
        }
        Ok("yaml") => {
            debug!("HID_OAUTH_MODE=yaml detected. Using YAML configuration mode.");
            let config_path = std::env::var("CONFIG_FILE_PATH")
                .unwrap_or_else(|_| DEFAULT_CONFIG_PATH.to_string());
            Ok(ConfigMode::Yaml { path: config_path })
        }
        Ok(invalid_mode) => {
            error!(
                "Invalid HID_OAUTH_MODE '{}'. Valid options: 'yaml', 'database'",
                invalid_mode
            );
            Err(anyhow::anyhow!("Invalid HID_OAUTH_MODE: {invalid_mode}"))
        }
        Err(_) => {
            debug!("HID_OAUTH_MODE not set. Defaulting to YAML configuration mode.");
            let config_path = std::env::var("CONFIG_FILE_PATH")
                .unwrap_or_else(|_| DEFAULT_CONFIG_PATH.to_string());
            Ok(ConfigMode::Yaml { path: config_path })
        }
    }
}

async fn get_initial_config_and_services(
    mode: ConfigMode,
) -> anyhow::Result<(GatewayConfig, Option<axum::Router>, Option<ConfigProvider>)> {
    match mode {
        ConfigMode::Database { pool } => {
            debug!("Initializing database-based configuration.");

            let db_integration = db_based_config_integration(pool).await?;
            debug!("Database integration initialized successfully.");

            match db_integration.config_provider.fetch_live_config().await {
                Ok(initial_db_config) => {
                    info!("Successfully fetched initial configuration from database.");

                    if let Err(val_errors) =
                        config::validation::validate_gateway_config(&initial_db_config)
                    {
                        error!(
                            "Initial database configuration is invalid: {:?}. Application cannot start.",
                            val_errors
                        );
                        return Err(anyhow::anyhow!(
                            "Invalid initial database configuration: {val_errors:?}"
                        ));
                    }

                    info!(
                        "Initial database configuration validated successfully. Config has {} providers, {} models, {} pipelines.",
                        initial_db_config.providers.len(),
                        initial_db_config.models.len(),
                        initial_db_config.pipelines.len()
                    );

                    Ok((
                        initial_db_config,
                        Some(db_integration.router.clone()),
                        Some(db_integration.config_provider.clone()),
                    ))
                }
                Err(e) => {
                    error!(
                        "Failed to fetch initial configuration from database: {:?}. Application cannot start.",
                        e
                    );
                    Err(anyhow::anyhow!(
                        "Failed to fetch initial database config: {e}"
                    ))
                }
            }
        }
        ConfigMode::Yaml { path } => {
            debug!("Loading configuration from YAML file: {}", path);
            let yaml_config = config::load_config(&path).map_err(|e| {
                anyhow::anyhow!("Failed to load YAML configuration from {path}: {e}")
            })?;

            if let Err(val_errors) = config::validation::validate_gateway_config(&yaml_config) {
                error!(
                    "YAML configuration from {} is invalid: {:?}. Halting.",
                    path, val_errors
                );
                return Err(anyhow::anyhow!("Invalid YAML config: {val_errors:?}"));
            }
            debug!("YAML configuration validated successfully.");
            Ok((yaml_config, None, None))
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    
    let log_level = std::env::var("RUST_LOG")
        .ok()
        .and_then(|level| level.parse::<Level>().ok())
        .unwrap_or(Level::WARN);

    tracing_subscriber::fmt().with_max_level(log_level).init();

    info!("Starting Hid-OAuth Gateway...");

    let config_mode = determine_config_mode().await?;
    info!("Configuration mode determined: {:?}", config_mode);

    let (initial_config, management_router_opt, config_provider_opt) =
        get_initial_config_and_services(config_mode.clone()).await?;

    let app_state = Arc::new(
        AppState::new(initial_config)
            .map_err(|e| anyhow::anyhow!("Failed to create app state: {e}"))?,
    );

    let gateway_router = match &config_mode {
        ConfigMode::Database { pool } => {
            info!("Creating gateway router with OAuth authentication support");
            routes::create_router_with_auth(app_state.clone(), pool.clone())
        }
        ConfigMode::Yaml { .. } => {
            info!("Creating gateway router without authentication (YAML mode)");
            routes::create_router(app_state.clone())
        }
    };

    if let Some(config_provider) = config_provider_opt {
        let poller_app_state = app_state.clone();
        let poller_config_provider = config_provider.clone();

        let poll_interval_seconds = std::env::var("DB_POLL_INTERVAL_SECONDS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(DEFAULT_DB_POLL_INTERVAL_SECONDS);
        let poll_duration = Duration::from_secs(poll_interval_seconds);

        info!(
            "Starting database configuration poller with interval: {:?}.",
            poll_duration
        );
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(poll_duration);
            let mut consecutive_failures = 0u32;

            loop {
                interval.tick().await;
                debug!("Polling database for configuration updates...");

                match poller_config_provider.fetch_live_config().await {
                    Ok(new_config) => {
                        consecutive_failures = 0;

                        debug!("Successfully fetched configuration from database.");
                        debug!(
                            "Config has {} providers, {} models, {} pipelines",
                            new_config.providers.len(),
                            new_config.models.len(),
                            new_config.pipelines.len()
                        );

                        match poller_app_state.update_config(new_config) {
                            Ok(()) => {
                                debug!("Configuration update completed successfully.");
                            }
                            Err(update_err) => {
                                error!("Failed to apply updated configuration: {:?}", update_err);
                            }
                        }
                    }
                    Err(e) => {
                        consecutive_failures += 1;

                        if consecutive_failures <= MAX_CONSECUTIVE_FAILURES {
                            error!(
                                "Failed to fetch configuration from DB (attempt {}/{}): {:?}",
                                consecutive_failures, MAX_CONSECUTIVE_FAILURES, e
                            );
                        } else {
                            error!(
                                "Failed to fetch configuration from DB {} consecutive times. Will keep retrying but reducing log verbosity.",
                                consecutive_failures
                            );
                        }

                        if consecutive_failures > 3 {
                            let backoff_duration = std::cmp::min(
                                poll_duration * consecutive_failures,
                                Duration::from_secs(MAX_BACKOFF_SECONDS),
                            );
                            debug!("Applying backoff: {:?}", backoff_duration);
                            tokio::time::sleep(backoff_duration).await;
                        }
                    }
                }
            }
        });
    }

    let gateway_app = gateway_router.layer(
        TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::default().include_headers(true)),
    );

    let gateway_port = std::env::var("PORT").unwrap_or_else(|_| DEFAULT_PORT.to_string());
    let gateway_bind_address = format!("0.0.0.0:{gateway_port}");

    info!("Starting LLM Gateway server on {}", gateway_bind_address);
    let gateway_listener = tokio::net::TcpListener::bind(&gateway_bind_address)
        .await
        .map_err(|e| {
            anyhow::anyhow!("Failed to bind LLM Gateway to {gateway_bind_address}: {e}")
        })?;

    match management_router_opt {
        Some(management_router) => {
            let management_port = std::env::var("MANAGEMENT_PORT")
                .unwrap_or_else(|_| DEFAULT_MANAGEMENT_PORT.to_string());
            let management_bind_address = format!("0.0.0.0:{management_port}");

            info!(
                "Starting Management API server on {}",
                management_bind_address
            );
            let management_listener = tokio::net::TcpListener::bind(&management_bind_address)
                .await
                .map_err(|e| {
                    anyhow::anyhow!(
                        "Failed to bind Management API to {management_bind_address}: {e}"
                    )
                })?;

            let management_app = management_router.layer(
                TraceLayer::new_for_http()
                    .make_span_with(DefaultMakeSpan::default().include_headers(true)),
            );

            tokio::select! {
                res = axum::serve(gateway_listener, gateway_app) => {
                    if let Err(e) = res {
                        error!("LLM Gateway server failed: {}", e);
                    }
                },
                res = axum::serve(management_listener, management_app) => {
                    if let Err(e) = res {
                        error!("Management API server failed: {}", e);
                    }
                },
            }
        }
        None => {
            let app = gateway_app;
            axum::serve(gateway_listener, app)
                .await
                .map_err(|e| anyhow::anyhow!("LLM Gateway server failed: {e}"))?;
        }
    }

    Ok(())
}
