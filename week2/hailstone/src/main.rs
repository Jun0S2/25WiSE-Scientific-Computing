use std::collections::HashMap;

/// Collatz sequence helper function.
/// From n ~ 1, calculate Collatz Sequence
/// Return the sequence length and max cal
/// Algorithm used : Memoization
fn collatz_info(n: u128, memo: &mut HashMap<u128, (u128, u128)>) -> (u128, u128) {
    if n == 1 {                                 /* Base Case */
        return (1, 1);                          // len 1, max 1
    }
    if let Some(&(len, max)) = memo.get(&n) {  /* Already Exists */
        return (len, max); 
    }

    let next = if n % 2 == 0 {                  /* Next in Sequence */
        n / 2                                   /* even case */              
    } else {                                    /* odd case */
        3 * n + 1
    };

    let (sub_len, sub_max) = collatz_info(next, memo);  /* get sub-results */
    let current_max = n.max(sub_max);                   /* current max */
    let current_len = sub_len + 1;                      /* current length */

    memo.insert(n, (current_len, current_max));         /* store in memo */
    (current_len, current_max)                          /* return current results */
}

fn main() {
    let threshold: u128 = 2_000_000_000_000; // example: 2*10^12
    let mut start_value: u128 = 1;           // starting from 1

    let mut memo: HashMap<u128, (u128, u128)> = HashMap::new(); // memoization map

    let result_start;
    let result_first_above_threshold;

    let result_len;
    let result_max;

    loop {
        let mut n = start_value;
        let mut local_max = n;
        let mut first_above_threshold = 0;

        while n != 1 {                      /* calculate Colltaz Sequence */
            if n > local_max {              /* update local max */
                local_max = n;
            }
            if first_above_threshold == 0 && n > threshold {         /* check threshold */
                first_above_threshold = n;                           /* record first exceed */
            }
            n = if n % 2 == 0 { n / 2 } else { 3 * n + 1 };          /* next in sequence */
        }

        if first_above_threshold != 0 {                              /* end case*/
            result_start = start_value;                              /* record results */
            result_first_above_threshold = first_above_threshold;    
            result_max = local_max;
            result_len = collatz_info(start_value, &mut memo).0;     /* get length from memoization */
            break;
        }

        start_value += 1;
    }

    println!(
        "{} {} {} {}",
        result_start, result_first_above_threshold, result_max, result_len
    );
}