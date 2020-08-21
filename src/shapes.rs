use std::{
    f32::consts::PI,
    io::{self, Write},
};

use rand::prelude::*;

use crate::Shape;

#[derive(Clone, Debug)]
pub struct Bbox {
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
}

#[derive(Clone, Debug)]
pub struct Circle {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
}

impl Bbox {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x0: x,
            y0: y,
            x1: x,
            y1: y,
        }
    }

    pub fn expand(&mut self, x: f32, y: f32) {
        self.x0 = self.x0.min(x);
        self.x1 = self.x1.max(x);
        self.y0 = self.y0.min(y);
        self.y1 = self.y1.max(y);
    }

    pub fn x0(&self) -> f32 {
        self.x0
    }
    pub fn y0(&self) -> f32 {
        self.y0
    }
    pub fn width(&self) -> f32 {
        self.x1 - self.x0
    }
    pub fn height(&self) -> f32 {
        self.y1 - self.y0
    }
}

impl Shape for Bbox {
    fn bbox(&self) -> Bbox {
        self.clone()
    }

    fn center(&self) -> (f32, f32) {
        ((self.x0 + self.x1) / 2.0, (self.y0 + self.y1) / 2.0)
    }

    fn sdf(&self, x: f32, y: f32) -> f32 {
        let (cx, cy) = self.center();
        let dx = (x - cx).abs() - self.width() / 2.0;
        let dy = (y - cy).abs() - self.height() / 2.0;

        let out = f32::sqrt(f32::max(dx, 0.0).powi(2) + f32::max(dy, 0.0).powi(2));
        let ins = f32::max(dx, dy).min(0.0);

        out + ins
    }

    fn area(&self) -> f32 {
        self.width() * self.height()
    }

    fn random_point<R: Rng>(&self, rng: &mut R) -> (f32, f32) {
        let x = rng.gen_range(self.x0, self.x1);
        let y = rng.gen_range(self.y0, self.y1);
        (x, y)
    }

    fn write_svg<W: Write>(&self, w: &mut W, fill: &str, stroke: &str) -> io::Result<()> {
        writeln!(
            w,
            r#"<rect x="{}" y="{}" width="{}" height="{}" fill="{}" stroke="{}"/>"#,
            self.x0,
            self.y0,
            self.width(),
            self.height(),
            fill,
            stroke
        )
    }
}

impl Circle {
    pub fn new(x: f32, y: f32, radius: f32) -> Self {
        Self { x, y, radius }
    }
}

impl Shape for Circle {
    fn bbox(&self) -> Bbox {
        let mut bbox = Bbox::new(self.x - self.radius, self.y - self.radius);
        bbox.expand(self.x + self.radius, self.y + self.radius);
        bbox
    }

    fn center(&self) -> (f32, f32) {
        (self.x, self.y)
    }

    fn sdf(&self, x: f32, y: f32) -> f32 {
        let d2 = (self.x - x).powi(2) + (self.y - y).powi(2);
        d2.sqrt() - self.radius
    }

    fn area(&self) -> f32 {
        PI * self.radius.powi(2)
    }

    fn random_point<R: Rng>(&self, rng: &mut R) -> (f32, f32) {
        let a = rng.gen_range(0.0, 2.0 * PI);
        let d = rng.gen_range(0.0, self.radius);

        let x = a.cos() * d;
        let y = a.sin() * d;

        (x, y)
    }

    fn write_svg<W: Write>(&self, w: &mut W, fill: &str, stroke: &str) -> io::Result<()> {
        writeln!(
            w,
            r#"<circle cx="{}" cy="{}" r="{}" fill="{}" stroke="{}"/>"#,
            self.x, self.y, self.radius, fill, stroke
        )
    }
}
