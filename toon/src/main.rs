use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use serde::Deserialize;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(name = "toon", author, version, about = "Local driver for the OtterSlice DEX bot")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Override the default logging filter (info)
    #[arg(long, global = true, default_value = "info")]
    log: String,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run the bot in live, paper or replayed mode
    Run {
        /// Execution mode
        #[arg(long, value_enum, default_value_t = RunMode::Paper)]
        mode: RunMode,
        /// Path to the runtime configuration file
        #[arg(long, default_value = "config/default.toml")]
        config: PathBuf,
        /// Path to the markets configuration file
        #[arg(long, default_value = "config/markets.toml")]
        markets: PathBuf,
    },
    /// Replay previously captured data for analysis
    Replay {
        /// Path to a parquet capture file
        #[arg(long)]
        parquet: PathBuf,
        /// Path to the runtime configuration file
        #[arg(long, default_value = "config/default.toml")]
        config: PathBuf,
    },
}

#[derive(Copy, Clone, Debug, ValueEnum, Eq, PartialEq)]
enum RunMode {
    Live,
    Paper,
    Replay,
}

#[derive(Debug, Deserialize)]
struct RuntimeConfig {
    #[serde(default)]
    cluster: ClusterConfig,
    #[serde(default)]
    accounts: AccountConfig,
}

#[derive(Debug, Deserialize, Default)]
struct ClusterConfig {
    #[serde(default = "default_rpc_url")]
    rpc_url: String,
    #[serde(default = "default_ws_url")]
    ws_url: String,
    #[serde(default)]
    compute_unit_price_micro_lamports: u64,
}

#[derive(Debug, Deserialize, Default)]
struct AccountConfig {
    #[serde(default)]
    authority: Option<String>,
    #[serde(default)]
    payer: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MarketsConfig {
    #[serde(default)]
    phoenix: Vec<BookConfig>,
    #[serde(default)]
    openbook: Vec<BookConfig>,
}

#[derive(Debug, Deserialize)]
struct BookConfig {
    name: String,
    base_mint: String,
    quote_mint: String,
    #[serde(default)]
    min_spread_bps: Option<f64>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(cli.log.clone()))
        .with_target(false)
        .init();

    match cli.command {
        Commands::Run {
            mode,
            config,
            markets,
        } => run(mode, &config, &markets),
        Commands::Replay { parquet, config } => replay(&parquet, &config),
    }
}

fn run(mode: RunMode, config_path: &Path, markets_path: &Path) -> Result<()> {
    let runtime = load_runtime_config(config_path)?;
    let markets = load_markets_config(markets_path)?;

    info!(mode = ?mode, rpc = %runtime.cluster.rpc_url, "starting toon runtime");
    info!(phoenix = %markets.phoenix.len(), openbook = %markets.openbook.len(), "configured orderbooks");

    if mode == RunMode::Live {
        warn!("live trading mode is not yet implemented; falling back to paper scaffolding");
    }

    info!("paper engine bootstrapped – waiting for strategy wiring");
    Ok(())
}

fn replay(parquet_path: &Path, config_path: &Path) -> Result<()> {
    let runtime = load_runtime_config(config_path)?;
    info!(parquet = %parquet_path.display(), rpc = %runtime.cluster.rpc_url, "starting replay session");
    info!("replay pipeline is not yet implemented but configuration loading succeeded");
    Ok(())
}

fn load_runtime_config(path: &Path) -> Result<RuntimeConfig> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read runtime config at {}", path.display()))?;
    let config: RuntimeConfig = toml::from_str(&raw)
        .with_context(|| format!("failed to parse runtime config at {}", path.display()))?;
    Ok(config)
}

fn load_markets_config(path: &Path) -> Result<MarketsConfig> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read markets config at {}", path.display()))?;
    let config: MarketsConfig = toml::from_str(&raw)
        .with_context(|| format!("failed to parse markets config at {}", path.display()))?;
    Ok(config)
}

fn default_rpc_url() -> String {
    "http://localhost:8899".to_owned()
}

fn default_ws_url() -> String {
    "ws://localhost:8900".to_owned()
}
