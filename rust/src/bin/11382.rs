// 꼬마 정민
// 20.10.2025
// June Park

use std::io;
fn main() {
    let mut line = String::new();
    io::stdin().read_line(&mut line).expect("Failed to read line");
    let mut iter = line.trim().split_whitespace();

    //10^12
    // i32 :  2,147,483,647 -> so use i64
    let a: i64 = iter.next().unwrap().parse().unwrap();
    let b: i64 = iter.next().unwrap().parse().unwrap();
    let c: i64 = iter.next().unwrap().parse().unwrap();

    // print answer
    println!("{}", a + b + c);
}