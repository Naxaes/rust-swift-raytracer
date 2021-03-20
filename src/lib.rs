pub mod maths;
pub mod parser;
pub mod camera;
pub mod image;
pub mod random;
pub mod mat3;
pub mod materials;
pub mod common;


use image::{Framebuffer, Color};
use camera::Camera;
use common::{World, Options, ray_trace};

use std::ffi::CStr;
use std::os::raw::c_char;
use std::ptr::NonNull;


#[repr(C)]
pub struct CFramebuffer {
    pub width:  usize,
    pub height: usize,
    pub pixels: NonNull<Color>,
}

#[repr(C)]
pub struct WorldHandle<'a> {
    world:  Box<World<'a>>,
    camera: Box<Camera>,
}

#[no_mangle]
pub extern "C" fn load_world<'a>(source: *const c_char) -> Box<WorldHandle<'a>> {
    let c_str = unsafe { CStr::from_ptr(source) };
    let (camera, spheres) = parser::parse_input(c_str.to_str().unwrap()).unwrap();
    let world = World::new(spheres, vec![]);
    Box::new(WorldHandle {
        camera: Box::new(camera),
        world: Box::new(world),
    })
}


#[no_mangle]
pub extern "C" fn render(framebuffer: CFramebuffer, handle: *const WorldHandle) -> CFramebuffer {
    let options = Options::new(16, 8, true);

    let WorldHandle { world, camera } = unsafe { &(*handle) };
    let framebuffer = ray_trace(world, camera, framebuffer.into(), &options);

    framebuffer.into()
}





impl Into<Framebuffer> for CFramebuffer {
    fn into(self) -> Framebuffer {
        let count = self.width * self.height;
        Framebuffer {
            width:  self.width,
            height: self.height,
            pixels: unsafe { std::slice::from_raw_parts(self.pixels.as_ptr(), count).to_vec() },
        }
    }
}
impl From<Framebuffer> for CFramebuffer {
    fn from(mut framebuffer: Framebuffer) -> Self {
        Self {
            width:  framebuffer.width,
            height: framebuffer.height,
            pixels: unsafe { std::ptr::NonNull::new_unchecked(framebuffer.pixels.as_mut_ptr()) }
        }
    }
}
