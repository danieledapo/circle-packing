use std::{fs::File, io::BufWriter, path::PathBuf};

use rand::prelude::*;
use structopt::StructOpt;

use circle_packing::{self, Bbox, PackShape, Settings};

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

    let mut root = PackShape::new(container);
    root.color = 1 % settings.palette.len();

    circle_packing::pack(&mut root, &settings, &mut rng);

    let f = File::create(app.output).unwrap();
    let mut bf = BufWriter::new(f);
    circle_packing::dump_svg(&mut bf, &[root], &settings).unwrap();
}
