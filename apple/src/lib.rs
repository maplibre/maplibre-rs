//! Apple host facade for the MapLibre Rust renderer.

#![deny(unused_imports)]

use std::{ffi::CStr, os::raw::c_char, path::PathBuf};

use maplibre::{render::settings::WgpuSettings, style::Style};
use maplibre_winit::{run_headed_map, run_headed_map_with_mbtiles, WinitMapWindowConfig};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[cfg(not(any(no_pendantic_os_check, target_os = "macos", target_os = "ios")))]
compile_error!("apple works only on macOS and iOS.");

#[no_mangle]
pub fn maplibre_apple_main() {
    initialize_diagnostics();

    run_headed_map::<String>(
        None,
        WinitMapWindowConfig::new("maplibre".to_string()),
        WgpuSettings {
            backends: Some(maplibre::render::settings::Backends::all()),
            ..WgpuSettings::default()
        },
    );
}

/// Starts the Apple renderer with one read-only MBTiles archive.
///
/// # Safety
///
/// `archive_path` and `style_json` must point to valid, null-terminated UTF-8 strings for the
/// duration of this call.
#[no_mangle]
pub unsafe extern "C" fn maplibre_apple_main_with_mbtiles(
    archive_path: *const c_char,
    style_json: *const c_char,
) -> bool {
    initialize_diagnostics();
    if archive_path.is_null() || style_json.is_null() {
        tracing::error!("The MBTiles archive path or style JSON is null");
        return false;
    }
    let archive_path = unsafe { CStr::from_ptr(archive_path) };
    let style_json = unsafe { CStr::from_ptr(style_json) };
    let Ok(archive_path) = archive_path.to_str() else {
        tracing::error!("The MBTiles archive path is not valid UTF-8");
        return false;
    };
    let Ok(style_json) = style_json.to_str() else {
        tracing::error!("The style JSON is not valid UTF-8");
        return false;
    };
    let Ok(mut style) = serde_json::from_str::<Style>(style_json) else {
        tracing::error!("The style JSON is not valid");
        return false;
    };
    if let Err(position) = assign_style_layer_indices(&mut style) {
        tracing::error!(position, "The style has too many layers");
        return false;
    }

    match run_headed_map_with_mbtiles(
        PathBuf::from(archive_path),
        style,
        WinitMapWindowConfig::new("maplibre".to_string()),
        WgpuSettings {
            backends: Some(maplibre::render::settings::Backends::all()),
            ..WgpuSettings::default()
        },
    ) {
        Ok(()) => true,
        Err(error) => {
            tracing::error!("Failed to start with the MBTiles archive: {error}");
            false
        }
    }
}

fn assign_style_layer_indices(style: &mut Style) -> Result<(), usize> {
    for (position, layer) in style.layers.iter_mut().enumerate() {
        layer.index = u32::try_from(position).map_err(|_| position)?;
    }
    Ok(())
}

fn initialize_diagnostics() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(filter)
        .try_init()
        .ok();
}

#[cfg(test)]
mod tests;
