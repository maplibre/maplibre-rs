#![deny(unused_imports)]

use std::{io::ErrorKind, path::PathBuf};

use clap::{Parser, Subcommand};
use maplibre::{coords::LatLon, render::settings::WgpuSettings};
use maplibre_winit::{run_headed_map, WinitMapWindowConfig};

#[cfg(feature = "headless")]
mod headless;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

fn parse_lat_long(env: &str) -> Result<LatLon, std::io::Error> {
    let split = env.split(',').collect::<Vec<_>>();
    if let (Some(latitude), Some(longitude)) = (split.first(), split.get(1)) {
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
    #[cfg(feature = "headless")]
    Headless {
        #[clap(default_value_t = 400)]
        tile_size: u32,
        #[clap(
            value_parser = clap::builder::ValueParser::new(parse_lat_long),
            default_value_t = LatLon::new(48.0345697188, 11.3475219363)
        )]
        min: LatLon,
        #[clap(
            value_parser = clap::builder::ValueParser::new(parse_lat_long),
            default_value_t = LatLon::new(48.255861, 11.7917815798)
        )]
        max: LatLon,
    },
}

fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    #[cfg(feature = "trace")]
    maplibre::platform::trace::enable_tracing();

    let cli = Cli::parse();

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        Commands::Headed {} => run_headed_map(
            Some(PathBuf::from("./maplibre-cache".to_string())),
            WinitMapWindowConfig::new("maplibre".to_string()),
            WgpuSettings {
                backends: Some(maplibre::render::settings::Backends::all()),
                ..WgpuSettings::default()
            },
        ),
        #[cfg(feature = "headless")]
        Commands::Headless {
            tile_size,
            min,
            max,
        } => {
            maplibre::platform::run_multithreaded(async {
                headless::run_headless(*tile_size, *min, *max).await
            });
        }
    }
}
