//https://www.acmicpc.net/problem/25304
// 23.10.2025
// for loop
// June Park

use std::io;

fn main() {
    /**
    * In Rust, you need to clear buffer by yourself unlike Java and C++ after \N
    * Rust는 read_line이 문자열 전체를 버퍼에서 읽어와 기존 String에 append하기 때문에 clear()가 필요
    */
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    let total_p: i32 = input.trim().parse().unwrap();  // total price
    input.clear();

    std::io::stdin().read_line(&mut input).unwrap();
    let mut n: i32 = input.trim().parse().unwrap();    // number of items

    // for N loops, price (a) and # (b) are given with whitespace
    let mut sum = 0;
    /* for loop (explicit)
    loop { 
        // you cant do -- operations in rust:  n--;
        n-=1;
        if n < 0 {
            break;
        }
        input.clear();
        std::io::stdin().read_line(&mut input).unwrap();
        let mut iter = input.trim().split_whitespace();
        let a: i32 = iter.next().unwrap().parse().unwrap(); // price
        let b: i32 = iter.next().unwrap().parse().unwrap(); // number of items
        sum += a * b;
    }
    */
    // for loop (concise)
    /**
    _는 사용하지 않는 반복 변수를 의미합니다.
    0..n은 0 이상 n 미만 반복 → 기존 n을 감소시키는 loop와 같은 효과
    */
    for _ in 0..n {
        input.clear();
        std::io::stdin().read_line(&mut input).unwrap();
        let mut iter = input.trim().split_whitespace();
        let a: i32 = iter.next().unwrap().parse().unwrap(); // price
        let b: i32 = iter.next().unwrap().parse().unwrap(); // number of items
        sum += a * b;
    }
    // Rust에서는 println! 매크로는 중괄호 대신 괄호를 사용해야 함
    // println!{if sum == total_p { "Yes" } else { "No" };}
    println!("{}", if sum == total_p { "Yes" } else { "No" });
    
}