fn main() {
    let threshold: u128 = 1u128 << 64;            // 2^64
    let mut start: u128 = 1;                      // Starting value for Collatz sequence

    loop {                                        // Infinite loop to find the first sequence exceeding threshold
        let mut n = start;                        // Current value in the Collatz sequence
        let mut max_value = n;                    // Track maximum value in the sequence
        let mut first_above_threshold = 0u128;    // First value exceeding 2^64
        let mut sequence_length = 1u128;          // Length of the sequence

        // Compute the complete Collatz sequence for current starting value
        while n != 1 {                            // Continue until reaching 1
            if n % 2 == 0 {                       // Even number case
                n /= 2;
            } else {                              // Odd number case
                n = 3 * n + 1;
            }

            sequence_length += 1;                 // Increment sequence length
            
            // Update maximum value encountered in the sequence
            if n > max_value {
                max_value = n;
            }
            
            // Record the first value that exceeds 2^64
            if first_above_threshold == 0 && n > threshold {
                first_above_threshold = n;
            }
        }

        // Check if this sequence contained a value above 2^64
        if first_above_threshold != 0 {
            // Output format: starting_value first_value_above_2^64 max_value sequence_length
            println!("{} {} {} {}", start, first_above_threshold, max_value, sequence_length);
            return;
        }

        start += 1;
        
        // Progress indicator for long-running computation
        // This is just for my debugging purpose
        // if start % 10_000_000 == 0 {
        //     eprintln!("Progress: checked {} million numbers", start / 1_000_000);
        // }
    }
}