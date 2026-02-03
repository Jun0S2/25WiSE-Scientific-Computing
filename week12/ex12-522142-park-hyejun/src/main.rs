// src/main.rs

use std::fs;
use std::io;

// Standard Sudoku constants
const SIZE: usize = 9;        // Grid is 9x9
const BOX_SIZE: usize = 3;    // Each sub-box is 3x3

// Represents a single Sudoku puzzle
#[derive(Debug, Clone)]
struct SudokuPuzzle {
    // 9x9 grid
    // 0 means "empty cell"
    // 1..=9 are filled digits
    grid: [[u8; SIZE]; SIZE],
}

impl SudokuPuzzle {
    // Parse a Sudoku puzzle from a single-line string
    //
    // Expected format:
    // - Length: exactly 81 characters
    // - Digits '1'..'9' are fixed values
    // - '0' or '.' represent empty cells
    fn from_string(line: &str) -> Result<Self, String> {
        let chars: Vec<char> = line.trim().chars().collect();
        if chars.len() != SIZE * SIZE {
            return Err(format!("Invalid Sudoku string length: {}", chars.len()));
        }

        let mut grid = [[0; SIZE]; SIZE];

        // Map linear string index to (row, col)
        for (i, ch) in chars.iter().enumerate() {
            let row = i / SIZE;
            let col = i % SIZE;

            grid[row][col] = match ch.to_digit(10) {
                Some(d) if d >= 1 && d <= 9 => d as u8,
                _ => 0, // '.' or '0' => empty cell
            };
        }

        Ok(SudokuPuzzle { grid })
    }

    // Public entry point for solving the puzzle
    // Starts recursive backtracking from the top-left cell (0,0)
    fn solve(&mut self) -> bool {
        self.backtrack(0, 0)
    }

    // Core backtracking algorithm (Depth-First Search)
    // Returns true if a solution is found
    fn backtrack(&mut self, row: usize, col: usize) -> bool {
        // Base case:
        // If row == SIZE, we successfully filled the entire grid
        if row == SIZE {
            return true;
        }

        // Compute coordinates of the next cell
        let next_row = if col == SIZE - 1 { row + 1 } else { row };
        let next_col = if col == SIZE - 1 { 0 } else { col + 1 };

        // If the current cell is already filled, skip it
        if self.grid[row][col] != 0 {
            return self.backtrack(next_row, next_col);
        }

        // Try all possible digits (1..9)
        for digit in 1..=9 {
            // Check Sudoku constraints
            if self.is_valid(row, col, digit) {
                // Tentatively place digit
                self.grid[row][col] = digit;

                // Recurse to the next cell
                if self.backtrack(next_row, next_col) {
                    return true; // Solution found
                }

                // Undo choice (backtrack)
                self.grid[row][col] = 0;
            }
        }

        // No valid digit worked here → dead end
        false
    }

    // Check whether placing `digit` at (row, col) is valid
    //
    // Constraints:
    // - No duplicate in row
    // - No duplicate in column
    // - No duplicate in 3x3 sub-box
    fn is_valid(&self, row: usize, col: usize, digit: u8) -> bool {
        // Check row
        for c in 0..SIZE {
            if self.grid[row][c] == digit {
                return false;
            }
        }

        // Check column
        for r in 0..SIZE {
            if self.grid[r][col] == digit {
                return false;
            }
        }

        // Check 3x3 sub-box
        let box_row = (row / BOX_SIZE) * BOX_SIZE;
        let box_col = (col / BOX_SIZE) * BOX_SIZE;

        for r in box_row..box_row + BOX_SIZE {
            for c in box_col..box_col + BOX_SIZE {
                if self.grid[r][c] == digit {
                    return false;
                }
            }
        }

        true
    }

    // Convert the solved grid back into a single-line string
    //
    // - Digits 1..9 stay as digits
    // - Empty cells (0) become '.'
    fn to_string(&self) -> String {
        let mut result = String::with_capacity(SIZE * SIZE);

        for row in 0..SIZE {
            for col in 0..SIZE {
                let digit = self.grid[row][col];
                if digit == 0 {
                    result.push('.');
                } else {
                    result.push_str(&digit.to_string());
                }
            }
        }

        result
    }
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    // Expect exactly one argument: input file
    if args.len() != 2 {
        eprintln!("Usage: {} <puzzle_file.sdk>", args[0]);
        eprintln!("Example: cargo run --release test.sdk");
        eprintln!("Solution will be written to test_solved.sdk");
        std::process::exit(1);
    }

    let input_filename = &args[1];

    // Generate output filename
    let output_filename = if input_filename == "test.sdk" {
        "test_solved.sdk".to_string()
    } else {
        let input_path = std::path::Path::new(input_filename);
        let stem = input_path.file_stem().unwrap_or_default().to_string_lossy();
        format!("{}_solved.sdk", stem)
    };

    // Read entire input file
    let content = fs::read_to_string(input_filename)?;

    let mut solved_puzzles = Vec::new();
    let mut total_puzzles = 0;
    let mut solved_count = 0;

    println!("Solving Sudoku puzzles from {}...", input_filename);

    // Each non-empty line is treated as one puzzle
    for (i, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        total_puzzles += 1;

        let mut puzzle = match SudokuPuzzle::from_string(line) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Error parsing puzzle {}: {}", i + 1, e);
                solved_puzzles.push(line.to_string());
                continue;
            }
        };

        if puzzle.solve() {
            solved_puzzles.push(puzzle.to_string());
            solved_count += 1;
        } else {
            eprintln!("No solution found for puzzle {}", i + 1);
            solved_puzzles.push(line.to_string());
        }
    }

    // Write results to output file
    fs::write(&output_filename, solved_puzzles.join("\n"))?;

    println!("\n=== Results ===");
    println!("Total puzzles processed: {}", total_puzzles);
    println!("Successfully solved: {}", solved_count);
    println!("Solutions written to: {}", output_filename);

    Ok(())
}
