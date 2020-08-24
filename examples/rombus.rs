use std::{fs::File, io::BufWriter};

use rand::prelude::*;

use circle_packing::*;

pub fn main() {
    let mut rng = thread_rng();

    let settings = Settings {
        min_radius: 5.0,
        padding: 5.0,
        inside: true,
        palette: &["#ec6c26", "#613a53", "#e8ac52", "#639aa0"],
        target_area: 0.8,
        max_stall_iterations: 1000,
    };

    let mut container = Polyline::new(vec![
        (0.0, -250.0), //
        (500.0, 0.0),  //
        (0.0, 250.0),  //
        (-500.0, 0.0), //
    ])
    .unwrap();

    let mut hole = Polyline::new(vec![
        (0.0, -150.0), //
        (400.0, 0.0),  //
        (0.0, 150.0),  //
        (-400.0, 0.0), //
    ])
    .unwrap();

    hole.push_hole(
        Polyline::new(vec![
            (0.0, -50.0),  //
            (300.0, 0.0),  //
            (0.0, 50.0),   //
            (-300.0, 0.0), //
        ])
        .unwrap(),
    );

    container.push_hole(hole);

    let mut root = PackShape::new(container);
    root.color = 1 % settings.palette.len();

    circle_packing::pack(&mut root, &settings, &mut rng);

    let f = File::create("packed_rombus.svg").unwrap();
    let mut bf = BufWriter::new(f);
    dump_svg(&mut bf, &[root], &settings).unwrap();
}
