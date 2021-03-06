use cubestore::mysql::MySqlServer;
use cubestore::config::Config;
use simple_logger::SimpleLogger;
use log::Level;
use std::env;
use tokio::runtime::Builder;
use log::{debug};
use cubestore::telemetry::{track_event, ReportingLogger};
use std::collections::HashMap;

fn main() {
    let log_level = match env::var("CUBESTORE_LOG_LEVEL").unwrap_or("info".to_string()).to_lowercase().as_str() {
        "error" => Level::Error,
        "warn" => Level::Warn,
        "info" => Level::Info,
        "debug" => Level::Debug,
        "trace" => Level::Trace,
        x => panic!("Unrecognized log level: {}", x)
    };

    let logger = SimpleLogger::new()
        .with_level(Level::Error.to_level_filter())
        .with_module_level("cubestore", log_level.to_level_filter());
    ReportingLogger::init(Box::new(logger), log_level.to_level_filter()).unwrap();

    let mut runtime = Builder::new()
        .enable_all()
        .threaded_scheduler()
        .build()
        .unwrap();


    let config = Config::default();

    config.configure_worker();

    debug!("New process started");

    runtime.enter(|| {
        procspawn::init();
    });

    runtime.block_on(async move {
        let services = config.configure().await;
        services.start_processing_loops().await.unwrap();

        track_event("Cubestore Start".to_string(), HashMap::new()).await;

        MySqlServer::listen("0.0.0.0:3306".to_string(), services.sql_service.clone()).await.unwrap();
    });
}
