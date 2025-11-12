/**
 \file      fibonacci.rs
 \brief     Examples for computing the fibonacci number
 \author    Thorsten Koch
 \version   1.0
 \date      18Oct2022 25Apr2023
*/ 

use std::arch::asm;
use std::env;

/// Classic recursive fibonacci computation
fn fib_recursive1(n: u32) -> u64 {
    if n <= 2 {
        return 1;
    }
    return fib_recursive1(n - 1) + fib_recursive1(n - 2);   
}


/// Save as above but writte in a different way
fn fib_recursive2(n: u32) -> u64 {
    if n <= 2 {
        1
    } else {
        fib_recursive2(n - 1) + fib_recursive2(n - 2)
    }   
}


/// Iterative fibonacci computation
fn fib_iterative1(mut n: u32) -> u64 {
    let mut a: u64 = 0;
    let mut b: u64 = 1;
    
    while n > 0 {
        let t = a + b;
        a     = b;
        b     = t;
        n    -= 1;
    }
    return a;
}

/// One can do without the temorary variable  and using a for loop
fn fib_iterative2(n: u32) -> u64 {
    let mut a = 0u64;
    let mut b = 1u64;

   for _ in 0..n {
       b += a;
       a  = b - a;
   }
   a
}

/// Maybe these variable names are easier to understand, however the code is more complicated
fn fib_iterative3(n: u32) -> u64 {
    if n <= 2 {
        return 1;
    }
    let mut prev  : u64 = 1;
    let mut result: u64 = 2;

    for _ in 3..n {
        let prev_prev = prev;
        prev          = result;
        result        = prev + prev_prev;
    }
    result
}


/// Without the if in the beginning
fn fib_iterative4(n: u32) -> u64 {
    let mut a_1: u64 = 1;
    let mut a  : u64 = 2;

    for _ in 2 .. n {
        let a_2 = a_1;
        a_1     = a;
        a       = a_2 + a_1;
    }
    a_1
}

/// Actually, there exists and instruction on x86 for this.
/// WARNING: will only compile on x86 architecture (i486 upwards)
fn fib_iterative_asm(n: u32) -> u64 {
    let mut a: u64 = 0;
    let mut b: u64 = 1;

    for _ in 0 .. n {
        unsafe {
            /* TEMP <- SRC + DEST;
             * SRC  <- DEST;
             * DEST <- TEMP;
             */ 
            asm!("xadd {}, {}", inout(reg) a, inout(reg) b);
            //      asm("xadd %1, %0" : "+r" (b), "+r" (a));
        }
    } 
    a
}


/// Direct computation
fn fib_direct(n: u32) -> u64 {
    let sqrt5 = f64::sqrt(5.0);
    let phi  = (1.0 + sqrt5) / 2.0;

    (f64::powf(phi, n as f64) / sqrt5).round() as u64
}

#[test]
fn check_all() {
    let expected = [ 0, 1, 1, 2, 3, 5, 8, 13, 21, 34, 55 ];

    for i in 1..11 {
        assert_eq!(fib_iterative1(i),    expected[i as usize]);
        assert_eq!(fib_iterative2(i),    expected[i as usize]);
        assert_eq!(fib_iterative3(i),    expected[i as usize]);
        assert_eq!(fib_iterative4(i),    expected[i as usize]);
        assert_eq!(fib_iterative_asm(i), expected[i as usize]);
        assert_eq!(fib_recursive1(i),    expected[i as usize]);
        assert_eq!(fib_recursive2(i),    expected[i as usize]);
        assert_eq!(fib_direct(i),        expected[i as usize]);
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!("usage: {} function-no n\n", &args[0]); 
        eprintln!("       compute fibonacci(n) using function 1..10");
       
        std::process::exit(1);
    }
    let function_no = args[1].parse().unwrap();
    let n           = args[2].parse().unwrap();

    print!("n = {n} : ");

    match function_no {
        1 => println!("iterative 1   = {}", fib_iterative1(n)),
        2 => println!("iterative 2   = {}", fib_iterative2(n)),
        3 => println!("iterative 3   = {}", fib_iterative3(n)),
        4 => println!("iterative 4   = {}", fib_iterative4(n)),
        5 => println!("iterative asm = {}", fib_iterative_asm(n)),
        6 => println!("recursive 1   = {}", fib_recursive1(n)),
        7 => println!("recursive 3   = {}", fib_recursive2(n)),
        8 => println!("direct        = {}", fib_direct(n)),
        _ => eprintln!("error: illegal function number {function_no}"),
    }
}

