use crate::core::analysis::FileAnalysis;
use anyhow::{Context, Result};
use image::{Rgb, RgbImage};
use std::path::Path;

const WIDTH: u32 = 960;
const HEIGHT: u32 = 540;

pub fn write_png_report(path: &Path, analysis: &FileAnalysis) -> Result<()> {
    let mut img = RgbImage::from_pixel(WIDTH, HEIGHT, Rgb([250, 250, 252]));

    draw_panels(&mut img);
    draw_heatmap(&mut img, analysis);
    draw_stress_curve(&mut img, analysis);
    draw_metric_bars(&mut img, analysis);

    img.save(path)
        .with_context(|| format!("failed to write PNG report: {}", path.display()))?;
    Ok(())
}

fn draw_panels(img: &mut RgbImage) {
    fill_rect(img, 0, 0, WIDTH as i32, HEIGHT as i32, [248, 249, 252]);
    fill_rect(img, 24, 24, 430, 492, [255, 255, 255]);
    fill_rect(img, 478, 24, 458, 234, [255, 255, 255]);
    fill_rect(img, 478, 282, 458, 234, [255, 255, 255]);

    stroke_rect(img, 24, 24, 430, 492, [210, 214, 220]);
    stroke_rect(img, 478, 24, 458, 234, [210, 214, 220]);
    stroke_rect(img, 478, 282, 458, 234, [210, 214, 220]);
}

fn draw_heatmap(img: &mut RgbImage, analysis: &FileAnalysis) {
    let metrics = [
        normalize_score(analysis.physical_model.decoherence_rate, 0.0, 0.10),
        normalize_score(analysis.physical_model.von_neumann_entropy, 0.0, 2.0),
        normalize_score(analysis.physical_model.global_constraint_penalty, 0.0, 2.0),
        normalize_score(analysis.solver_summary.collapse_probability, 0.0, 1.0),
        normalize_score(analysis.singularity_risk, 0.0, 1.0),
    ];

    let blocks = &analysis.physical_model.blocks;
    if blocks.is_empty() {
        return;
    }

    let cell_w = 72;
    let cell_h = 76;
    let start_x = 48;
    let start_y = 72;

    for (row, block) in blocks.iter().enumerate() {
        let block_scale = normalize_score(block.information_density, 0.0, 1.0);
        for (col, metric) in metrics.iter().enumerate() {
            let mix = ((*metric * 0.65) + (block_scale * 0.35)).clamp(0.0, 1.0);
            let color = heat_color(mix);
            let x = start_x + (col as i32 * cell_w);
            let y = start_y + (row as i32 * cell_h);
            fill_rect(img, x, y, cell_w - 8, cell_h - 8, color);
            stroke_rect(img, x, y, cell_w - 8, cell_h - 8, [238, 240, 244]);
        }
    }
}

fn draw_stress_curve(img: &mut RgbImage, analysis: &FileAnalysis) {
    let x0 = 504;
    let y0 = 48;
    let w = 406;
    let h = 186;

    draw_axes(img, x0, y0, w, h);

    let values = [
        analysis.solver_summary.p05_stress,
        analysis.solver_summary.p50_stress,
        analysis.solver_summary.p95_stress,
    ];

    let max_v = values
        .iter()
        .fold(0.0_f64, |acc, v| if *v > acc { *v } else { acc })
        .max(1.0);

    let points = [
        (
            x0 + 28,
            y0 + h - 24 - ((values[0] / max_v) * (h as f64 - 48.0)).round() as i32,
        ),
        (
            x0 + (w / 2),
            y0 + h - 24 - ((values[1] / max_v) * (h as f64 - 48.0)).round() as i32,
        ),
        (
            x0 + w - 28,
            y0 + h - 24 - ((values[2] / max_v) * (h as f64 - 48.0)).round() as i32,
        ),
    ];

    draw_line(
        img,
        points[0].0,
        points[0].1,
        points[1].0,
        points[1].1,
        [54, 104, 201],
    );
    draw_line(
        img,
        points[1].0,
        points[1].1,
        points[2].0,
        points[2].1,
        [54, 104, 201],
    );

    for point in points {
        fill_circle(img, point.0, point.1, 4, [32, 74, 161]);
    }
}

fn draw_metric_bars(img: &mut RgbImage, analysis: &FileAnalysis) {
    let x0 = 504;
    let y0 = 306;
    let w = 406;
    let h = 186;

    draw_axes(img, x0, y0, w, h);

    let items = [
        (
            "decoherence",
            normalize_score(analysis.physical_model.decoherence_rate, 0.0, 0.10),
        ),
        (
            "entropy",
            normalize_score(analysis.physical_model.von_neumann_entropy, 0.0, 2.0),
        ),
        (
            "collapse",
            normalize_score(analysis.solver_summary.collapse_probability, 0.0, 1.0),
        ),
        ("risk", normalize_score(analysis.singularity_risk, 0.0, 1.0)),
        (
            "stability",
            normalize_score(analysis.stability_score, 0.0, 100.0),
        ),
    ];

    let bar_w = 50;
    let gap = 22;
    let base_x = x0 + 34;
    let max_bar_h = h - 42;

    for (idx, (_, value)) in items.iter().enumerate() {
        let bh = ((*value) * max_bar_h as f64).round() as i32;
        let x = base_x + idx as i32 * (bar_w + gap);
        let y = y0 + h - 20 - bh;
        let color = bar_color(idx);
        fill_rect(img, x, y, bar_w, bh, color);
    }
}

fn draw_axes(img: &mut RgbImage, x: i32, y: i32, w: i32, h: i32) {
    draw_line(
        img,
        x + 18,
        y + h - 20,
        x + w - 12,
        y + h - 20,
        [160, 165, 175],
    );
    draw_line(img, x + 18, y + 12, x + 18, y + h - 20, [160, 165, 175]);
}

fn normalize_score(value: f64, min: f64, max: f64) -> f64 {
    if max <= min {
        return 0.0;
    }
    ((value - min) / (max - min)).clamp(0.0, 1.0)
}

fn heat_color(v: f64) -> [u8; 3] {
    let t = v.clamp(0.0, 1.0);
    let r = (30.0 + t * 210.0).round() as u8;
    let g = (120.0 + (1.0 - t) * 90.0).round() as u8;
    let b = (210.0 - t * 150.0).round() as u8;
    [r, g, b]
}

fn bar_color(index: usize) -> [u8; 3] {
    match index {
        0 => [78, 121, 167],
        1 => [242, 142, 43],
        2 => [225, 87, 89],
        3 => [118, 183, 178],
        _ => [89, 161, 79],
    }
}

fn fill_rect(img: &mut RgbImage, x: i32, y: i32, w: i32, h: i32, color: [u8; 3]) {
    if w <= 0 || h <= 0 {
        return;
    }

    let x_start = x.max(0) as u32;
    let y_start = y.max(0) as u32;
    let x_end = (x + w).min(WIDTH as i32).max(0) as u32;
    let y_end = (y + h).min(HEIGHT as i32).max(0) as u32;

    for yy in y_start..y_end {
        for xx in x_start..x_end {
            img.put_pixel(xx, yy, Rgb(color));
        }
    }
}

fn stroke_rect(img: &mut RgbImage, x: i32, y: i32, w: i32, h: i32, color: [u8; 3]) {
    draw_line(img, x, y, x + w, y, color);
    draw_line(img, x, y, x, y + h, color);
    draw_line(img, x + w, y, x + w, y + h, color);
    draw_line(img, x, y + h, x + w, y + h, color);
}

fn draw_line(img: &mut RgbImage, mut x0: i32, mut y0: i32, x1: i32, y1: i32, color: [u8; 3]) {
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        put_pixel_safe(img, x0, y0, color);
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            if x0 == x1 {
                break;
            }
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            if y0 == y1 {
                break;
            }
            err += dx;
            y0 += sy;
        }
    }
}

fn fill_circle(img: &mut RgbImage, cx: i32, cy: i32, radius: i32, color: [u8; 3]) {
    for y in -radius..=radius {
        for x in -radius..=radius {
            if x * x + y * y <= radius * radius {
                put_pixel_safe(img, cx + x, cy + y, color);
            }
        }
    }
}

fn put_pixel_safe(img: &mut RgbImage, x: i32, y: i32, color: [u8; 3]) {
    if x >= 0 && y >= 0 && x < WIDTH as i32 && y < HEIGHT as i32 {
        img.put_pixel(x as u32, y as u32, Rgb(color));
    }
}
