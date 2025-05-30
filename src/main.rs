use clap::{Parser, Subcommand};

use crate::kafka::{run_consumer, run_producer};
mod config;
mod kafka;
mod model;

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
    Producer,

    /// Run the Kafka consumer
    Consumer,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Producer => {
            run_producer(&cli.broker).await;
        }
        Commands::Consumer => {
            run_consumer(&cli.broker).await;
        }
    }
}
