use std::{
    fmt::Debug,
    io::{self, Write},
};

use rand::prelude::*;

pub mod shapes;
pub use shapes::{Bbox, Circle, Polyline};

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
    pub inside: bool,

    pub palette: &'static [&'static str],

    pub target_area: f32,
    pub max_stall_iterations: usize,
}

pub fn pack(root: &mut PackShape<impl Shape>, settings: &Settings, rng: &mut impl Rng) {
    let target_area = settings.target_area * root.area();

    let mut stall_i = 0;
    while root.occupied_area() < target_area {
        let (x, y) = root.random_point(rng);
        let radius = -root.sdf(x, y) - settings.padding;

        let stall = !root.pack(PackShape::circle(x, y, radius), &settings);

        if stall {
            stall_i += 1;
            if stall_i >= settings.max_stall_iterations {
                break;
            }
        } else {
            stall_i = 0;
        }
    }
}

pub fn dump_svg<S: Shape>(
    out: &mut impl Write,
    roots: &[PackShape<S>],
    cfg: &Settings,
) -> io::Result<()> {
    let mut bbox = roots[0].bbox();
    for s in &roots[1..] {
        let b = s.bbox();
        bbox.expand(b.x0(), b.y0());
        bbox.expand(b.x0() + b.width(), b.y0() + b.height());
    }

    writeln!(
        out,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE svg PUBLIC "-//W3C//DTD SVG 1.1//EN" "http://www.w3.org/Graphics/SVG/1.1/DTD/svg11.dtd">
<svg xmlns="http://www.w3.org/2000/svg" version="1.1" viewBox="{x} {y} {width} {height}">
<rect x="{x}" y="{y}" width="{width}" height="{height}" stroke="none" fill="{color}" />"#,
        x = bbox.x0(),
        y = bbox.y0(),
        width = bbox.width(),
        height = bbox.height(),
        color = cfg.palette[0],
    )?;

    let mut stack = vec![];

    for root in roots {
        root.write_svg(out, cfg.palette[root.color], "none")?;
        stack.extend(root.children());
    }

    while let Some(c) = stack.pop() {
        c.write_svg(out, cfg.palette[c.color], "none")?;
        stack.extend(c.children());
    }

    writeln!(out, "</svg>")
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

            if cfg.inside && d < -cfg.padding {
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
