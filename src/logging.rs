use std::fs::{self, File};
use std::path::Path;
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::{fmt, prelude::*};

pub fn setup_logging() {
    let log_dir = Path::new("logs");
    if !log_dir.exists() {
        fs::create_dir_all(log_dir).expect("Failed to create logs directory");
    }

    let producer_file =
        File::create(log_dir.join("producer.log")).expect("Failed to create producer log file");
    let consumer_file =
        File::create(log_dir.join("consumer.log")).expect("Failed to create consumer log file");

    let producer_layer = fmt::layer()
        .with_writer(producer_file)
        .with_ansi(false)
        .with_filter(EnvFilter::new("producer=info"));

    let consumer_layer = fmt::layer()
        .with_writer(consumer_file)
        .with_ansi(false)
        .with_filter(EnvFilter::new("consumer=info"));

    let stdout_layer = fmt::layer()
        .with_ansi(true)
        .with_writer(std::io::stdout)
        .with_filter(EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(producer_layer)
        .with(consumer_layer)
        .with(stdout_layer)
        .init();
}
