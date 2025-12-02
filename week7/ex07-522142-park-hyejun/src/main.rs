use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 3 {
        eprintln!("Usage: {} <matrix_file.mtx> <b_value>", args[0]);
        std::process::exit(1);
    }
    
    let filename = &args[1];
    let b_value: f64 = args[2].parse()?;
    
    // Read matrix from MTX file
    let (_matrix_name, k) = read_mtx_matrix(filename)?;
    
    // Get matrix name without path and extension
    let base_name = std::path::Path::new(filename)
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    
    // Compute L = K * K^T
    let l = compute_l(&k);
    
    // Create vector b with all elements = b_value
    let n = l.rows;
    let b = vec![b_value; n];
    
    // Perform Cholesky decomposition L = M * M^T
    let m = cholesky_decomposition(&l)?;
    
    // Solve Lx = b using Cholesky decomposition
    let x = solve_cholesky(&m, &b)?;
    
    // Compute Lx
    let lx = compute_matrix_vector_product(&l, &x);
    
    // Compute errors
    let (err_max, err_2) = compute_errors(&lx, &b);
    
    // Print result in required format
    println!("{}: err_max={:.12}, err_2={:.12}", 
             base_name, err_max, err_2);
    
    Ok(())
}

#[derive(Debug, Clone)]
struct Matrix {
    rows: usize,
    cols: usize,
    data: Vec<f64>,
}

impl Matrix {                                       // Implementation of Matrix methods
    fn new(rows: usize, cols: usize) -> Self {
        Matrix {
            rows,
            cols,
            data: vec![0.0; rows * cols],
        }
    }
    
    fn get(&self, row: usize, col: usize) -> f64 {
        self.data[row * self.cols + col]
    }
    
    fn set(&mut self, row: usize, col: usize, value: f64) {
        self.data[row * self.cols + col] = value;
    }
    
    fn add(&mut self, row: usize, col: usize, value: f64) {
        self.data[row * self.cols + col] += value;
    }
}

fn read_mtx_matrix(filename: &str) -> Result<(String, Matrix), Box<dyn Error>> {    // Read MTX matrix from file
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    
    // Skip comment lines (starting with %)
    let mut line = String::new();
    while let Some(Ok(l)) = lines.next() {
        line = l;
        if !line.starts_with('%') {
            break;
        }
    }
    
    // Parse matrix dimensions
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 2 {
        return Err("Invalid MTX format: insufficient dimensions".into());
    }
    
    let rows: usize = parts[0].parse()?;
    let cols: usize = parts[1].parse()?;
    let nnz = if parts.len() > 2 {
        parts[2].parse::<usize>()?
    } else {
        0
    };
    
    let mut matrix = Matrix::new(rows, cols);   // Initialize matrix
    
    // Read matrix entries
    for _ in 0..nnz {                           // Read non-zero entries
        if let Some(Ok(l)) = lines.next() {     // Read each line
            let parts: Vec<&str> = l.split_whitespace().collect();  // Split line into parts
            if parts.len() >= 2 {                                   // Ensure at least row and column are present and parse
                let row: usize = parts[0].parse()?;                 
                let col: usize = parts[1].parse()?;
                
                // Handle complex numbers (real part only)
                let val = if parts.len() >= 3 {
                    parts[2].parse().unwrap_or(1.0)
                } else {
                    1.0
                };
                
                // MTX format uses 1-based indexing
                if row > 0 && col > 0 && row <= rows && col <= cols {
                    matrix.add(row - 1, col - 1, val);
                    
                    // For symmetric matrices, set symmetric entry
                    if row != col && parts.len() >= 2 {
                        matrix.add(col - 1, row - 1, val);
                    }
                }
            }
        }
    }
    
    Ok((filename.to_string(), matrix))
}

fn compute_l(k: &Matrix) -> Matrix {        // Compute L = K * K^T
    let n = k.rows;
    let mut l = Matrix::new(n, n);
    
    // L = K * K^T (since K is real, conjugate transpose = transpose)
    for i in 0..n {
        for j in 0..=i {  // Only compute lower triangle
            let mut sum = 0.0;
            for k_idx in 0..k.cols {
                sum += k.get(i, k_idx) * k.get(j, k_idx);
            }
            l.set(i, j, sum);
            if i != j {
                l.set(j, i, sum);  // Symmetric
            }
        }
    }
    
    l
}

fn cholesky_decomposition(l: &Matrix) -> Result<Matrix, Box<dyn Error>> {       // Cholesky decomposition L = M * M^T
    let n = l.rows;
    let mut m = Matrix::new(n, n);
    
    for i in 0..n {             // Compute M
        for j in 0..=i {        // Only compute lower triangle
            let mut sum = 0.0;  // Initialize sum
            
            if j == i {
                // Diagonal element
                for k in 0..j {             // Sum over previous elements
                    let mik = m.get(j, k);  // Get M(j, k)
                    sum += mik * mik;       // Square and add to sum
                }
                let diag = l.get(i, i) - sum; // Compute diagonal value
                if diag <= 0.0 {              // Check for positive definiteness
                    // Add small regularization for numerical stability and set M(i.j)
                    const EPSILON: f64 = 1e-12;
                    m.set(i, j, (diag + EPSILON).sqrt());
                } else {
                    m.set(i, j, diag.sqrt());
                }
            } else {
                // Off-diagonal element
                for k in 0..j {
                    sum += m.get(i, k) * m.get(j, k);   // Sum over previous elements
                }
                let mjj = m.get(j, j);  // Get M(j, j)
                if mjj.abs() < 1e-12 {  // Check for singularity
                    return Err("Matrix is singular or ill-conditioned".into());
                }
                m.set(i, j, (l.get(i, j) - sum) / mjj); // Set M(i, j)
            }
        }
    }
    
    Ok(m)
}

fn solve_cholesky(m: &Matrix, b: &[f64]) -> Result<Vec<f64>, Box<dyn Error>> {  // Solve Lx = b using Cholesky decomposition
    let n = m.rows;
    
    // Forward substitution: solve M * y = b
    let mut y = vec![0.0; n];
    for i in 0..n {
        let mut sum = 0.0;
        // Use iterator to avoid clippy warning
        for (j, y_j) in y.iter().enumerate().take(i) {  // Iterate up to i
            sum += m.get(i, j) * y_j;   // Accumulate sum
        }
        let mii = m.get(i, i);  // Diagonal element
        if mii.abs() < 1e-12 {  // Check for singularity
            return Err("Singular matrix in forward substitution".into());
        }
        y[i] = (b[i] - sum) / mii; // Compute y[i]
    }
    
    // Backward substitution: solve M^T * x = y
    let mut x = vec![0.0; n];
    for i in (0..n).rev() {
        let mut sum = 0.0;
        // Use iterator to avoid clippy warning
        for (j, x_j) in x.iter().enumerate().skip(i + 1) { // Iterate from i+1 to n
            sum += m.get(j, i) * x_j;   // Accumulate sum
        }
        let mii = m.get(i, i);  // Diagonal element
        if mii.abs() < 1e-12 {  // Check for singularity
            return Err("Singular matrix in backward substitution".into());
        }
        x[i] = (y[i] - sum) / mii;
    }
    
    Ok(x)
}

fn compute_matrix_vector_product(a: &Matrix, x: &[f64]) -> Vec<f64> { // Compute Lx
    let n = a.rows; // Get number of rows
    let mut result = vec![0.0; n]; // Initialize result vector
    
    // Matrix-vector multiplication
    for (i, result_i) in result.iter_mut().enumerate() {
        let mut sum = 0.0;
        for (j, x_j) in x.iter().enumerate() {  // Iterate over x
            sum += a.get(i, j) * x_j;   // Accumulate sum
        }
        *result_i = sum; // Set result[i]
    }
    
    result
}

fn compute_errors(lx: &[f64], b: &[f64]) -> (f64, f64) { // Compute err_max and err_2
    let n = lx.len();
    let mut err_max: f64 = 0.0;
    let mut err_2_squared: f64 = 0.0;
    
    for i in 0..n { // Iterate over elements
        let diff = (lx[i] - b[i]).abs(); // Compute absolute difference
        if diff > err_max { // Update max error
            err_max = diff; // Set new max error
        }
        err_2_squared += diff * diff; // Accumulate squared error
    }
    
    let err_2 = err_2_squared.sqrt(); // Compute L2 norm
    (err_max, err_2)                  // Return errors
}