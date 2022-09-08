use std::io::ErrorKind;

use clap::{builder::ValueParser, Parser, Subcommand};
use maplibre::{coords::LatLon, platform::run_multithreaded};

use crate::{headed::run_headed, headless::run_headless};

mod headed;
mod headless;

#[cfg(feature = "trace")]
fn enable_tracing() {
    use tracing_subscriber::{layer::SubscriberExt, Registry};

    let subscriber = Registry::default().with(tracing_tracy::TracyLayer::new());

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
}

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

fn parse_lat_long(env: &str) -> Result<LatLon, std::io::Error> {
    let split = env.split(',').collect::<Vec<_>>();
    if let (Some(latitude), Some(longitude)) = (split.get(0), split.get(1)) {
        Ok(LatLon::new(
            latitude.parse::<f64>().unwrap(),
            longitude.parse::<f64>().unwrap(),
        ))
    } else {
        Err(std::io::Error::new(
            ErrorKind::InvalidData,
            "Failed to parse latitude and longitude.",
        ))
    }
}

#[derive(Subcommand)]
enum Commands {
    Headed {},
    Headless {
        #[clap(default_value_t = 400)]
        tile_size: u32,
        #[clap(
            value_parser = ValueParser::new(parse_lat_long),
            default_value_t = LatLon::new(48.0345697188, 11.3475219363)
        )]
        min: LatLon,
        #[clap(
            value_parser = ValueParser::new(parse_lat_long),
            default_value_t = LatLon::new(48.255861, 11.7917815798)
        )]
        max: LatLon,
    },
}

fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    #[cfg(feature = "trace")]
    enable_tracing();

    let cli = Cli::parse();

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        Commands::Headed {} => {
            run_multithreaded(async { run_headed().await });
        }
        Commands::Headless {
            tile_size,
            min,
            max,
        } => {
            run_multithreaded(async { run_headless(*tile_size, *min, *max).await });
        }
    }
}
