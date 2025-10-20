// 20.10.2025
// June Park
// Python : 28776 KB, 72 ms
// RUST : 13200 KB, 0 ms

use std::io; //  How to get Input from Standard Input in Rust

fn main(){
    let mut line = String::new();
    io::stdin().read_line(&mut line).expect("Failed to read line");
    let mut iter = line.trim().split_whitespace();

    //  Save input in A,B (a, b are int)
    // in CPP, its cin >> a >> b
    // variables sholud be snake_case
    let a: i32 = iter.next().unwrap().parse().unwrap();
    let b: i32 = iter.next().unwrap().parse().unwrap();

    //  : - type annotation
    // i32 : 32 bit integer
    // let mut iter = line.trim().split_whitespace();
    // trim : 문자열 앞 뒤 공백 제거. 
    // split_whitespace() → 문자열을 공백 단위로 잘라서 Iterator(반복자) 생성
    // next().unwrap().parse().unwrap() 이유 
    // - iter.next() : iterator 에서 다음 요소를 가져오고 없으면 none
    // - unwrap() : Option type에서 갓 꺼내기. 없으면 panic 발생
    // - parse() : 문자열을 지정한 타입으로 변환
    // - unwrap() : parse 성공 시 결과는 result 타입 (OK(number) or error)
    println!("{}", a + b);
}