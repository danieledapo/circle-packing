use std::{
    fs::File,
    io::{self, BufWriter, Write},
};

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

    let mut logo_paths = load_logo();

    for root in &mut logo_paths {
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
    }

    let f = File::create("packed_logo.svg").unwrap();
    let mut bf = BufWriter::new(f);
    dump_svg(&mut bf, &logo_paths, &settings).unwrap();
}

fn load_logo() -> Vec<PackShape<Polyline>> {
    let svg_data = [
        "M 1211.4498,144.0724 L 1154.7727,168.63671 L 1132.771,231.9025 L 1139.6796,271.23937 L 1161.035,306.24543 L 1137.4318,329.2032 V 369.66616 L 1183.1912,324.8677 L 1224.1349,348.15034 L 1266.045,358.42813 V 328.56281 L 1231.3583,322.30218 L 1200.8521,307.52992 L 1284.3465,225.96267 L 1307.6293,206.53507 L 1324.1691,201.88254 L 1347.126,212.31931 L 1356.1222,239.77744 L 1352.1074,263.38057 L 1340.7035,290.03154 H 1369.9285 L 1378.5987,263.05383 L 1381.4925,236.56282 L 1365.5933,189.19664 L 1323.8488,171.37559 L 1304.095,175.38217 L 1284.0263,187.10656 L 1249.9857,154.83343 L 1211.4498,144.07491 Z M 1213.3772,175.05978 L 1240.3545,181.8049 L 1266.3654,203.3257 L 1179.1764,288.42221 L 1163.7632,263.37809 L 1158.7872,234.79384 L 1174.3606,192.08178 L 1213.3769,175.06229 Z", //
        "M 1203.9061,405.1539 V 455.72879 L 1137.432,439.03387 V 464.88453 L 1203.9061,481.41923 V 528.46488 L 1137.432,511.93113 V 537.61563 L 1203.9061,554.31619 V 611.63899 H 1228.4701 V 560.57838 L 1282.0989,573.74409 V 630.10632 H 1306.8286 V 580.00598 L 1373.4629,596.54688 V 570.85614 L 1306.8286,553.99566 V 507.10993 L 1373.4629,523.80954 V 497.79834 L 1306.8286,481.41923 V 423.45541 H 1282.0989 V 475.15639 L 1228.4701,461.67083 V 405.1539 Z M 1228.4701,487.68142 L 1282.0989,501.16763 V 547.89343 L 1228.4701,534.72802 Z", //
        "M 687.94065,161.08786 V 654.28448 L 568.30716,533.8331 L 395.76826,493.95545 L 269.52907,516.51549 L 126.38169,625.79888 L 23.022893,972.50053 L 126.38169,1319.2022 L 395.76826,1451.0457 L 568.30716,1411.9854 L 687.94065,1290.7166 V 1427.4475 H 837.69381 V 161.08786 Z M 432.39336,619.28839 L 619.57793,713.69523 L 687.94065,972.50053 L 619.57793,1232.118 L 432.39336,1325.7127 L 245.2034,1232.118 L 177.6579,972.50053 L 245.2034,713.69523 Z", //
        "M 1144.6554,680.36106 L 1135.825,713.27514 L 1132.7712,744.42746 L 1152.2048,813.78923 L 1207.1149,838.19891 L 1245.971,825.0332 L 1266.6858,788.74512 L 1285.6335,821.1785 L 1319.3538,832.5774 L 1364.6326,811.06128 L 1381.4927,752.617 L 1378.9197,722.11099 L 1371.2159,687.58444 H 1342.3112 L 1351.3019,721.14483 L 1354.1957,749.72318 L 1344.2387,787.13826 L 1315.9801,800.30398 L 1288.3623,787.61903 L 1278.8911,751.01015 V 721.78521 H 1252.2342 V 749.72318 L 1240.1951,790.9933 L 1207.1155,805.92548 L 1172.1141,789.86626 L 1160.0694,743.46624 L 1164.0843,709.9068 L 1175.9686,680.36106 Z", //
        "M 1299.92,885.24551 L 1240.3545,906.11379 L 1218.6788,963.11769 L 1226.062,994.7504 L 1247.2572,1018.3535 L 1181.5839,998.92501 L 1160.0683,949.95198 L 1163.1221,925.38238 L 1171.9525,900.65815 H 1142.4072 L 1135.1839,926.66935 L 1132.7709,951.39891 L 1166.4912,1023.6496 L 1257.0487,1050.627 L 1349.5393,1028.6307 L 1381.4924,965.20496 L 1359.1705,907.08122 L 1299.9198,885.24551 Z M 1299.92,917.83848 L 1340.7035,930.52343 L 1355.8019,965.20496 L 1340.7035,999.72657 L 1299.92,1012.5711 L 1258.9763,999.72657 L 1244.0435,965.20496 L 1258.9763,930.52343 Z", //
        "M 1266.6856,1112.2852 V 1146.1602 H 1307.4692 V 1112.2852 Z", //
        "M 1099.2165,1099.1145 V 1119.8291 L 1150.5976,1146.1602 H 1178.2155 V 1112.2852 H 1150.5976 Z", //
    ];

    svg_data
        .iter()
        .map(|d| {
            let mut paths = read_d(d);

            // consider first path as boundary and everything else as holes in it.
            // not correct given that svg uses orientation, but meh

            let mut boundary = Polyline::new(paths.swap_remove(0)).unwrap();
            for hole in paths {
                boundary.push_hole(Polyline::new(hole).unwrap());
            }

            dbg!(&boundary);

            PackShape::new(boundary)
        })
        .collect()
}

fn read_d(d: &str) -> Vec<Vec<(f32, f32)>> {
    let d = d.trim();

    let mut paths = vec![];
    let mut path = vec![];

    let mut it = d.split_ascii_whitespace();
    while let Some(cmd) = it.next() {
        if cmd == "M" || cmd == "L" {
            let data = it.next().unwrap();

            let mut parts = data.split(',');
            let x: f32 = parts.next().unwrap().parse().unwrap();
            let y: f32 = parts.next().unwrap().parse().unwrap();
            path.push((x, y));
            continue;
        }

        if cmd == "V" {
            let x = path.last().unwrap().0;
            let y = it.next().unwrap().parse().unwrap();
            path.push((x, y));
            continue;
        }

        if cmd == "H" {
            let x = it.next().unwrap().parse().unwrap();
            let y = path.last().unwrap().1;
            path.push((x, y));
            continue;
        }

        if cmd == "Z" {
            path.push(path[0]);
            paths.push(path);
            path = vec![];
            continue;
        }

        panic!("unsupported cmd {}", cmd);
    }

    if !path.is_empty() {
        paths.push(path);
    }

    paths
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
