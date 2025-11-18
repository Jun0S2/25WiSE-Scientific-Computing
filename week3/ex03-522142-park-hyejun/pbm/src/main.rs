/**
* Results are stored in output directory as CXX.pbm where C is character and XX is 01 to 30.
*/
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Write};

/* Define the grid layout of characteres on the page */
const ROWS: usize = 16;                     // 16 rows of characteres (1~F)
const COLS: usize = 30;                     // 30 characteres per row
const CHAR_SIZE: usize = 128;               // Output size for character is 128x128 pixels

fn main() -> io::Result<()> {
    let filename = "exercise_scan.pbm";
    let file = File::open(filename)?;
    let mut reader = BufReader::new(file);


    let mut line = String::new();            // read header
    reader.read_line(&mut line)?;            // reads first line
    if !line.trim().starts_with("P1") {      // check PBM type (P1 Format starts with P1, and i used P1)
        panic!("Not a P1 PBM file");
    }


    let (width, height) = loop {               // Reads image dimensions
        line.clear();                          // clear previous line
        reader.read_line(&mut line)?;          // read next line
        let line_trim = line.trim();           // trim whitespace
        if line_trim.is_empty() || line_trim.starts_with('#') { // Skip comments and empty lines
            continue;
        }
        let parts: Vec<usize> = line_trim                    // parse dimensions
            .split_whitespace()                              // split by whitespace
            .map(|s| s.parse().expect("Invalid dimension"))  // parse to usize
            .collect();                                      // collect into vector
        if parts.len() != 2 {
            panic!("Invalid PBM dimensions");
        }
        break (parts[0], parts[1]);                         // return width and height
    };

    /* Read all the pixel data from the file  */
    let mut all_numbers = Vec::with_capacity(width * height);   // store all pixel values
    for l in reader.lines() {                                    
        let l = l?;                                             
        for s in l.split_whitespace() {
            match s {                                            // parse pixel values
                "0" => all_numbers.push(0u8),                    // white pixel
                "1" => all_numbers.push(1u8),                    // black pixel
                _ => {}                                          // ignore invalid values
            }
        }
    }
    if all_numbers.len() != width * height {                    // validate pixel count
        panic!("Pixel count does not match dimensions");
    }


    let mut pixels = vec![vec![0u8; width]; height];            // 2D pixel array
    for y in 0..height {                                        // populate 2D array
        pixels[y] = all_numbers[y*width..(y+1)*width].to_vec(); // slice from 1D to 2D
    }


    fs::create_dir_all("output")?;                              // create output directory if not exists

    let characters = "1234567890ABCDEF".chars().collect::<Vec<_>>();    // characters to process
    let char_width = width / COLS;                                      // character width in pixels
    let char_height = height / ROWS;                                    // character height in pixels

    let mut gm_counts = vec![Vec::new(); 16];                           // store black pixel counts for geometric mean calculation

    for (row_idx, ch) in characters.iter().enumerate() {                         // process each character
        for col_idx in 0..COLS {                                                 // process each instance of the character
            let mut char_pixels = vec![vec![0u8; CHAR_SIZE]; CHAR_SIZE];         // resized character pixel array
            for y in 0..CHAR_SIZE {                                              // resize to CHAR_SIZE x CHAR_SIZE
                for x in 0..CHAR_SIZE {                                          // using nearest neighbor scaling
                    let src_y = row_idx*char_height + y*char_height/CHAR_SIZE;   // source y coordinate
                    let src_x = col_idx*char_width + x*char_width/CHAR_SIZE;     // source x coordinate
                    char_pixels[y][x] = pixels[src_y][src_x];                    // copy pixel value
                }
            }

            let fname = format!("output/{}{:02}.pbm", ch, col_idx+1);           // output filename (CXX.pbm)
            let mut f = File::create(&fname)?;                                  // create output file   
            writeln!(f, "P1")?;                                                 // write PBM header        
            writeln!(f, "{} {}", CHAR_SIZE, CHAR_SIZE)?;                        // write dimensions
            for row in &char_pixels {                                           // write pixel data     
                for p in row {                                                  // each pixel in row
                    write!(f, "{} ", p)?;                                       // write pixel value              
                }
                writeln!(f)?;                                                   // new line after each row          
            }

            let black_count = char_pixels.iter().flatten().filter(|&&v| v == 1).count(); // count black pixels
            gm_counts[row_idx].push(black_count as f64);                        // store count for geometric mean calculation
        }
    }
    println!("Geometric means per character:");                                 // output geometric means
    for (i, ch) in characters.iter().enumerate() {                              // for each character
        let product: f64 = gm_counts[i].iter().product();                       // calculate product of counts
        let gm = product.powf(1.0 / gm_counts[i].len() as f64);                 // calculate geometric mean
        println!("{}: {:.2}", ch, gm);                                          // print character and its geometric mean
    }

    Ok(())                                                                      // successful completion
}
