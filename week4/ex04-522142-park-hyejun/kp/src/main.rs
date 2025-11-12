/**
Algorithm used - dp
*/
use std::io::{self, Read};
use std::time::Instant;

#[derive(Debug)]
struct Item {
    id: i64,
    value: i64,
    weight: usize,
}

fn main() {
    let start_time = Instant::now();

    // read whole stdin
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).expect("failed to read stdin");

    let mut iter = input.split_whitespace();
    let h: usize = match iter.next() {
        Some(s) => s.parse().expect("invalid H"),
        None => {
            eprintln!("No input");
            return;
        }
    };
    // read items
    let mut items: Vec<Item> = Vec::with_capacity(h);
    for _ in 0..h {
        let id: i64 = iter.next().expect("missing id").parse().expect("invalid id");
        let value: i64 = iter.next().expect("missing value").parse().expect("invalid value");
        let weight: usize = iter.next().expect("missing weight").parse().expect("invalid weight");
        items.push(Item { id, value, weight });
    }

    let capacity: usize = iter.next().expect("missing capacity").parse().expect("invalid capacity");

    // DP table: dp[i][w] = max value using first i items with capacity w
    // Use (h+1) x (capacity+1) table, so i can iterate from 1 to h
    let mut dp = vec![vec![0i64; capacity + 1]; h + 1];

    for i in 1..=h {
        let it = &items[i - 1];
        for w in 0..=capacity {
            // don't take
            let mut best = dp[i - 1][w];
            // take if fits
            if it.weight <= w {
                let cand = dp[i - 1][w - it.weight] + it.value; // take item i-1
                if cand > best {                                // better
                    best = cand;                                // update best
                }
            }
            dp[i][w] = best;                                    // store best value
        }
    }

    let total_value = dp[h][capacity];                          // maximum value with all items and full capacity

    // reconstruct chosen items
    let mut chosen: Vec<i64> = Vec::new();
    let mut w = capacity;
    let mut i = h;
    while i > 0 {
        // check if item i-1 was taken
        if dp[i][w] != dp[i - 1][w] {
            // item was taken
            let it = &items[i - 1]; // item i-1
            chosen.push(it.id); // record chosen item
            w = w.saturating_sub(it.weight); // reduce remaining capacity
        }
        i -= 1;     // move to previous item
    }
    chosen.reverse(); // in input order (increasing by i)

    let duration = start_time.elapsed();
    // print results
    print!("Items:");
    if !chosen.is_empty() {
        for id in &chosen {
            print!(" {}", id);
        }
    }
    println!();
    println!("Total: {}", total_value);
    println!("Time : {:.3} s", duration.as_secs_f64());
}
