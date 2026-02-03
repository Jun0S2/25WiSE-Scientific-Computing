use image::{ImageBuffer, Luma, imageops};
use std::collections::VecDeque;
use std::fs;

const COLS: usize = 30;
const ROWS: usize = 16;
const OUTPUT_SIZE: u32 = 28;

pub fn split_all_images() -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all("data/split")?;
    // Label Order : 1, 2, ..., 9, 0, A, B, ..., F (16)
    let labels = ['1','2','3','4','5','6','7','8','9','0','A','B','C','D','E','F'];

    let mut entries: Vec<_> = fs::read_dir("downloads")?.filter_map(Result::ok)
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            name.ends_with(".png") && !name.contains("_rotated")
        })
        .collect();
    entries.sort_by_key(|e| e.file_name());

    let mut global = 0;
    for entry in entries {
        let path = entry.path();
        let file_stem = path.file_stem().unwrap().to_string_lossy().into_owned();
        println!("--- Processing: {}.png ---", file_stem);
        
        let mut img = image::open(&path)?.to_luma8();
        let mut boxes = find_components(&img);

        // 1. Decide whether to rotate or not
        if need_rotate_90(&boxes) {
            println!("🔄 [ACTION] Rotating 90° CCW");
            img = imageops::rotate270(&img); 
            // find the box again after rotation to maintain the positions
            boxes = find_components(&img);
        }

        if boxes.len() < 100 { continue; }

        // 2. Find the bounding box of the character areas (excluding noise)
        let mut xs: Vec<u32> = boxes.iter().map(|b| (b.0 + b.2)/2).collect();
        let mut ys: Vec<u32> = boxes.iter().map(|b| (b.1 + b.3)/2).collect();
        xs.sort(); ys.sort();
        
        // Use 5 ~ 95 percentiles to avoid extreme noise
        let min_x = xs[xs.len() / 20]; 
        let max_x = xs[xs.len() - 1 - (xs.len() / 20)];
        let min_y = ys[ys.len() / 20];
        let max_y = ys[ys.len() - 1 - (ys.len() / 20)];

        let grid_w = (max_x - min_x) as f32 / (COLS - 1) as f32;
        let grid_h = (max_y - min_y) as f32 / (ROWS - 1) as f32;

        // 3. 16x30 grid matching
        for r in 0..ROWS {
            let label = labels[r]; // fix label per row
            let target_y = min_y as f32 + (r as f32 * grid_h);

            for c in 0..COLS {
                let target_x = min_x as f32 + (c as f32 * grid_w);

                // find the closest box to (target_x, target_y)
                if let Some(best_box) = boxes.iter().min_by_key(|b| {
                    let cx = (b.0 + b.2) / 2;
                    let cy = (b.1 + b.3) / 2;
                    ((cx as i32 - target_x as i32).pow(2) + (cy as i32 - target_y as i32).pow(2))
                }) {
                    let cx = (best_box.0 + best_box.2) / 2;
                    let cy = (best_box.1 + best_box.3) / 2;
                    
                    // Ignore boxes that are too far from the target grid point (its prolly grid or other char)
                    if (cx as i32 - target_x as i32).abs() < (grid_w * 0.6) as i32 && 
                       (cy as i32 - target_y as i32).abs() < (grid_h * 0.6) as i32 {
                        
                        let (x1, y1, x2, y2) = *best_box;
                        let cell = imageops::crop_imm(&img, x1, y1, x2 - x1 + 1, y2 - y1 + 1).to_image();
                        let resized = imageops::resize(&cell, OUTPUT_SIZE, OUTPUT_SIZE, imageops::FilterType::Nearest);

                        save_as_pbm(&resized, &format!("data/split/{}_{:05}.pbm", label, global))?;
                        global += 1;
                    }
                }
            }
        }
    }
    println!("✨ Final Complete. Total PBMs: {}", global);
    Ok(())
}

fn need_rotate_90(boxes: &[(u32, u32, u32, u32)]) -> bool {
    if boxes.len() < 50 { return false; }
    let mut h_votes = 0; let mut v_votes = 0;
    // Increase sample size for better accuracy
    let sample = if boxes.len() > 400 { &boxes[..400] } else { boxes };
    for i in 0..sample.len() {
        let (cx1, cy1) = ((sample[i].0 + sample[i].2)/2, (sample[i].1 + sample[i].3)/2);
        for j in 0..sample.len() {
            if i == j { continue; }
            let (cx2, cy2) = ((sample[j].0 + sample[j].2)/2, (sample[j].1 + sample[j].3)/2); // center points
            let dx = (cx1 as i32 - cx2 as i32).abs();
            let dy = (cy1 as i32 - cy2 as i32).abs();
            if dy < 15 && dx < 150 { h_votes += 1; }
            if dxEdge(dx, dy) { v_votes += 1; }
        }
    }
    fn dxEdge(dx: i32, dy: i32) -> bool { dx < 15 && dy < 150 } // helper for vertical check
    v_votes > h_votes
}

fn find_components(img: &ImageBuffer<Luma<u8>, Vec<u8>>) -> Vec<(u32,u32,u32,u32)> {
    let (w, h) = img.dimensions();
    let mut visited = vec![false; (w * h) as usize];
    let mut boxes = vec![];
    for y in 0..h {
        for x in 0..w {
            let idx = (y * w + x) as usize;
            if visited[idx] || img.get_pixel(x, y)[0] > 180 { continue; } 
            let mut q = VecDeque::new();
            q.push_back((x, y));
            visited[idx] = true;
            let (mut minx, mut maxx, mut miny, mut maxy) = (x, x, y, y);
            let mut area = 0;
            while let Some((cx, cy)) = q.pop_front() {
                area += 1;
                minx = minx.min(cx); maxx = maxx.max(cx);
                miny = miny.min(cy); maxy = maxy.max(cy);
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        let nx = cx as i32 + dx; let ny = cy as i32 + dy;
                        if nx >= 0 && nx < w as i32 && ny >= 0 && ny < h as i32 {
                            let n_idx = (ny as u32 * w + nx as u32) as usize;
                            if !visited[n_idx] && img.get_pixel(nx as u32, ny as u32)[0] <= 180 {
                                visited[n_idx] = true;
                                q.push_back((nx as u32, ny as u32));
                            }
                        }
                    }
                }
            }
            // Reduce noise by neglecting too small areas
            // 0 was getting filled with noise, so increased the threshold
            if area > 120 && area < 20000 { boxes.push((minx, miny, maxx, maxy)); }
        }
    }
    boxes
}

fn save_as_pbm(img: &ImageBuffer<Luma<u8>, Vec<u8>>, path: &str) -> Result<(), std::io::Error> {
    use std::io::Write;
    let mut f = fs::File::create(path)?;
    writeln!(f, "P1")?;
    writeln!(f, "{} {}", img.width(), img.height())?;
    for y in 0..img.height() {
        for x in 0..img.width() {
            let v = if img.get_pixel(x, y)[0] < 180 { 1 } else { 0 };
            write!(f, "{} ", v)?;
        }
        writeln!(f)?;
    }
    Ok(())
}