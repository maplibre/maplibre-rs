use std::{env, path::Path, process::exit};

use naga::{
    front::{wgsl, wgsl::ParseError},
    valid::{Capabilities, ValidationError, ValidationFlags, Validator},
    SourceLocation,
};
use walkdir::WalkDir;

#[derive(Debug)]
pub enum WgslError {
    ValidationErr(ValidationError),
    ParserErr {
        error: String,
        location: Option<SourceLocation>,
    },
    IoErr(std::io::Error),
}

impl From<std::io::Error> for WgslError {
    fn from(err: std::io::Error) -> Self {
        Self::IoErr(err)
    }
}

impl WgslError {
    pub fn from_parse_err(err: ParseError, src: &str) -> Self {
        let location = err.location(src);
        let error = err.message().to_string();
        Self::ParserErr { error, location }
    }
}

fn validate_wgsl(validator: &mut Validator, path: &Path) -> Result<(), WgslError> {
    let shader = std::fs::read_to_string(path).map_err(WgslError::from)?;
    let module = wgsl::parse_str(&shader).map_err(|err| WgslError::from_parse_err(err, &shader))?;

    if let Err(err) = validator.validate(&module) {
        Err(WgslError::ValidationErr(err.into_inner()))
    } else {
        Ok(())
    }
}

pub fn validate_project_wgsl() {
    let mut validator = Validator::new(ValidationFlags::all(), Capabilities::all());

    let root_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let dir_walk = WalkDir::new(&root_dir);
    let dir_walk = dir_walk.into_iter().filter_entry(|e| {
        let path = e.path();

        if !path.is_dir() {
            path.extension().map(|ext| ext == "wgsl").unwrap_or(false)
        } else {
            true
        }
    });

    for entry in dir_walk {
        match entry {
            Ok(entry) => {
                let path = entry.path();
                if !path.is_dir() {
                    match validate_wgsl(&mut validator, path) {
                        Ok(_) => {}
                        Err(err) => {
                            let path = path.strip_prefix(&root_dir).unwrap_or(path);
                            println!(
                                "cargo:warning={}{}",
                                path.to_str().unwrap(),
                                match err {
                                    WgslError::ValidationErr(error) => format!(": {error:?}"),
                                    WgslError::ParserErr { error, location } =>
                                        if let Some(SourceLocation {
                                            line_number,
                                            line_position,
                                            ..
                                        }) = location
                                        {
                                            format!(":{line_number}:{line_position} {error}")
                                        } else {
                                            error
                                        },
                                    WgslError::IoErr(error) => format!(": {error:?}"),
                                }
                            );
                            exit(1);
                        }
                    };
                }
            }
            Err(error) => {
                println!("cargo:warning={error:?}");
                exit(1);
            }
        }
    }
}
