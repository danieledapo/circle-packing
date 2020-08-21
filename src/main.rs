use std::{
    fs::File,
    io::{self, BufWriter, Write},
};

use rand::prelude::*;

use circle_packing::{Bbox, PackShape, Settings, Shape};

fn main() {
    let mut rng = thread_rng();

    let settings = Settings {
        min_radius: 5.0,
        padding: 5.0,
        inside: true,
        palette: &["black", "white"],
        target_area: 0.8,
        max_stall_iterations: 1000,
    };

    assert!(settings.padding >= 0.0);

    let container = {
        let mut b = Bbox::new(0.0, 0.0);
        b.expand(1920.0, 1080.0);
        b
    };
    let mut root = PackShape::new(container);
    root.color = 1 % settings.palette.len();

    let target_area = settings.target_area * root.area();

    let mut stall_i = 0;
    while root.occupied_area() < target_area {
        let (x, y) = root.random_point(&mut rng);
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

    let f = File::create("packing.svg").unwrap();
    let mut bf = BufWriter::new(f);
    dump_svg(&mut bf, &root, &settings).unwrap();
}

pub fn dump_svg<S: Shape>(
    out: &mut impl Write,
    root: &PackShape<S>,
    cfg: &Settings,
) -> io::Result<()> {
    let bbox = root.bbox();

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

    root.write_svg(out, cfg.palette[root.color], "none")?;

    let mut stack: Vec<_> = root.children().iter().collect();
    while let Some(c) = stack.pop() {
        c.write_svg(out, cfg.palette[c.color], "none")?;
        stack.extend(c.children());
    }

    writeln!(out, "</svg>")
}
