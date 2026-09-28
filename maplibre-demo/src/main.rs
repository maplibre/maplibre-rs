#![deny(unused_imports)]

use std::{io::ErrorKind, path::PathBuf};

use clap::{Parser, Subcommand, ValueEnum};
use maplibre::{
    coords::LatLon,
    projection::{ProjectionSpecification, ProjectionType},
    render::settings::WgpuSettings,
    style::Style,
};
use maplibre_winit::{run_headed_map, WinitMapWindowConfig};

#[cfg(feature = "headless")]
mod headless;

/// World-scale style rendered on the globe, bundled so the demo works without any arguments.
const GLOBE_STYLE: &str = include_str!("../res/globe.json");

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[allow(dead_code)]
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

/// Projection forced onto the loaded style.
#[derive(Clone, Copy, Debug, ValueEnum)]
enum ProjectionArg {
    Globe,
    Mercator,
    VerticalPerspective,
}

impl From<ProjectionArg> for ProjectionType {
    fn from(projection: ProjectionArg) -> Self {
        match projection {
            ProjectionArg::Globe => ProjectionType::Globe,
            ProjectionArg::Mercator => ProjectionType::Mercator,
            ProjectionArg::VerticalPerspective => ProjectionType::VerticalPerspective,
        }
    }
}

#[derive(Subcommand)]
enum Commands {
    /// Opens an interactive map window.
    ///
    /// Controls: left-drag pans, wheel or +/- zooms around the pointer, right-drag rotates and
    /// tilts, W/A/S/D or the arrow keys pan, Escape quits.
    Headed {
        /// Path to a MapLibre style JSON file. Defaults to the built-in style.
        #[clap(long, conflicts_with = "globe")]
        style: Option<PathBuf>,
        /// Use the bundled world style rendered on the globe.
        #[clap(long)]
        globe: bool,
        /// Override the projection declared by the style.
        #[clap(long, value_enum)]
        projection: Option<ProjectionArg>,
        /// Exit after rendering this many frames.
        #[clap(long)]
        frames: Option<u64>,
    },
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

fn load_style(
    path: Option<&PathBuf>,
    globe: bool,
    projection: Option<ProjectionArg>,
) -> Result<Style, Box<dyn std::error::Error>> {
    let mut style = match (path, globe) {
        (Some(path), _) => {
            let json = std::fs::read_to_string(path)
                .map_err(|error| format!("cannot read style {}: {error}", path.display()))?;
            serde_json::from_str::<Style>(&json)
                .map_err(|error| format!("cannot parse style {}: {error}", path.display()))?
        }
        (None, true) => serde_json::from_str::<Style>(GLOBE_STYLE)?,
        (None, false) => Style::default(),
    };
    if path.is_some() || globe {
        // Layer order becomes render order; index 0 is reserved for the depth clear.
        for (index, layer) in style.layers.iter_mut().enumerate() {
            layer.index = index as u32 + 1;
        }
    }
    if let Some(projection) = projection {
        style.projection = Some(ProjectionSpecification {
            projection_type: projection.into(),
        });
    }
    Ok(style)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Captures both tracing events and `log` records; RUST_LOG selects the level.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    #[cfg(feature = "trace")]
    maplibre::platform::trace::enable_tracing();

    let cli = Cli::parse();

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        Commands::Headed {
            style,
            globe,
            projection,
            frames,
        } => {
            let style = load_style(style.as_ref(), *globe, *projection)?;
            run_headed_map(
                Some(PathBuf::from("./maplibre-cache".to_string())),
                WinitMapWindowConfig::new("maplibre".to_string()),
                WgpuSettings {
                    backends: Some(maplibre::render::settings::Backends::all()),
                    ..WgpuSettings::default()
                },
                style,
                *frames,
            );
        }
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
    Ok(())
}
