use clap::{Parser, Subcommand, ValueEnum};

use crate::{
    kafka::{run_consumer, run_historical_producer, run_recent_producer},
    logging::setup_logging,
};
mod config;
mod kafka;
mod logging;
mod model;
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
    let cli = Cli::parse();
    setup_logging();

    match cli.command {
        Commands::Producer { mode } => match mode {
            ProducerMode::Historical => {
                info!(target: "producer", "Starting historical grab");
                run_historical_producer(&cli.broker).await;
            }
            ProducerMode::Recent => {
                info!(target: "producer", "Starting historical grab");
                run_recent_producer(&cli.broker).await;
            }
        },
        Commands::Consumer => {
            info!(target: "consumer", "Starting historical grab");
            run_consumer(&cli.broker).await;
        }
    }
}
