use crate::entity::model::files::obj::Error::MissingTag;
use std::borrow::Cow;
use std::num::{NonZeroU32, ParseFloatError, ParseIntError};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    MissingTag,
    UnrecognizedTag,
    ParseIntError(ParseIntError),
    ParseFloatError(ParseFloatError),
    MissingNumber,
}
impl From<ParseIntError> for Error {
    fn from(e: ParseIntError) -> Self {
        Error::ParseIntError(e)
    }
}
impl From<ParseFloatError> for Error {
    fn from(e: ParseFloatError) -> Self {
        Error::ParseFloatError(e)
    }
}
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Default, Hash)]
pub struct VertexIndices {
    pub position: u32,
    pub texture_coords: Option<NonZeroU32>,
    pub normal: Option<NonZeroU32>,
}
impl FromStr for VertexIndices {
    type Err = std::num::ParseIntError;

    fn from_str(indices_str: &str) -> Result<Self, Self::Err> {
        let mut indices = [0; 3];
        for (index, s) in indices_str.split('/').enumerate() {
            if !s.is_empty() {
                indices[index] = s.parse()?;
            }
        }
        Ok(VertexIndices {
            position: indices[0],
            texture_coords: NonZeroU32::new(indices[1]),
            normal: NonZeroU32::new(indices[2]),
        })
    }
}
#[derive(Copy, Clone, PartialOrd, PartialEq, Debug, Default)]
pub struct Vertex {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}
impl FromStr for Vertex {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut nums = s.split(' ');
        let x = nums.next().ok_or(Error::MissingNumber)?.parse()?;
        let y = nums.next().ok_or(Error::MissingNumber)?.parse()?;
        let z = nums.next().ok_or(Error::MissingNumber)?.parse()?;
        let w = nums.next().map(|w| w.parse()).transpose()?.unwrap_or(1f32);
        Ok(Vertex { x, y, z, w })
    }
}
#[derive(Copy, Clone, PartialOrd, PartialEq, Debug, Default)]
pub struct TextureCoords {
    pub u: f32,
    pub v: f32,
    pub w: f32,
}
impl FromStr for TextureCoords {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut nums = s.split(' ');
        let u = nums.next().ok_or(Error::MissingNumber)?.parse()?;
        let v = nums.next().ok_or(Error::MissingNumber)?.parse()?;
        let w = nums.next().map(|w| w.parse()).transpose()?.unwrap_or(1f32);
        Ok(TextureCoords { u, v, w })
    }
}
#[derive(Clone, PartialOrd, PartialEq, Debug)]
pub enum Line<'a> {
    Vertex(Vertex),
    Normal(Vertex),
    TextureCoords(TextureCoords),
    Point(VertexIndices),
    Line(VertexIndices, VertexIndices),
    Face(VertexIndices, VertexIndices, VertexIndices),
    SmoothingGroup(Option<NonZeroU32>),
    Group(Cow<'a, str>),
    UseMtl(Cow<'a, str>),
    MtlLib(Cow<'a, str>),
    Name(Cow<'a, str>),
    Comment(Cow<'a, str>),
}
impl<'a> Line<'a> {
    pub fn to_static(self) -> Line<'static> {
        match self {
            Line::Group(name) => Line::Group(Cow::Owned(name.into_owned())),
            Line::UseMtl(name) => Line::UseMtl(Cow::Owned(name.into_owned())),
            Line::MtlLib(name) => Line::MtlLib(Cow::Owned(name.into_owned())),
            Line::Name(name) => Line::Name(Cow::Owned(name.into_owned())),
            Line::Comment(name) => Line::Comment(Cow::Owned(name.into_owned())),

            Line::Vertex(x) => Line::Vertex(x),
            Line::Normal(x) => Line::Normal(x),
            Line::TextureCoords(x) => Line::TextureCoords(x),
            Line::Point(p1) => Line::Point(p1),
            Line::SmoothingGroup(g) => Line::SmoothingGroup(g),

            Line::Line(p1, p2) => Line::Line(p1, p2),
            Line::Face(p1, p2, p3) => Line::Face(p1, p2, p3),
        }
    }
    pub fn process_line(line: &'a str) -> Result<Self, Error> {
        let (tag, rest) = line.split_once(' ').ok_or(MissingTag)?;
        match tag {
            "#" => Ok(Line::Comment(Cow::Borrowed(rest))),
            "o" => Ok(Line::Name(Cow::Borrowed(rest))),
            "usemtl" => Ok(Line::UseMtl(Cow::Borrowed(rest))),
            "mtllib" => Ok(Line::MtlLib(Cow::Borrowed(rest))),
            "g" => Ok(Line::Group(Cow::Borrowed(rest))),

            "v" => Ok(Line::Vertex(rest.parse()?)),
            "vt" => Ok(Line::TextureCoords(rest.parse()?)),
            "vn" => Ok(Line::Normal(rest.parse()?)),
            "p" => Ok(Line::Point(rest.parse()?)),
            "l" => {
                let mut nums = rest.split(' ');
                let p1 = nums.next().ok_or(Error::MissingNumber)?.parse()?;
                let p2 = nums.next().ok_or(Error::MissingNumber)?.parse()?;
                Ok(Line::Line(p1, p2))
            }
            "f" => {
                let mut nums = rest.split(' ');
                let p1 = nums.next().ok_or(Error::MissingNumber)?.parse()?;
                let p2 = nums.next().ok_or(Error::MissingNumber)?.parse()?;
                let p3 = nums.next().ok_or(Error::MissingNumber)?.parse()?;
                Ok(Line::Face(p1, p2, p3))
            }
            "s" => Ok(Line::SmoothingGroup(match rest {
                "off" => None,
                _ => NonZeroU32::new(rest.parse()?),
            })),
            _ => Err(Error::UnrecognizedTag),
        }
    }
}
