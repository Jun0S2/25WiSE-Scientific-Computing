//https://www.acmicpc.net/problem/25314
// 23.10.2025
// June Park

use std::io;

fn main (){
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let n: i32 = input.trim().parse().unwrap();
    for _ in 0..n/4 {
        print!("long ");
    }
    println!("int");
}