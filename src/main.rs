use crate::{
    air_models::APIFetcher,
    config::{load_config, AppConfig},
    kafka::{run_consumer, run_historical_producer, run_recent_producer},
    logging::setup_logging,
};
use clap::{Parser, Subcommand, ValueEnum};
use reqwest::Client;
mod air_models;
mod config;
mod kafka;
mod logging;
mod traits;
use tracing::info;

/// Kafka CLI App
#[derive(Parser)]
#[command(
    name = "Test App",
    version = "1.0",
    about = "Runs a Kafka producer or consumer"
)]
struct Cli {
    /// Kafka broker address, e.g., localhost:9092
    #[arg(short, long, default_value = "localhost:9092")]
    broker: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the Kafka producer
    Producer {
        #[arg(short, long, default_value = "recent")]
        mode: ProducerMode,
    },

    /// Run the Kafka consumer
    Consumer,
}

#[derive(ValueEnum, Clone)]
enum ProducerMode {
    Historical,
    Recent,
}

#[tokio::main]
async fn main() {
    let config: AppConfig = load_config().expect("Failed to load config");
    let fetcher = APIFetcher {
        client: Client::new(),
        latitude: config.location.latitude,
        longitude: config.location.longitude,
    };

    let cli = Cli::parse();
    setup_logging();

    match cli.command {
        Commands::Producer { mode } => match mode {
            ProducerMode::Historical => {
                info!(target: "producer", "Starting historical grab");
                run_historical_producer(&cli.broker, &fetcher).await;
            }
            ProducerMode::Recent => {
                info!(target: "producer", "Starting historical grab");
                run_recent_producer(&cli.broker, &fetcher).await;
            }
        },
        Commands::Consumer => {
            info!(target: "consumer", "Starting historical grab");
            run_consumer(&cli.broker, config.database.db_url.as_str()).await;
        }
    }
}
