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

#[derive(Clone, Debug)]
pub struct Polyline {
    points: Vec<(f32, f32)>,
    holes: Vec<Polyline>,
    cx: f32,
    cy: f32,
    bbox: Bbox,
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

impl Polyline {
    pub fn new(points: Vec<(f32, f32)>) -> Option<Self> {
        if points.is_empty() {
            None
        } else {
            let (mut cx, mut cy) = points[0];
            let mut bbox = Bbox::new(cx, cy);
            for &(x, y) in points.iter().skip(1) {
                bbox.expand(x, y);
                cx += x;
                cy += y;
            }
            cx /= points.len() as f32;
            cy /= points.len() as f32;
            Some(Self {
                points,
                cx,
                cy,
                bbox,
                holes: vec![],
            })
        }
    }

    /// as of now, hole must be completely contained in the polyline otherwise
    /// it won't be added to the list of holes. This is to ensure the naive area
    /// calculation is correct.
    //
    // TODO: clipping
    pub fn push_hole(&mut self, hole: Polyline) -> bool {
        let completely_contained = hole.points.iter().all(|&(x, y)| self.sdf(x, y) < 0.0);
        if !completely_contained {
            return false;
        }

        self.holes.push(hole);
        true
    }

    fn get_d(&self) -> String {
        let mut d = format!("M {},{}", self.points[0].0, self.points[0].1);
        for &(x, y) in self.points.iter().skip(1) {
            d += &format!("L {},{}", x, y);
        }
        d += "Z";

        for hole in &self.holes {
            d += &hole.get_d();
        }

        d
    }
}

impl Shape for Polyline {
    fn bbox(&self) -> Bbox {
        let mut bbox = Bbox::new(self.points[0].0, self.points[0].1);
        for &(x, y) in self.points.iter().skip(1) {
            bbox.expand(x, y);
        }
        bbox
    }

    fn center(&self) -> (f32, f32) {
        (self.cx, self.cy)
    }

    fn sdf(&self, x: f32, y: f32) -> f32 {
        let (x0, y0) = self.points[0];
        let mut d = (x - x0).powi(2) + (y - y0).powi(2);
        let mut s = 1.0;

        for i in 0..self.points.len() {
            let j = (i + self.points.len() - 1) % self.points.len();

            let (ix, iy) = self.points[i];
            let (jx, jy) = self.points[j];
            let (ex, ey) = (jx - ix, jy - iy);
            let (wx, wy) = (x - ix, y - iy);

            let t = ((wx * ex + wy * ey) / (ex.powi(2) + ey.powi(2)))
                .max(0.0)
                .min(1.0);

            let (bx, by) = (wx - ex * t, wy - ey * t);

            d = d.min(bx.powi(2) + by.powi(2));

            let a = y >= iy;
            let b = y < jy;
            let c = ex * wy > ey * wx;
            if (a && b && c) || (!a && !b && !c) {
                s *= -1.0;
            }
        }

        let mut d = s * d.sqrt();
        for h in &self.holes {
            let dd = h.sdf(x, y);
            d = d.max(-dd);
        }

        d
    }

    fn area(&self) -> f32 {
        let mut area = 0.0;

        for i in 0..self.points.len() {
            let j = (i + 1) % self.points.len();

            let (x0, y0) = self.points[i];
            let (x1, y1) = self.points[j];
            area += x0 * y1 - x1 * y0;
        }

        area = area.abs() / 2.0;

        area - self.holes.iter().map(|h| h.area()).sum::<f32>()
    }

    fn random_point<R: Rng>(&self, rng: &mut R) -> (f32, f32) {
        loop {
            let (x, y) = self.bbox.random_point(rng);
            if self.sdf(x, y) <= 0.0 {
                break (x, y);
            }
        }
    }

    fn write_svg<W: Write>(&self, w: &mut W, fill: &str, stroke: &str) -> io::Result<()> {
        writeln!(
            w,
            r#"<path d="{}" fill="{}" stroke="{}"/>"#,
            self.get_d(),
            fill,
            stroke
        )
    }
}
