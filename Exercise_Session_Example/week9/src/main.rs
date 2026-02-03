/* Find the Sum of even numbers each multiplied by 3 from 6 to 6 million using iterators (loop and filter)*/
// previous
fn main() {
    // used u64 because the sum will prolly exceed the maximum value of u32 (4x10^9)
    let sum: u64 = (6..=6_000_000)  // Range from 6 to 6 million inclusive
        .filter(|&x| x % 2 == 0)    // Filter even numbers
        .map(|x| x * 3)             // Multiply each by 3
        .sum();                     // Sum them up

    println!("The sum of even numbers each multiplied by 3 from 6 to 6 million is: {}", sum);
}

use std::thread::sleep;
use std::time::Duration;

fn main() {
    let iter = (6..=6_000_000)  // Range from 6 to 6 million inclusive
        .filter(|&x| x % 2 == 0)// Filter even numbers
        .map(|x| x * 3);        // Multiply each by 3
   
    //  Feedback : i need to tell iterator to use it every seconds ? 이라는데 머야
    let mut sum: u64 = 0;
    // Without using for loop, or while loops, we can use iterator's next method in a loop
    let mut iter = iter.into_iter();    // Convert to iterator
    loop {
        match iter.next() {
            Some(value) => sum += value as u64,
            None => break,
        }
    }
    println!("The sum of even numbers each multiplied by 3 from 6 to 6 million is: {}", sum);
}
