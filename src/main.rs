extern crate rand;
use rand::{random, thread_rng, Rng};

extern crate image;
use image::{ImageBuffer, Rgb};

use std::cmp::min;

type Color = (u8, u8, u8);

const NEEDED_NONZERO: u16 = 5;

fn best(scores: &[f64]) -> u8 {
    let mut indices: Vec<u8> = if scores.len() == 256 {
        (0..=255).collect()
    } else {
        (0..scores.len() as u8).collect()
    };
    thread_rng().shuffle(&mut indices);
    indices
        .into_iter()
        .max_by(|i, j| {
            scores[*i as usize]
                .partial_cmp(&scores[*j as usize])
                .unwrap()
        })
        .unwrap()
}

fn dist(a: u8, b: u8) -> f64 {
    min(a.wrapping_sub(b), b.wrapping_sub(a)) as f64
}

fn paint(size: usize) -> Vec<Vec<Color>> {
    let mut rng = thread_rng();
    let mut canvas: Vec<Vec<Option<Color>>> = vec![vec![None; size]; size];
    let positions: Vec<(usize, usize)> = {
        let mut positions: Vec<(usize, usize)> = (0..size)
            .flat_map(|i| (0..size).map(move |j| (i, j)))
            .collect();
        rng.shuffle(&mut positions);
        positions
    };
    for (i, &(x, y)) in positions.iter().enumerate() {
        let fraction: f64 = i as f64 / positions.len() as f64;
        let needed_squares = f64::from(NEEDED_NONZERO) / fraction;
        let min_side = needed_squares.sqrt().ceil() as u8;
        let max_radius = ((size/2) as f64).sqrt().ceil().min(255.0) as u8;
        let radius = min(min_side / 2, max_radius);
        let mut neighbors = vec!();
        for dx in -(radius as isize)..=radius as isize {
            for dy in -(radius as isize)..=radius as isize {
                let new_x = (((x as isize + dx) + size as isize) % size as isize) as usize;
                let new_y = (((y as isize + dy) + size as isize) % size as isize) as usize;
                if canvas[new_x][new_y].is_some() {
                    let distance = (dx * dx + dy * dy) as f64;
                    neighbors.push((new_x, new_y, distance));
                }
            }
        }
        let mut current: Color = (0, 0, 0);
        for iter in 0..7 {
            let width: u8 = 128 >> iter;
            let mut points: Vec<f64> = vec![0.0; 8];
            for &(new_x, new_y, distance) in &neighbors {
                let (red, blue, green) = canvas[new_x][new_y].unwrap();
                for p_i in 0..8 {
                    let color: Color = (
                        current.0 + (p_i & 1) as u8 * width + width/2,
                        current.1 + (p_i & 2) as u8 * width + width/2,
                        current.2 + (p_i & 4) as u8 * width + width/2,
                    );
                    let color_distance: f64 = {
                        let dr = dist(red, color.0);
                        let db = dist(blue, color.1);
                        let dg = dist(green, color.2);
                        dr * dr + db * db + dg * dg
                    };
                    let value = (color_distance as f64) / distance;
                    points[p_i] -= value;
                }
            }
            let best_point = best(&points);
            current = (
                current.0 + (best_point & 1) as u8 * width,
                current.1 + (best_point & 2) as u8 * width,
                current.2 + (best_point & 4) as u8 * width,
            );
        }
        canvas[x][y] = Some(current);
    }
    canvas
        .iter()
        .map(|row| row.iter().map(|elem| elem.unwrap()).collect())
        .collect()
}

fn main() {
    let size: usize = std::env::args().nth(1).unwrap().parse().unwrap();
    let mut img = ImageBuffer::new(size as u32, size as u32);
    let canvas = paint(size);
    for (x, row) in canvas.into_iter().enumerate() {
        for (y, c) in row.into_iter().enumerate() {
            let pixel = Rgb([c.0, c.1, c.2]);
            img.put_pixel(x as u32, y as u32, pixel);
        }
    }
    let filename = format!("pic{}-{}.png", size, random::<u32>());
    img.save(&filename).unwrap();
    println!("{}", filename);
}
