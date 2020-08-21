use std::{
    fmt::Debug,
    io::{self, Write},
};

use rand::prelude::*;

pub mod shapes;
pub use shapes::{Bbox, Circle};

pub trait Shape: Clone + Debug {
    fn bbox(&self) -> Bbox;
    fn center(&self) -> (f32, f32);
    fn area(&self) -> f32;
    fn sdf(&self, x: f32, y: f32) -> f32;

    fn random_point<R: Rng>(&self, rng: &mut R) -> (f32, f32);
    fn write_svg<W: Write>(&self, w: &mut W, fill: &str, stroke: &str) -> io::Result<()>;
}

#[derive(Debug, Clone)]
pub struct PackShape<S: Shape> {
    container: S,
    children: Vec<PackShape<Circle>>,

    occupied_area: f32,
    pub color: usize,
}

pub struct Settings {
    pub min_radius: f32,
    pub padding: f32,

    pub palette: &'static [&'static str],

    pub target_area: f32,
    pub max_stall_iterations: usize,
}

impl<S: Shape> PackShape<S> {
    pub fn new(shape: S) -> Self {
        Self {
            container: shape,
            children: vec![],
            occupied_area: 0.0,
            color: 0,
        }
    }

    pub fn children(&self) -> &[PackShape<Circle>] {
        &self.children
    }

    pub fn occupied_area(&self) -> f32 {
        self.occupied_area
    }

    pub fn pack(&mut self, mut shape: PackShape<Circle>, cfg: &Settings) -> bool {
        for c in self.children.iter_mut() {
            let (x, y) = shape.center();
            let d = c.sdf(x, y);

            if d < -cfg.padding {
                shape.set_radius(-d - cfg.padding);
                shape.color = (shape.color + 1) % cfg.palette.len();
                return c.pack(shape, cfg);
            }

            if d - cfg.padding < shape.get_radius() {
                shape.set_radius(d - cfg.padding);
            }
        }

        if shape.get_radius() >= cfg.min_radius {
            self.occupied_area += shape.area();
            self.children.push(shape);
            return true;
        }

        false
    }
}

impl PackShape<Circle> {
    pub fn circle(x: f32, y: f32, r: f32) -> Self {
        PackShape::new(Circle::new(x, y, r))
    }

    pub fn set_radius(&mut self, r: f32) {
        self.container.radius = r;
    }

    pub fn get_radius(&mut self) -> f32 {
        self.container.radius
    }
}

impl<S: Shape> Shape for PackShape<S> {
    fn bbox(&self) -> Bbox {
        self.container.bbox()
    }
    fn center(&self) -> (f32, f32) {
        self.container.center()
    }
    fn area(&self) -> f32 {
        self.container.area()
    }
    fn sdf(&self, x: f32, y: f32) -> f32 {
        self.container.sdf(x, y)
    }
    fn random_point<R: Rng>(&self, rng: &mut R) -> (f32, f32) {
        self.container.random_point(rng)
    }
    fn write_svg<W: Write>(&self, w: &mut W, fill: &str, stroke: &str) -> io::Result<()> {
        self.container.write_svg(w, fill, stroke)
    }
}
