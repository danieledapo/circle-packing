use std::{
    fs::File,
    io::{self, BufWriter, Write},
    path::PathBuf,
};

use rand::prelude::*;
use structopt::StructOpt;

use circle_packing::{Bbox, PackShape, Settings, Shape};

type PALETTE = (&'static str, &'static [&'static str]);
static PALETTES: &'static [PALETTE] = &[
    //
    // duo
    //
    ("dt01", &["#172a89", "#f7f7f3"]),
    ("dt02", &["#302956", "#f3c507"]),
    ("dt03", &["#000000", "#a7a7a7"]),
    ("dt04", &["#50978e", "#f7f0df"]),
    ("dt05", &["#ee5d65", "#f0e5cb"]),
    ("dt06", &["#271f47", "#e7ceb5"]),
    ("dt07", &["#6a98a5", "#d24c18"]),
    ("dt08", &["#5d9d88", "#ebb43b"]),
    ("dt09", &["#052e57", "#de8d80"]),
    //
    // rag
    //
    ("rag-mysore", &["#ec6c26", "#613a53", "#e8ac52", "#639aa0"]),
    ("rag-gol", &["#d3693e", "#803528", "#f1b156", "#90a798"]),
    ("rag-belur", &["#f46e26", "#68485f", "#3d273a", "#535d55"]),
    (
        "rag-bangalore",
        &["#ea720e", "#ca5130", "#e9c25a", "#52534f"],
    ),
    ("rag-taj", &["#ce565e", "#8e1752", "#f8a100", "#3ac1a6"]),
    (
        "rag-virupaksha",
        &["#f5736a", "#925951", "#feba4c", "#9d9b9d"],
    ),
];

/// Program to create some SVG images from random circle packing runs.
#[derive(Debug, StructOpt)]
pub struct App {
    /// Show available themes and exit.
    #[structopt(long)]
    list_themes: bool,

    /// Padding between the circles.
    #[structopt(short, long, default_value = "5.0")]
    padding: f32,

    /// Minimum radius all the packed circles must have.
    #[structopt(short = "r", long, default_value = "5.0")]
    min_radius: f32,

    /// Percentage between [0, 1] of the total area that must be packed with
    /// circles.
    #[structopt(long, default_value = "0.8")]
    target_coverage: f32,

    /// Whether circles can contain other non-intersecting circles. This does
    /// not affect the total area covered by circles.
    #[structopt(long)]
    no_inside: bool,

    /// Theme to use when saving the final image.
    #[structopt(short, long)]
    theme: Option<String>,

    /// Width of the image.
    #[structopt(short, long, default_value = "1920")]
    width: u16,

    /// Height of the image.
    #[structopt(short, long, default_value = "1080")]
    height: u16,

    /// Path where to save the image at.
    #[structopt(short, long, default_value = "packing.svg")]
    output: PathBuf,
}

fn main() {
    let mut rng = thread_rng();

    let app = App::from_args();

    if app.list_themes {
        println!("Available themes");
        println!();
        for (name, colors) in PALETTES {
            println!("  {}: {:?}", name, colors);
        }
        println!();
        return;
    }

    let (theme_name, palette) = app
        .theme
        .and_then(|t| {
            let theme = PALETTES.iter().find(|(n, _)| n == &&t);
            if theme.is_none() {
                println!("theme {} not found, using a random one", t);
            }
            theme
        })
        .or_else(|| PALETTES.choose(&mut rng))
        .unwrap();

    println!("using theme {}", theme_name);

    let settings = Settings {
        min_radius: app.min_radius,
        padding: app.padding,
        inside: !app.no_inside,
        palette,
        target_area: app.target_coverage,
        max_stall_iterations: 1000,
    };

    assert!(settings.padding >= 0.0);

    let container = {
        let mut b = Bbox::new(0.0, 0.0);
        b.expand(app.width.into(), app.height.into());
        b
    };
    let container = {
        let points = vec![
            (0.0, -250.0), //
            (500.0, 0.0),  //
            (0.0, 250.0),  //
            (-500.0, 0.0), //
        ];

        // let points = vec![
        //     (500.0, 0.0),    //
        //     (1000.0, 250.0), //
        //     (500.0, 500.0),  //
        //     (0.0, 250.0),    //
        // ];

        // let mut points = vec![];
        // for i in 0_u16..=180 {
        //     let i = f32::from(i);
        //     points.push((
        //         30.0 * i,
        //         800.0 * (8.0 * i / 180.0 * std::f32::consts::PI).sin(),
        //     ));
        // }
        // points.push((30.0 * 180.0 / 2.0, 900.0));

        let mut poly = circle_packing::shapes::Polyline::new(points).unwrap();

        let mut hole = circle_packing::shapes::Polyline::new(vec![
            (0.0, -150.0), //
            (400.0, 0.0),  //
            (0.0, 150.0),  //
            (-400.0, 0.0), //
        ])
        .unwrap();
        hole.push_hole(
            circle_packing::shapes::Polyline::new(vec![
                (0.0, -50.0),  //
                (300.0, 0.0),  //
                (0.0, 50.0),   //
                (-300.0, 0.0), //
            ])
            .unwrap(),
        );
        poly.push_hole(hole);
        poly
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

    let f = File::create(app.output).unwrap();
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
