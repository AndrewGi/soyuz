use crate::entity::model;
use crate::entity::model::files::obj::Error::MissingTag;
use std::borrow::Cow;
use std::io::BufRead;
use std::num::{NonZeroU32, ParseFloatError, ParseIntError};
use std::str::FromStr;
#[derive(Debug)]
pub enum Error {
    IO(std::io::Error),
    MissingTag,
    UnrecognizedTag,
    ParseIntError(ParseIntError),
    ParseFloatError(ParseFloatError),
    MissingNumber,
    MissingNormal,
    MissingTextureCoord,
    InvalidIndex,
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
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IO(e)
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

pub struct ObjectBuilder {
    pub vertices: Vec<Vertex>,
    pub normals: Vec<Vertex>,
    pub texture_coords: Vec<TextureCoords>,
    pub indices: Vec<VertexIndices>,

    pub mesh_vertices: Vec<model::Vertex>,
    pub mesh_indices: Vec<u32>,
}
impl ObjectBuilder {
    pub fn new() -> Self {
        ObjectBuilder {
            vertices: vec![],
            normals: vec![],
            texture_coords: vec![],
            indices: vec![],
            mesh_vertices: vec![],
            mesh_indices: vec![],
        }
    }
    pub fn handle_face(
        &mut self,
        v1: VertexIndices,
        v2: VertexIndices,
        v3: VertexIndices,
    ) -> Result<(), Error> {
        if v1.texture_coords.is_some() != v2.texture_coords.is_some()
            || v2.texture_coords.is_some() != v3.texture_coords.is_some()
        {
            return Err(Error::MissingTextureCoord);
        }
        if v1.normal.is_some() != v2.normal.is_some() || v2.normal.is_some() != v3.normal.is_some()
        {
            return Err(Error::MissingNormal);
        }
        let v1 = self.get_vertex(v1).ok_or(Error::InvalidIndex)?;
        let v2 = self.get_vertex(v2).ok_or(Error::InvalidIndex)?;
        let v3 = self.get_vertex(v3).ok_or(Error::InvalidIndex)?;

        let v1_i = self.add_vertex(v1);
        let v2_i = self.add_vertex(v2);
        let v3_i = self.add_vertex(v3);

        self.mesh_indices.push(v1_i);
        self.mesh_indices.push(v2_i);
        self.mesh_indices.push(v3_i);
        Ok(())
    }
    pub fn add_vertex(&mut self, v: model::Vertex) -> u32 {
        let existing_pos = self.mesh_vertices.iter().position(|vi| vi == &v);
        let pos = match existing_pos {
            Some(pos) => pos,
            None => {
                let pos = self.mesh_vertices.len();
                self.mesh_vertices.push(v);
                pos
            }
        };
        pos as u32
    }
    pub fn get_vertex(&self, v: VertexIndices) -> Option<model::Vertex> {
        let default_vertex = Vertex::default();
        let default_tc = TextureCoords::default();
        let vertex: &Vertex = self.vertices.get(v.position as usize)?;
        let normal: &Vertex = match v.normal {
            Some(ni) => self.normals.get(ni.get() as usize)?,
            None => &default_vertex,
        };
        let texture_coords: &TextureCoords = match v.texture_coords {
            Some(ti) => self.texture_coords.get(ti.get() as usize)?,
            None => &default_tc,
        };
        Some(model::Vertex {
            position: [vertex.x, vertex.y, vertex.z],
            normal: [normal.x, normal.y, normal.z],
            texture_coords: [texture_coords.u, texture_coords.v],
        })
    }
    pub fn process_line<'a>(&mut self, line: Line<'a>) -> Result<(), Error> {
        match line {
            Line::Vertex(v) => self.vertices.push(v),
            Line::Normal(n) => self.normals.push(n),
            Line::TextureCoords(tc) => self.texture_coords.push(tc),
            Line::Face(v1, v2, v3) => self.handle_face(v1, v2, v3)?,

            Line::Point(_) => todo!("handle obj point"),
            Line::Line(_, _) => todo!("handle obj line"),
            Line::SmoothingGroup(_) => todo!("handle obj smoothing group"),
            Line::Group(_) => todo!("handle obj group"),
            Line::UseMtl(_) => todo!("handle obj usemtl"),
            Line::MtlLib(_) => todo!("handle obj mtllib"),
            Line::Name(_) => todo!("handle obj name"),
            Line::Comment(_) => todo!("handle obj comment"),
        }
        Ok(())
    }
    pub fn process_lines<'a>(
        &mut self,
        mut lines: impl Iterator<Item = Line<'a>>,
    ) -> Result<(), Error> {
        while let Some(line) = lines.next() {
            self.process_line(line)?;
        }
        Ok(())
    }
    pub async fn load_file(filename: impl AsRef<std::path::Path>) -> Result<Self, Error> {
        let mut obj = Self::new();
        let file = tokio::fs::File::open(filename).await?;
        let file = tokio::io::BufReader::new(file);

        Ok(obj)
    }
}
