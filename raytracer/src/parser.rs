use std::collections::HashMap;
use std::fmt;

use crate::materials::MaterialType;
use crate::common::{Sphere, Triangle, Mesh};
use crate::camera::Camera;
use crate::maths::Vec3;


#[derive(Debug, Clone)]
pub enum ParseError {
    CouldntOpenFile,
    MissingCamera,
    WrongSyntax,
    DidntStartWith,
    NotAI32,
    NotAF32
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParseError::CouldntOpenFile => write!(f, "Couldn't open file"),
            ParseError::MissingCamera => write!(f, "Missing camera"),
            ParseError::WrongSyntax   => write!(f, "Wrong syntax"),
            _ => write!(f, "Error."),
        }
    }
}
impl From<std::num::ParseIntError> for ParseError {
    fn from(_err: std::num::ParseIntError) -> ParseError {
        ParseError::NotAI32
    }
}

impl From<std::num::ParseFloatError> for ParseError {
    fn from(_err: std::num::ParseFloatError) -> ParseError {
        ParseError::NotAF32
    }
}

impl std::error::Error for ParseError {}


type Result<T> = std::result::Result<T, ParseError>;

pub fn parse_world() -> Result<(Camera, Vec<Sphere>, Mesh)>  {
    parse_input(
        &std::fs::read_to_string("/Users/tedkleinbergman/Programming/rust_raytracer/src/world.txt")
            .map_err(|_| ParseError::CouldntOpenFile)?
    )
}

pub fn skip_whitespace(source: &str) -> &str {
    let index = source.find(|c: char| !c.is_whitespace()).unwrap_or(source.len());
    &source[index..]
}

pub fn get_identifier(source: &str) -> (&str, &str) {
    let index = source.find(|c: char| !(c.is_alphanumeric() || c == '_')).unwrap_or(source.len());
    (&source[index..], &source[..index])  // TODO: Index error on first slice.
}

// Returns the source at the start of the substring target.
pub fn find<'a>(source: &'a str, target: &str) -> Result<&'a str> {
    // TODO: Better algorithm. KMP?
    let size = target.len();
    let mut i = 0;
    while i + size <= source.len()  {
        if source[i..i+size] == target[..] {
            return Ok(&source[i..]);
        }
        i += 1;
    }

    Err(ParseError::DidntStartWith)  // TODO: Custom error
}

/// Checks if `source` starts with `target` and returns a
/// slice of `source` from after the `target`.
pub fn starts_with<'a>(source: &'a str, target: &str) -> Result<&'a str> {
    let size = target.len();
    if source.len() >= size && source[0..size] == target[..] {
        Ok(&source[size..])
    } else {
        Err(ParseError::DidntStartWith)
    }
}

pub fn parse_int(source: &str) -> Result<(&str, i32)> {
    let data = source.as_bytes();
    let mut index = 0;

    for c in data[index..data.len()].into_iter() {
        if b'0' <= *c && *c <= b'9' {
            index += 1;
        } else {
            break;
        }
    }

    let result = source[0..index].parse::<i32>()?;
    Ok((&source[index..], result))  // Index error
}


pub fn parse_float(source: &str) -> Result<(&str, f32)> {
    let data = source.as_bytes();
    let mut found_dot = false;
    let mut index     = 0;

    if data.len() < 3 {
        return Err(ParseError::NotAF32);
    }

    if data[0] == b'-' {
        index = 1;
    }

    for c in data[index..data.len()].into_iter() {
        if b'0' <= *c && *c <= b'9' {
            index += 1;
        } else if *c == b'.' {
            if !found_dot { found_dot = true } else { return Err(ParseError::NotAF32); }
            index += 1;
        } else {
            break;
        }
    }

    let result = source[0..index].parse::<f32>()?;
    Ok((&source[index..], result))  // Index error
}

pub fn parse_vec3(source: &str) -> Result<(&str, Vec3)> {
    let (source, x) = parse_float(source)?;
    let source      = skip_whitespace(source);
    let (source, y) = parse_float(source)?;
    let source      = skip_whitespace(source);
    let (source, z) = parse_float(source)?;
    Ok((source, Vec3{x, y, z}))
}

/// camera : camera origin <f32> <f32> <f32> aspect <f32> ;
pub fn parse_camera(source: &str) -> Option<Result<(&str, Camera)>> {
    if let Ok(source) = starts_with(source, "camera") {
        let result = || {
            let source = skip_whitespace(source);

            let source = starts_with(source, "origin")?;
            let source = skip_whitespace(source);
            let (source, o) = parse_vec3(source)?;
            let source = skip_whitespace(source);

            let source = starts_with(source, "aspect")?;
            let source = skip_whitespace(source);
            let (source, a) = parse_float(source)?;
            let source = skip_whitespace(source);

            let source = starts_with(source, ";")?;

            Ok((source, Camera::new_at(o, a)))
        };
        return Some(result());
    }
    None
}


/// material :  material <name> : <type> ;
/// type     :  <diffuse> | <metal> | <dielectric>
/// diffuse  :  Diffuse color <f32> <f32> <f32>
/// metal    :  Metal color <f32> <f32> <f32> fuzz <f32>
/// dielectric : Dielectric ir <f32>
pub fn parse_material(source: &str) -> Option<Result<(&str, &str, MaterialType)>> {
    if let Ok(source) = starts_with(source, "material") {
        let result = || {
            let source = skip_whitespace(source);

            let (source, name) = get_identifier(source);
            let source = skip_whitespace(source);
            let source = starts_with(source, ":")?;
            let source = skip_whitespace(source);

            if let Ok(source) = starts_with(source, "Diffuse") {
                let source = skip_whitespace(source);

                let source = starts_with(source, "color")?;
                let source = skip_whitespace(source);
                let (source, c) = parse_vec3(source)?;
                let source = skip_whitespace(source);

                let source = starts_with(source, ";")?;

                return Ok((source, name, MaterialType::Diffuse(c.into())));
            }

            if let Ok(source) = starts_with(source, "Metal") {
                let source = skip_whitespace(source);

                let source = starts_with(source, "color")?;
                let source = skip_whitespace(source);
                let (source, c) = parse_vec3(source)?;
                let source = skip_whitespace(source);

                let source = starts_with(source, "fuzz")?;
                let source = skip_whitespace(source);
                let (source, f) = parse_float(source)?;
                let source = skip_whitespace(source);

                let source = starts_with(source, ";")?;

                return Ok((source, name, MaterialType::Metal(c.into(), f)));
            }

            if let Ok(source) = starts_with(source, "Dielectric") {
                let source = skip_whitespace(source);

                let source = starts_with(source, "ir")?;
                let source = skip_whitespace(source);
                let (source, i) = parse_float(source)?;
                let source = skip_whitespace(source);

                let source = starts_with(source, ";")?;

                return Ok((source, name, MaterialType::Dielectric(i)));
            }

            Err(ParseError::WrongSyntax)
        };
        return Some(result());
    }
    None
}

/// sphere : sphere center <f32> <f32> <f32> radius <f32> material <name> ;
pub fn parse_sphere<'a>(source: &'a str, materials: &HashMap<&'a str, MaterialType>) -> Option<Result<(&'a str, Sphere)>> {
    if let Ok(source) = starts_with(source, "sphere") {
        let result = || {
            let source = skip_whitespace(source);

            let source = starts_with(source, "center")?;
            let source = skip_whitespace(source);
            let (source, c) = parse_vec3(source)?;
            let source = skip_whitespace(source);

            let source = starts_with(source, "radius")?;
            let source = skip_whitespace(source);
            let (source, r) = parse_float(source)?;
            let source = skip_whitespace(source);

            let source = starts_with(source, "material")?;
            let source = skip_whitespace(source);
            let (source, m) = get_identifier(source);
            let source = skip_whitespace(source);

            let source = starts_with(source, ";")?;

            let material = materials.get(m).ok_or(ParseError::WrongSyntax)?.to_owned();

            return Ok((
                source, Sphere{ center: c, radius: r, material }
            ));
        };
        return Some(result());
    }

    None
}

/// triangle : triangle v0 <f32> <f32> <f32> v1 <f32> <f32> <f32> v2 <f32> <f32> <f32> material <name> ;
pub fn parse_triangle<'a>(source: &'a str, materials: &HashMap<&'a str, MaterialType>) -> Option<Result<(&'a str, Triangle)>> {
    if let Ok(source) = starts_with(source, "triangle") {
        let result = || {
            let source = skip_whitespace(source);

            let source = starts_with(source, "v0")?;
            let source = skip_whitespace(source);
            let (source, v0) = parse_vec3(source)?;
            let source = skip_whitespace(source);

            let source = starts_with(source, "v1")?;
            let source = skip_whitespace(source);
            let (source, v1) = parse_vec3(source)?;
            let source = skip_whitespace(source);

            let source = starts_with(source, "v2")?;
            let source = skip_whitespace(source);
            let (source, v2) = parse_vec3(source)?;
            let source = skip_whitespace(source);

            let source = starts_with(source, "material")?;
            let source = skip_whitespace(source);
            let (source, m) = get_identifier(source);
            let source = skip_whitespace(source);

            let source = starts_with(source, ";")?;

            let material = materials.get(m).ok_or(ParseError::WrongSyntax)?.to_owned();

            return Ok((
                source, Triangle::new(v0, v1, v2, material)
            ));
        };

        return Some(result());
    }

    None
}


pub fn skip_comment(mut source: &str) -> Result<&str>  {
    while let Ok(comment_line) = starts_with(source, "//") {
        if let Ok(end_line) = find(comment_line, "\n") {
            source = &end_line[1..];  // Go past new line. TODO: Index error.
        } else {
            return Err(ParseError::WrongSyntax);  // TODO: Better error.
        }
    }

    Ok(source)
}


/// --- Syntax ----
/// program  :  <camera> (<material>)* (<sphere>)* (<triangle>)*
/// camera   :  camera origin <f32> <f32> <f32> aspect <f32> ;
/// material :  material <name> : <type> ;
/// type     :  <diffuse> | <metal> | <dielectric>
/// diffuse  :  Diffuse color <f32> <f32> <f32>
/// metal    :  Metal color <f32> <f32> <f32> fuzz <f32>
/// dielectric : Dielectric ir <f32>
/// sphere   :  sphere center <f32> <f32> <f32> radius <f32> material <name> ;
/// triangle :  v0 <f32> <f32> <f32> v1 <f32> <f32> <f32> v2 <f32> <f32> <f32> material <name> ;
pub fn parse_input(mut source: &str) -> Result<(Camera, Vec<Sphere>, Mesh)> {
    let mut materials = HashMap::new();
    let mut spheres : Vec<Sphere> = Vec::new();
    let mut triangles: Vec<Triangle> = Vec::new();

    // Parse camera
    source = skip_comment(source)?;
    let camera =
        if let Some(result) = parse_camera(source) {
            let (next, camera) = result?;
            source = skip_whitespace(next);
            camera
        } else {
            return Err(ParseError::MissingCamera);
        };

    // Parse all materials
    source = skip_comment(source)?;
    while let Some(result) = parse_material(source) {
        let (next, name, material) = result?;
        materials.insert(name, material);
        source = skip_whitespace(next);
        source = skip_comment(source)?;
    }

    // Parse all spheres.
    while let Some(result) = parse_sphere(source, &materials) {
        let (next, sphere) = result?;
        spheres.push(sphere);
        source = skip_whitespace(next);
        source = skip_comment(source)?;
    }

    // Parse all triangles.
    while let Some(result) = parse_triangle(source, &materials) {
        let (next, triangle) = result?;
        triangles.push(triangle);
        source = skip_whitespace(next);
        source = skip_comment(source)?;
    }

    if !source.is_empty() {
        Err(ParseError::WrongSyntax)
    } else {
        Ok((camera, spheres, Mesh::new(triangles)))
    }
}