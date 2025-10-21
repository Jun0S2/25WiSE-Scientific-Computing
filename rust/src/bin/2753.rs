// cpp ?: -> rust  doesnt need ? or :
// June Park
// https://www.acmicpc.net/problem/2753

use std::io;

fn main() {
    let mut line = String::new();
    io::stdin().read_line(&mut line).expect("Failed to read line");
    let mut iter = line.trim().split_whitespace();

    let a: i32 = iter.next().unwrap().parse().unwrap();

    let result = if (a % 100 != 0&& a % 4 == 0 ) ||  (a % 400 == 0) { 1 } else { 0 };
    println!("{}", result);   

}