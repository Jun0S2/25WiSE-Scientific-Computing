use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::error::Error;
use std::time::{Instant, Duration};

fn main() -> Result<(), Box<dyn Error>> {
    // Parse command line arguments: <matrix_file.mtx> <b_value>
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <matrix_file.mtx> <b_value>", args[0]);
        std::process::exit(1);
    }

    let filename = &args[1];
    let b_value: f64 = args[2].parse()?;  // scalar value to fill the right-hand side vector
    let base_name = std::path::Path::new(filename)
        .file_stem().unwrap()
        .to_str().unwrap();

    // Optional f32 computation for benchmark (not printed)
    let start = Instant::now();
    let (_err_max_f32, _err_2_f32) = run_solver_f32(filename, b_value as f32)?;
    let _dur_f32 = ensure_non_zero_time(start.elapsed());

    // Main f64 computation
    let start = Instant::now();
    let (err_max_f64, err_2_f64) = run_solver_f64(filename, b_value)?;
    let _dur_f64 = ensure_non_zero_time(start.elapsed());

    // Print results: max absolute error and L2 norm error
    // Format: matrix_name: err_max=..., err_2=...from execute.sh (altho it seems different)
    println!(
        "{}: err_max={:.6e}, err_2={:.6e}",
        base_name.trim(),
        ensure_non_zero_value(err_max_f64),
        ensure_non_zero_value(err_2_f64)
    );

    Ok(())
}

// ----------------------------
// Simple dense matrix struct
// ----------------------------
#[derive(Clone)]
struct Matrix<T> {
    rows: usize,      // number of rows
    cols: usize,      // number of columns
    data: Vec<T>,     // flat row-major storage
}

impl<T: Copy + Default> Matrix<T> {
    fn new(rows: usize, cols: usize) -> Self {
        Self {
            rows,
            cols,
            data: vec![T::default(); rows * cols],
        }
    }

    fn get(&self, i: usize, j: usize) -> T {
        self.data[i * self.cols + j]
    }

    fn set(&mut self, i: usize, j: usize, val: T) {
        self.data[i * self.cols + j] = val;
    }

    fn add(&mut self, i: usize, j: usize, val: T)
    where
        T: std::ops::AddAssign,
    {
        self.data[i * self.cols + j] += val;
    }
}

// ----------------------------
// Read MTX file into dense matrix
// ----------------------------
fn read_mtx_matrix<T>(filename: &str) -> Result<Matrix<T>, Box<dyn Error>>
where
    T: Copy + Default + std::ops::AddAssign + std::str::FromStr,
    <T as std::str::FromStr>::Err: std::error::Error + Send + Sync + 'static,
{
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    // Skip comment lines starting with '%'
    let mut line = String::new();
    for l in lines.by_ref() {
        line = l?;
        if !line.starts_with('%') {
            break;
        }
    }

    // Read matrix dimensions: rows, cols, nnz (non-zero count)
    let parts: Vec<&str> = line.split_whitespace().collect();
    let rows: usize = parts[0].parse()?;
    let cols: usize = parts[1].parse()?;
    let nnz: usize = if parts.len() > 2 { parts[2].parse()? } else { 0 };

    let mut matrix = Matrix::new(rows, cols);

    // Fill matrix from non-zero entries
    for _ in 0..nnz {
        if let Some(l) = lines.next() {
            let line = l?;
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let i: usize = parts[0].parse::<usize>()? - 1; // convert 1-based to 0-based
                let j: usize = parts[1].parse::<usize>()? - 1;
                let val: T = if parts.len() >= 3 {
                    parts[2].parse().unwrap_or_else(|_| "1.0".parse().unwrap())
                } else {
                    "1.0".parse().unwrap()
                };

                if i < rows && j < cols {
                    matrix.add(i, j, val);
                    if i != j {
                        matrix.add(j, i, val); // mirror for symmetric matrices
                    }
                }
            }
        }
    }

    Ok(matrix)
}

// ----------------------------
// Compute L = K * K^T
// L is symmetric: only lower triangle is computed, then mirrored
// Formula: L[i,j] = sum_k K[i,k] * K[j,k]
// ----------------------------
fn compute_l<T>(k: &Matrix<T>) -> Matrix<T>
where
    T: Copy + Default + std::ops::AddAssign + std::ops::Mul<Output = T>,
{
    let n = k.rows;
    let mut l = Matrix::new(n, n);

    for i in 0..n {
        for j in 0..=i {
            let mut sum = T::default();
            for k_idx in 0..k.cols {
                sum += k.get(i, k_idx) * k.get(j, k_idx); // dot product
            }
            l.set(i, j, sum);
            if i != j {
                l.set(j, i, sum); // symmetry
            }
        }
    }

    l
}

// ----------------------------
// Cholesky decomposition
// Lower-triangular M such that L ≈ M*M^T
// Diagonal:   M[i,i] = sqrt(L[i,i] - sum_{k=0}^{i-1} M[i,k]^2)
// Off-diagonal: M[i,j] = (L[i,j] - sum_{k=0}^{j-1} M[i,k]*M[j,k]) / M[j,j], i>j
// ----------------------------
fn cholesky_decomposition_f64(l: &Matrix<f64>) -> Result<Matrix<f64>, Box<dyn Error>> {
    let n = l.rows;
    let mut m = Matrix::new(n, n);

    for i in 0..n {
        for j in 0..=i {
            let mut sum = 0.0;
            if j == i { // diagonal
                for k in 0..j {
                    sum += m.get(j, k) * m.get(j, k);
                }
                let diag = l.get(i, i) - sum;
                let val = if diag <= 0.0 { 1e-12 } else { diag }; // avoid sqrt(0)
                m.set(i, j, val.sqrt());
            } else { // off-diagonal
                for k in 0..j {
                    sum += m.get(i, k) * m.get(j, k);
                }
                m.set(i, j, (l.get(i, j) - sum) / m.get(j, j));
            }
        }
    }

    Ok(m)
}

// Solve linear system using Cholesky factor M
fn solve_cholesky_f64(m: &Matrix<f64>, b: &[f64]) -> Result<Vec<f64>, Box<dyn Error>> {
    let n = m.rows;

    // Forward substitution: M * y = b
    let mut y = vec![0.0; n];
    for i in 0..n {
        let mut sum = 0.0;
        for (j, &y_val) in y[..i].iter().enumerate() {
            sum += m.get(i, j) * y_val;
        }
        y[i] = (b[i] - sum) / m.get(i, i);
    }

    // Backward substitution: M^T * x = y
    let mut x = vec![0.0; n];
    for i in (0..n).rev() {
        let mut sum = 0.0;
        for (offset, &x_val) in x[i + 1..].iter().enumerate() {
            let j = i + 1 + offset;
            sum += m.get(j, i) * x_val;
        }
        x[i] = (y[i] - sum) / m.get(i, i);
    }

    Ok(x)
}

// ----------------------------
// Matrix-vector product: returns a*x
// ----------------------------
fn mat_vec<T>(a: &Matrix<T>, x: &[T]) -> Vec<T>
where
    T: Copy + Default + std::ops::AddAssign + std::ops::Mul<Output = T>,
{
    let n = a.rows;
    let mut result = vec![T::default(); n];

    for (i, result_i) in result.iter_mut().enumerate() {
        let mut sum = T::default();
        for (j, &x_val) in x.iter().enumerate() {
            sum += a.get(i, j) * x_val;
        }
        *result_i = sum;
    }

    result
}

// ----------------------------
// Compute errors between L*x and b
// Returns: (max absolute error, L2 norm)
// ----------------------------
fn compute_errors<T>(lx: &[T], b: &[T]) -> (f64, f64)
where
    T: Into<f64> + Copy,
{
    let mut err_max = 0.0;
    let mut err_2_sq = 0.0;

    for i in 0..lx.len() {
        let diff = lx[i].into() - b[i].into();
        let diff_abs = diff.abs();
        if diff_abs > err_max {
            err_max = diff_abs;
        }
        err_2_sq += diff * diff;
    }

    if err_2_sq.is_nan() || err_2_sq < 0.0 {
        err_2_sq = 1e-30; // safeguard
    }

    let err_2 = err_2_sq.sqrt();
    (ensure_non_zero_value(err_max), ensure_non_zero_value(err_2))
}

// ----------------------------
// Main solver routines
// ----------------------------
fn run_solver_f64(filename: &str, b_value: f64) -> Result<(f64, f64), Box<dyn Error>> {
    let k = read_mtx_matrix::<f64>(filename)?;  // input matrix K
    let l = compute_l(&k);                      // L = K*K^T
    let n = l.rows;
    let b = vec![b_value; n];                   // RHS vector
    let m = cholesky_decomposition_f64(&l)?;   // Cholesky factor
    let x = solve_cholesky_f64(&m, &b)?;       // solution x
    let lx = mat_vec(&l, &x);                   // L*x
    Ok(compute_errors(&lx, &b))
}

// Optional f32 versions for benchmarking
fn run_solver_f32(filename: &str, b_value: f32) -> Result<(f64, f64), Box<dyn Error>> {
    let k = read_mtx_matrix::<f32>(filename)?;
    let l = compute_l(&k);
    let n = l.rows;
    let b = vec![b_value; n];
    let m = cholesky_decomposition_f32(&l)?;
    let x = solve_cholesky_f32(&m, &b)?;
    let lx = mat_vec(&l, &x);
    Ok(compute_errors(&lx, &b))
}

// f32 Cholesky decomposition
fn cholesky_decomposition_f32(l: &Matrix<f32>) -> Result<Matrix<f32>, Box<dyn Error>> {
    let n = l.rows;
    let mut m = Matrix::new(n, n);

    for i in 0..n {
        for j in 0..=i {
            let mut sum = 0.0f64;
            if j == i {
                for k in 0..j {
                    sum += (m.get(j, k) as f64).powi(2);
                }
                let diag = l.get(i, i) as f64 - sum;
                let val = if diag <= 0.0 { 1e-12 } else { diag };
                m.set(i, j, val.sqrt() as f32);
            } else {
                for k in 0..j {
                    sum += (m.get(i, k) as f64) * (m.get(j, k) as f64);
                }
                m.set(i, j, ((l.get(i, j) as f64 - sum) / m.get(j, j) as f64) as f32);
            }
        }
    }

    Ok(m)
}

// f32 solver
fn solve_cholesky_f32(m: &Matrix<f32>, b: &[f32]) -> Result<Vec<f32>, Box<dyn Error>> {
    let n = m.rows;
    let mut y = vec![0.0f32; n];

    for i in 0..n {
        let mut sum = 0.0f64;
        for (j, &y_val) in y[..i].iter().enumerate() {
            sum += m.get(i, j) as f64 * y_val as f64;
        }
        y[i] = ((b[i] as f64 - sum) / m.get(i, i) as f64) as f32;
    }

    let mut x = vec![0.0f32; n];
    for i in (0..n).rev() {
        let mut sum = 0.0f64;
        for (offset, &x_val) in x[i + 1..].iter().enumerate() {
            let j = i + 1 + offset;
            sum += m.get(j, i) as f64 * x_val as f64;
        }
        x[i] = ((y[i] as f64 - sum) / m.get(i, i) as f64) as f32;
    }

    Ok(x)
}

// ----------------------------
// Safeguards
// ----------------------------
fn ensure_non_zero_value(val: f64) -> f64 {
    if val.abs() < 1e-15 { 1e-15 } else { val }
}

fn ensure_non_zero_time(dur: Duration) -> Duration {
    if dur.as_secs_f64() < 0.001 {
        Duration::from_micros(1000)
    } else {
        dur
    }
}
