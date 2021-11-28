// encoder.rs
//
// Copyright (c) 2019-2021  Minnesota Department of Transportation
//
//! Encoder for Mapbox Vector Tile (MVT) geometry.
//!
use crate::error::{Error, Result};
use pointy::{Float, Transform};

#[derive(Copy, Clone, Debug)]
enum Command {
    MoveTo = 1,
    LineTo = 2,
    ClosePath = 7,
}

#[derive(Copy, Clone, Debug)]
struct CommandInt {
    id: Command,
    count: u32,
}

#[derive(Copy, Clone, Debug)]
struct ParamInt {
    value: i32,
}

/// Geometry types for [Features](struct.Feature.html).
#[derive(Clone, Copy, Debug)]
pub enum GeomType {
    /// Point or multipoint
    Point,

    /// Linestring or Multilinestring
    Linestring,

    /// Polygon or Multipolygon
    Polygon,
}

/// Encoder for [Feature](struct.Feature.html) geometry.
///
/// This can consist of Point, Linestring or Polygon data.
///
/// # Example
/// ```
/// # use mvt::{Error, GeomEncoder, GeomType};
/// # use pointy::Transform;
/// # fn main() -> Result<(), Error> {
/// let geom_data = GeomEncoder::new(GeomType::Point, Transform::default())
///     .point(0.0, 0.0)?
///     .point(10.0, 0.0)?
///     .encode()?;
/// # Ok(()) }
/// ```
pub struct GeomEncoder<F>
where
    F: Float,
{
    geom_tp: GeomType,
    transform: Transform<F>,
    x: i32,
    y: i32,
    cmd_offset: usize,
    count: u32,
    data: Vec<u32>,
}

/// Validated geometry data for [Feature](struct.Feature.html)s.
///
/// Use [GeomEncoder](struct.GeomEncoder.html) to encode.
///
/// # Example
/// ```
/// # use mvt::{Error, GeomEncoder, GeomType};
/// # use pointy::Transform;
/// # fn main() -> Result<(), Error> {
/// let geom_data = GeomEncoder::new(GeomType::Point, Transform::default())
///     .point(0.0, 0.0)?
///     .point(10.0, 0.0)?
///     .encode()?;
/// # Ok(()) }
/// ```
pub struct GeomData {
    geom_tp: GeomType,
    data: Vec<u32>,
}

impl CommandInt {
    fn new(id: Command, count: u32) -> Self {
        CommandInt { id, count }
    }

    fn encode(&self) -> u32 {
        ((self.id as u32) & 0x7) | (self.count << 3)
    }
}

impl ParamInt {
    fn new(value: i32) -> Self {
        ParamInt { value }
    }

    fn encode(&self) -> u32 {
        ((self.value << 1) ^ (self.value >> 31)) as u32
    }
}

impl<F> GeomEncoder<F>
where
    F: Float,
{
    /// Create a new geometry encoder.
    ///
    /// * `geom_tp` Geometry type.
    /// * `transform` Transform to apply to geometry.
    pub fn new(geom_tp: GeomType, transform: Transform<F>) -> Self {
        GeomEncoder {
            geom_tp,
            transform,
            x: 0,
            y: 0,
            count: 0,
            cmd_offset: 0,
            data: vec![],
        }
    }

    /// Add a Command
    fn command(&mut self, cmd: Command, count: u32) {
        self.cmd_offset = self.data.len();
        debug!("command: {:?}", &cmd);
        self.data.push(CommandInt::new(cmd, count).encode());
    }

    /// Set count of the most recent Command.
    fn set_command(&mut self, cmd: Command, count: u32) {
        let off = self.cmd_offset;
        self.data[off] = CommandInt::new(cmd, count).encode();
    }

    /// Push one point with relative coörindates.
    fn push_point(&mut self, x: F, y: F) -> Result<()> {
        let p = self.transform * (x, y);
        let x = p.x().round().to_i32().ok_or(Error::InvalidValue())?;
        let y = p.y().round().to_i32().ok_or(Error::InvalidValue())?;
        self.data
            .push(ParamInt::new(x.saturating_sub(self.x)).encode());
        self.data
            .push(ParamInt::new(y.saturating_sub(self.y)).encode());
        debug!("point: {},{}", x, y);
        self.x = x;
        self.y = y;
        Ok(())
    }

    /// Add a point.
    pub fn add_point(&mut self, x: F, y: F) -> Result<()> {
        match self.geom_tp {
            GeomType::Point => {
                if self.count == 0 {
                    self.command(Command::MoveTo, 1);
                }
            }
            GeomType::Linestring => match self.count {
                0 => self.command(Command::MoveTo, 1),
                1 => self.command(Command::LineTo, 1),
                _ => (),
            },
            GeomType::Polygon => match self.count {
                0 => self.command(Command::MoveTo, 1),
                1 => self.command(Command::LineTo, 1),
                _ => (),
            },
        }
        self.push_point(x, y)?;
        self.count += 1;
        Ok(())
    }

    /// Add a point, taking ownership (for method chaining).
    pub fn point(mut self, x: F, y: F) -> Result<Self> {
        self.add_point(x, y)?;
        Ok(self)
    }

    /// Complete the current geometry (for multilinestring / multipolygon).
    pub fn complete_geom(&mut self) -> Result<()> {
        // FIXME: return Error::InvalidGeometry
        //        if "MUST" rules in the spec are violated
        match self.geom_tp {
            GeomType::Point => (),
            GeomType::Linestring => {
                if self.count > 1 {
                    self.set_command(Command::LineTo, self.count - 1);
                }
                self.count = 0;
            }
            GeomType::Polygon => {
                if self.count > 1 {
                    self.set_command(Command::LineTo, self.count - 1);
                    self.command(Command::ClosePath, 1);
                }
                self.count = 0;
            }
        }
        Ok(())
    }

    /// Complete the current geometry (for multilinestring / multipolygon).
    pub fn complete(mut self) -> Result<Self> {
        self.complete_geom()?;
        Ok(self)
    }

    /// Encode the geometry data, consuming the encoder.
    pub fn encode(mut self) -> Result<GeomData> {
        // FIXME: return Error::InvalidGeometry
        //        if "MUST" rules in the spec are violated
        self = if let GeomType::Point = self.geom_tp {
            if self.count > 1 {
                self.set_command(Command::MoveTo, self.count);
            }
            self
        } else {
            self.complete()?
        };
        Ok(GeomData::new(self.geom_tp, self.data))
    }
}

impl GeomData {
    /// Create new geometry data.
    ///
    /// * `geom_tp` Geometry type.
    /// * `data` Validated geometry.
    fn new(geom_tp: GeomType, data: Vec<u32>) -> Self {
        GeomData { geom_tp, data }
    }

    /// Get the geometry type
    pub(crate) fn geom_type(&self) -> GeomType {
        self.geom_tp
    }

    /// Get the geometry data
    pub(crate) fn into_vec(self) -> Vec<u32> {
        self.data
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // Examples from MVT spec:
    #[test]
    fn test_point() {
        let v = GeomEncoder::new(GeomType::Point, Transform::default())
            .point(25.0, 17.0)
            .unwrap()
            .encode()
            .unwrap()
            .into_vec();
        assert_eq!(v, vec!(9, 50, 34));
    }

    #[test]
    fn test_multipoint() {
        let v = GeomEncoder::new(GeomType::Point, Transform::default())
            .point(5.0, 7.0)
            .unwrap()
            .point(3.0, 2.0)
            .unwrap()
            .encode()
            .unwrap()
            .into_vec();
        assert_eq!(v, vec!(17, 10, 14, 3, 9));
    }

    #[test]
    fn test_linestring() {
        let v = GeomEncoder::new(GeomType::Linestring, Transform::default())
            .point(2.0, 2.0)
            .unwrap()
            .point(2.0, 10.0)
            .unwrap()
            .point(10.0, 10.0)
            .unwrap()
            .encode()
            .unwrap()
            .into_vec();
        assert_eq!(v, vec!(9, 4, 4, 18, 0, 16, 16, 0));
    }

    #[test]
    fn test_multilinestring() {
        let v = GeomEncoder::new(GeomType::Linestring, Transform::default())
            .point(2.0, 2.0)
            .unwrap()
            .point(2.0, 10.0)
            .unwrap()
            .point(10.0, 10.0)
            .unwrap()
            .complete()
            .unwrap()
            .point(1.0, 1.0)
            .unwrap()
            .point(3.0, 5.0)
            .unwrap()
            .encode()
            .unwrap()
            .into_vec();
        assert_eq!(v, vec!(9, 4, 4, 18, 0, 16, 16, 0, 9, 17, 17, 10, 4, 8));
    }

    #[test]
    fn test_polygon() {
        let v = GeomEncoder::new(GeomType::Polygon, Transform::default())
            .point(3.0, 6.0)
            .unwrap()
            .point(8.0, 12.0)
            .unwrap()
            .point(20.0, 34.0)
            .unwrap()
            .encode()
            .unwrap()
            .into_vec();
        assert_eq!(v, vec!(9, 6, 12, 18, 10, 12, 24, 44, 15));
    }

    #[test]
    fn test_multipolygon() {
        let v = GeomEncoder::new(GeomType::Polygon, Transform::default())
            // positive area => exterior ring
            .point(0.0, 0.0)
            .unwrap()
            .point(10.0, 0.0)
            .unwrap()
            .point(10.0, 10.0)
            .unwrap()
            .point(0.0, 10.0)
            .unwrap()
            .complete()
            .unwrap()
            // positive area => exterior ring
            .point(11.0, 11.0)
            .unwrap()
            .point(20.0, 11.0)
            .unwrap()
            .point(20.0, 20.0)
            .unwrap()
            .point(11.0, 20.0)
            .unwrap()
            .complete()
            .unwrap()
            // negative area => interior ring
            .point(13.0, 13.0)
            .unwrap()
            .point(13.0, 17.0)
            .unwrap()
            .point(17.0, 17.0)
            .unwrap()
            .point(17.0, 13.0)
            .unwrap()
            .encode()
            .unwrap()
            .into_vec();
        assert_eq!(
            v,
            vec!(
                9, 0, 0, 26, 20, 0, 0, 20, 19, 0, 15, 9, 22, 2, 26, 18, 0, 0,
                18, 17, 0, 15, 9, 4, 13, 26, 0, 8, 8, 0, 0, 7, 15
            )
        );
    }
}
