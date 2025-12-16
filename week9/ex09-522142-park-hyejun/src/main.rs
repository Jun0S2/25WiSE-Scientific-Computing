use std::fs::File;
use std::io::{Write, BufWriter};
use std::time::Instant;

// CRC-32 implementation (from Rosetta Code)
fn crc32(table: &[u32; 256], bytes: &[u8]) -> u32 {
    let mut crc = !0u32;    // Initial value
    for byte in bytes {     // Process each byte
        crc = table[((crc as u8) ^ byte) as usize] ^ (crc >> 8);    // Update CRC value
    }
    !crc                    // Final XOR
}

// Generate CRC-32 lookup table
fn make_crc_table() -> [u32; 256] {
    let mut table = [0u32; 256];    // Initialize table with zeros and size 256
    for i in 0..256 {
        let mut c = i as u32;
        for _ in 0..8 {
            if c & 1 != 0 {                     //Is LSB set?(1)
                c = 0xedb88320 ^ (c >> 1);      // 1: Apply polynomial and shift right
            } else {
                c >>= 1;                        // 0: Just shift right
            }
        }
        table[i] = c;                 
    }
    table
}

// Mandelbrot set computation for a single point
// x² + y² > 4 = |z| > 2 -> escape
fn mandelbrot_iterations(cx: f64, cy: f64, max_iter: usize) -> u8 {
    let mut x = 0.0;        // Real part of z
    let mut y = 0.0;        // Imaginary part of z
    let mut x2 = 0.0;       // x squared
    let mut y2 = 0.0;       // y squared
    
    for i in 0..max_iter {  // Iterate up to max_iter
        if x2 + y2 > 4.0 {  // Escape condition : |z|^2 > 4 
            return i as u8; // Return number of iterations before escape
        }
        
        // Compute next iteration: z = z² + c
        y = 2.0 * x * y + cy; // Imaginary part: 2xy + cy
        x = x2 - y2 + cx;    // Real part: (x² - y²) + cx

        // Update squares
        x2 = x * x;
        y2 = y * y;
    }
    
    max_iter as u8
}

// added color_map function else, the image is plain black and white
fn color_map(iterations: u8, max_iter: u8) -> u8 {
    if iterations == max_iter {
        255  // white
    } else {
        // Logarithmic scaling for smoother gradient
        let log_scale = (iterations as f32 + 1.0).ln() / (max_iter as f32).ln(); // Normalize to [0,1]
        (log_scale * 255.0) as u8 // Scale to [0,255]
    }
}

fn main() {
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <output_file> <T>", args[0]);
        std::process::exit(1);
    }
    
    let output_file = &args[1];
    let t: u32 = match args[2].parse() {
        Ok(val) => val,
        Err(_) => {
            eprintln!("Error: T must be a positive integer");
            std::process::exit(1);
        }
    };
    
    // Calculate image dimensions: 2^(T+6) × 2^(T+6)
    let size = 2u32.pow(t + 9);  // 2048×2048
    let width = size as usize;
    let height = size as usize;
    let total_pixels = (width * height) as u64;
    
    // Image boundaries
    let x_min = -2.0;   // Left boundary
    let x_max = 0.5;    // Right boundary
    let y_min = -1.25;  // Bottom boundary
    let y_max = 1.25;   // Top boundary
    
    // Pre-calculate scaling factors
    let x_scale = (x_max - x_min) / (width as f64 - 1.0);
    let y_scale = (y_max - y_min) / (height as f64 - 1.0);
    
    // Allocate image buffer
    let mut image_data = vec![0u8; width * height];
    
    // Start timing
    let start_time = Instant::now();
    
    // Compute Mandelbrot set
    for row in 0..height {
        let y = y_max - (row as f64) * y_scale; // Map row to y coordinate
        
        for col in 0..width {
            let x = x_min + (col as f64) * x_scale; // Map column to x coordinate
            
            // Compute iterations for this point
            let iterations = mandelbrot_iterations(x, y, 255);
            
            // Store in image buffer 
            // image_data[row * width + col] = iterations;
            // made color_map because above produced plain black and white image
            image_data[row * width + col] = color_map(iterations,255);

        }
    }
    
    // Stop timing
    let elapsed = start_time.elapsed();
    let elapsed_ms = elapsed.as_secs_f64() * 1000.0;
    let mpx_per_sec = (total_pixels as f64) / (elapsed.as_secs_f64() * 1_000_000.0);
    
    // Compute CRC32 of image data
    let crc_table = make_crc_table();
    let crc = crc32(&crc_table, &image_data);
    
    // Write PGM file
    let file = File::create(output_file).expect("Failed to create output file");
    let mut writer = BufWriter::new(file);
    
    // Write PGM header
    writeln!(writer, "P5").expect("Failed to write PGM header");
    writeln!(writer, "{} {}", width, height).expect("Failed to write dimensions");
    writeln!(writer, "255").expect("Failed to write max value");
    
    // Write image data
    writer.write_all(&image_data).expect("Failed to write image data");
    
    // Print statistics
    println!("Checksum: 0x{:08X}, computed {} pixel in {:.2} ms = {:.2} Mpx/s", 
             crc, total_pixels, elapsed_ms, mpx_per_sec);
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mandelbrot_iterations() {
        // Point inside the set (should return 255)
        let inside = mandelbrot_iterations(0.0, 0.0, 255);
        assert_eq!(inside, 255);
        
        // Point outside the set (should return less than 255)
        let outside = mandelbrot_iterations(1.0, 1.0, 255);
        assert!(outside < 255);
    }
    
    #[test]
    fn test_crc32() {
        let table = make_crc_table();
        let test_data = b"123456789";
        let crc = crc32(&table, test_data);
        assert_eq!(crc, 0xCBF43926); // Known CRC-32 value for "123456789"
    }
}