// 21.10.2025
// June Park
// if statement practice

use std::io;

fn main() {
    let mut line = String::new();
    io::stdin().read_line(&mut line).expect("Failed to read line");
    let mut iter = line.trim().split_whitespace();

    let a: i32 = iter.next().unwrap().parse().unwrap();
    let b: i32 = iter.next().unwrap().parse().unwrap();

    if a<b{
        println!("<");
    } else if a>b{
        println!(">");
    } else{
        println!("==");
    }
}