use std::fs::File;
use std::io::{Write, BufWriter};
use std::time::Instant;

// CRC-32 implementation
fn crc32(table: &[u32; 256], bytes: &[u8]) -> u32 {
    let mut crc = !0u32;        // Initial value
    for byte in bytes {         // Process each byte
        crc = table[((crc as u8) ^ byte) as usize] ^ (crc >> 8);    // Update CRC value
    }
    !crc
}

// Generate CRC-32 lookup table
fn make_crc_table() -> [u32; 256] {
    let mut table = [0u32; 256];                        // Initialize table with zeros and size 256
    for (i, item) in table.iter_mut().enumerate() {     // Fill table
        let mut c = i as u32;
        for _ in 0..8 {                                 
            if c & 1 != 0 {                             // Is LSB set?(1)
                c = 0xedb88320 ^ (c >> 1);              // 1: Apply polynomial and shift right
            } else {                                    
                c >>= 1;                                // 0: shift right
            }
        }
        *item = c;
    }
    table
}

// Mandelbrot set computation
fn mandelbrot_iterations(cx: f64, cy: f64, max_iter: usize) -> u8 {
    let mut x = 0.0;             // Real part of z
    let mut y = 0.0;             // Imaginary part of z
    let mut x2 = 0.0;            // x squared
    let mut y2 = 0.0;            // y squared
    
    for i in 0..max_iter { 
        if x2 + y2 > 4.0 {      // Escape condition : |z|^2 > 4
            return i as u8;
        }
        
        y = 2.0 * x * y + cy;
        x = x2 - y2 + cx;
        x2 = x * x;
        y2 = y * y;
    }
    
    max_iter as u8
}

// added color_map function else, the image is plain black and white
fn color_map(iterations: u8, max_iter: u8) -> u8 {
    if iterations == max_iter {
        255
    } else {   // Logarithmic scaling for smoother gradient
        let log_scale = (iterations as f32 + 1.0).ln() / (max_iter as f32).ln();
        (log_scale * 255.0) as u8
    }
}

// Sequential implementation
fn sequential_mandelbrot(
    width: usize,
    height: usize,
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
) -> (Vec<u8>, f64) {
    let start_time = Instant::now();
    // Scale factors
    let x_scale = (x_max - x_min) / (width as f64 - 1.0); 
    let y_scale = (y_max - y_min) / (height as f64 - 1.0); 
    
    let mut image_data = vec![0u8; width * height]; // Image buffer
    
    for row in 0..height {
        let y = y_max - (row as f64) * y_scale; // Map row to y coordinate
        
        for col in 0..width {
            let x = x_min + (col as f64) * x_scale; // Map column to x coordinate
            let iterations = mandelbrot_iterations(x, y, 255);  // Compute iterations
            image_data[row * width + col] = color_map(iterations, 255); // Map to color
        }
    }
    
    let elapsed = start_time.elapsed();
    let elapsed_ms = elapsed.as_secs_f64() * 1000.0;
    
    (image_data, elapsed_ms)
}

// Thread-based implementation
fn thread_mandelbrot(
    width: usize,
    height: usize,
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
) -> (Vec<u8>, f64) {
    use rayon::prelude::*;  // Import Rayon for parallelism
    
    let start_time = Instant::now();
    
    let x_scale = (x_max - x_min) / (width as f64 - 1.0);
    let y_scale = (y_max - y_min) / (height as f64 - 1.0);
    
    let image_data: Vec<u8> = (0..height)
        .into_par_iter()    // Parallel iterator over rows
        .flat_map(|row| {   // Compute each row
            let y = y_max - (row as f64) * y_scale; // Map row to y coordinate
            (0..width)
                .map(|col| {    // Compute each column
                    let x = x_min + (col as f64) * x_scale; 
                    let iterations = mandelbrot_iterations(x, y, 255);
                    color_map(iterations, 255)
                })
                .collect::<Vec<u8>>()   // Collect row data
        })
        .collect(); // Collect all rows into a single vector
    
    let elapsed = start_time.elapsed();
    let elapsed_ms = elapsed.as_secs_f64() * 1000.0;
    
    (image_data, elapsed_ms)
}

// MPI parameters
struct MpiParams {
    rank: i32,
    size: i32,
    world: mpi::topology::SimpleCommunicator,
}

// MPI-based implementation
fn mpi_mandelbrot(
    width: usize,
    height: usize,
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
    params: &MpiParams,
) -> (Vec<u8>, f64) {
    use mpi::traits::*; // Import MPI traits
    
    let start_time = Instant::now();
    let rank = params.rank;
    let size = params.size;
    let world = &params.world;  // Get communicator
    
    let rows_per_process = height as i32 / size;    // Rows per process
    let extra_rows = height as i32 % size;          // Extra rows to distribute
    
    let mut start_row = 0;
    for i in 0..rank {
        start_row += rows_per_process + if i < extra_rows { 1 } else { 0 }; // Calculate start row
    }
    
    let my_rows = rows_per_process + if rank < extra_rows { 1 } else { 0 }; // Rows for this process
    
    let x_scale = (x_max - x_min) / (width as f64 - 1.0);
    let y_scale = (y_max - y_min) / (height as f64 - 1.0);
    
    let mut local_data = vec![0u8; (my_rows as usize) * width]; // Local buffer
    
    for local_row in 0..my_rows as usize {
        let global_row = start_row as usize + local_row; // Global row index
        let y = y_max - (global_row as f64) * y_scale;  // Map to y coordinate
        
        for col in 0..width {
            let x = x_min + (col as f64) * x_scale; 
            let iterations = mandelbrot_iterations(x, y, 255); 
            local_data[local_row * width + col] = color_map(iterations, 255); 
        }
    }
    
    let elapsed = start_time.elapsed();
    let elapsed_ms = elapsed.as_secs_f64() * 1000.0;
    
    if rank == 0 {
        let mut image_data = vec![0u8; width * height];
        
        // Copy rank 0's data
        let rows_in_rank0 = rows_per_process as usize + if extra_rows > 0 { 1 } else { 0 };
        image_data[0..rows_in_rank0 * width].copy_from_slice(&local_data[0..rows_in_rank0 * width]);
        
        // Receive data from other processes
        for source_rank in 1..size {
            let source_rows = rows_per_process + if source_rank < extra_rows { 1 } else { 0 };
            let mut recv_buffer = vec![0u8; (source_rows as usize) * width];
            
            world.process_at_rank(source_rank)      // Get process
                .receive_into(&mut recv_buffer[..]);// Receive data
            
            let mut source_start_row = 0;
            for i in 0..source_rank {
                source_start_row += rows_per_process + if i < extra_rows { 1 } else { 0 };
            }
            
            let start_idx = source_start_row as usize * width;
            let end_idx = start_idx + (source_rows as usize) * width;
            image_data[start_idx..end_idx].copy_from_slice(&recv_buffer); // Copy received data
        }
        
        (image_data, elapsed_ms) 
    } else {
        // Send data to rank 0
        world.process_at_rank(0).send(&local_data[..]);
        
        (Vec::new(), elapsed_ms)
    }
}

fn write_pgm(output_file: &str, width: usize, height: usize, image_data: &[u8]) -> std::io::Result<()> {
    let file = File::create(output_file)?;
    let mut writer = BufWriter::new(file);
    
    writeln!(writer, "P5")?;
    writeln!(writer, "{} {}", width, height)?;
    writeln!(writer, "255")?;
    
    writer.write_all(image_data)?;
    Ok(())
}

fn main() {
    // MPI initialization
    use mpi::traits::*;
    
    let universe = mpi::initialize().unwrap();
    let world = universe.world();
    let rank = world.rank();
    let size = world.size();
    
    let mpi_params = MpiParams { rank, size, world };
    
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        if rank == 0 {
            eprintln!("Usage: {} <output_file> <T>", args[0]);
        }
        std::process::exit(1);
    }
    
    let t_value = if args.len() >= 3 {
        args[2].clone()
    } else {
        "2".to_string()
    };
    
    let output_file = &args[1];
    let t: u32 = match t_value.parse() {
        Ok(val) => val,
        Err(_) => {
            if rank == 0 {
                eprintln!("Error: T must be a positive integer");
            }
            std::process::exit(1);
        }
    };
    
    // Calculate image dimensions: 2^(T+9) × 2^(T+9)
    let size_val = 2u32.pow(t + 9);
    let width = size_val as usize;
    let height = size_val as usize;
    let total_pixels = (width * height) as u64;
    
    // Image boundaries
    let x_min = -2.0;
    let x_max = 0.5;
    let y_min = -1.25;
    let y_max = 1.25;
    
    let crc_table = make_crc_table();
    
    // Algorithm 1: Sequential
    let (seq_image_data, seq_elapsed_ms) = sequential_mandelbrot(
        width, height, x_min, x_max, y_min, y_max
    );
    let seq_crc = crc32(&crc_table, &seq_image_data);
    let seq_mpx_per_sec = total_pixels as f64 / (seq_elapsed_ms / 1000.0) / 1_000_000.0;
    
    if rank == 0 {
        println!("Algorithm 1 Checksum: 0x{:08X}, computed {} pixel in {:.2} ms = {:.2} Mpx/s", 
               seq_crc, total_pixels, seq_elapsed_ms, seq_mpx_per_sec);
    }
    
    // Algorithm 2: Thread-based
    let (thread_image_data, thread_elapsed_ms) = thread_mandelbrot(
        width, height, x_min, x_max, y_min, y_max
    );
    let thread_crc = crc32(&crc_table, &thread_image_data);
    let thread_mpx_per_sec = total_pixels as f64 / (thread_elapsed_ms / 1000.0) / 1_000_000.0;
    
    if rank == 0 {
        println!("Algorithm 2 Checksum: 0x{:08X}, computed {} pixel in {:.2} ms = {:.2} Mpx/s", 
               thread_crc, total_pixels, thread_elapsed_ms, thread_mpx_per_sec);
    }
    
    // Algorithm 3: MPI-based
    let (mpi_image_data, mpi_elapsed_ms) = mpi_mandelbrot(
        width, height, x_min, x_max, y_min, y_max,
        &mpi_params
    );
    
    if rank == 0 {
        let mpi_crc = crc32(&crc_table, &mpi_image_data);
        let mpi_mpx_per_sec = total_pixels as f64 / (mpi_elapsed_ms / 1000.0) / 1_000_000.0;
        println!("Algorithm 3 Checksum: 0x{:08X}, computed {} pixel in {:.2} ms = {:.2} Mpx/s", 
               mpi_crc, total_pixels, mpi_elapsed_ms, mpi_mpx_per_sec);
        
        if let Err(e) = write_pgm(output_file, width, height, &mpi_image_data) {
            eprintln!("Failed to write PGM file: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mandelbrot_iterations() {
        let inside = mandelbrot_iterations(0.0, 0.0, 255);
        assert_eq!(inside, 255);
        
        let outside = mandelbrot_iterations(1.0, 1.0, 255);
        assert!(outside < 255);
    }
    
    #[test]
    fn test_crc32() {
        let table = make_crc_table();
        let test_data = b"123456789";
        let crc = crc32(&table, test_data);
        assert_eq!(crc, 0xCBF43926);
    }
}