//! Demonstration of 4-bit gate level binary addition and multiplication.
//! [See also](https://coertvonk.com/inquiries/computer-math/diode-resistor-logic-30701) 
//!
//! Scientific Computing
//! TK 04Oct2024-20Oct2024
//!
use std::env;

/// 2-bit add with carry
///
/// | a | b | ci | so | co |
/// |---|---|----|----|----|
/// | 0 | 0 |  0 |  0 |  0 |
/// | 0 | 0 |  1 |  1 |  0 |
/// | 0 | 1 |  0 |  1 |  0 |
/// | 1 | 0 |  0 |  1 |  0 |
/// | 0 | 1 |  1 |  0 |  1 |
/// | 1 | 0 |  1 |  0 |  1 |
/// | 1 | 1 |  0 |  0 |  1 |
/// | 1 | 1 |  1 |  1 |  1 |
/// |---|---|----|----|----|
fn fa(a: bool, b: bool, ci: bool) -> (bool, bool) {
    let so = a ^ b ^ ci;
    let co = a && b || ci && (a ^ b);
    
    (so, co)
}

/// 2-bit multiply with cary
///
/// | x | y | si | ci | b | co | so |
/// |---|---|----|----|---|----|----|
/// | 0 | * |  0 |  0 | 0 |  0 |  0 |
/// | * | 0 |  0 |  0 | 0 |  0 |  0 |
/// | 0 | * |  0 |  1 | 0 |  0 |  1 |
/// | * | 0 |  0 |  1 | 0 |  0 |  1 |
/// | 0 | * |  1 |  0 | 0 |  0 |  1 |
/// | * | 0 |  1 |  0 | 0 |  0 |  1 |
/// | 0 | * |  1 |  1 | 0 |  1 |  0 |
/// | * | 0 |  1 |  1 | 0 |  1 |  0 |
/// | 1 | 1 |  0 |  0 | 1 |  0 |  1 |
/// | 1 | 1 |  0 |  1 | 1 |  1 |  1 |
/// | 1 | 1 |  1 |  * | 1 |  1 |  0 |
/// |---|---|----|----|---|----|----|
fn ma(x: bool, y: bool, ci: bool, si: bool) -> (bool, bool) {
    fa(si, x && y, ci)
}

/// add two times 4 bits with carry
#[allow(clippy::too_many_arguments)]
fn add4(
    a0: bool, a1: bool, a2: bool, a3: bool,
    b0: bool, b1: bool, b2: bool, b3: bool) 
    -> (bool, bool, bool, bool, bool) {
    let (s0, c0) = fa(a0, b0, false);
    let (s1, c1) = fa(a1, b1, c0);
    let (s2, c2) = fa(a2, b2, c1);
    let (s3, c3) = fa(a3, b3, c2);   

    (s0, s1, s2, s3, c3)
}

/// multiply two times 4 bits giving 8 bits
#[allow(clippy::too_many_arguments)]
fn mul4x4(
    x0: bool, x1: bool, x2: bool, x3: bool,
    y0: bool, y1: bool, y2: bool, y3: bool) 
    -> (bool, bool, bool, bool, bool, bool, bool, bool) {

    // add the missing code here
}

/// Split an integer between 0 and 15 into 4 bits
fn split4(x: u8) -> (bool, bool, bool, bool) {
    assert!(x < 16);
    
    (x & 1 != 0, x & 2 != 0, x & 4 != 0, x & 8 != 0)
}

/// Merge 4 bits giving an integer
fn merge4(x0 : bool, x1: bool, x2: bool, x3: bool) -> u8 {
	(  if x0 { 1 } else { 0 })
	+ (if x1 { 2 } else { 0 })
	+ (if x2 { 4 } else { 0 })
	+ (if x3 { 8 } else { 0 })
}

/// Merge 8 bits giving an integer
#[allow(clippy::too_many_arguments)]
fn merge8(x : (bool, bool, bool, bool, bool, bool, bool, bool)) -> u8 {
	merge4(x.0, x.1, x.2, x.3) + 16 * merge4(x.4, x.5, x.6, x.7)
}

/// Add and multiply two numbers between 0 and 15. 
/// Example: cargo run 4 7
fn main() {
	// parse and check command line arguments
	let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
		eprintln!("usage: {} number number", &args[0]); 
		std::process::exit(1);
    }	
    let x = args[1].parse().unwrap_or_else(|err| panic!("argument 1 \"{}\": {err}", &args[1]));
    let y = args[2].parse().unwrap_or_else(|err| panic!("argument 2 \"{}\": {err}", &args[2]));

	if x > 15 || y > 15 {
		panic!("Argument > 15: {x} {y}");
	}
	
	//	Split the number into bits 
	let a = split4(x);
    let b = split4(y);
    
    // Add the bits and print the result
    let z = add4(a.0, a.1, a.2, a.3, b.0, b.1, b.2, b.3);

	print!("{x} + {y} = ");
	
    if z.4 {
		println!("Overflow!");
    } else {
		let r = merge4(z.0, z.1, z.2, z.3);
		assert_eq!(x + y, r);
		println!("{r}"); 
    }
    
    // Multiply the bits and print the result
    let z = mul4x4(a.0, a.1, a.2, a.3, b.0, b.1, b.2, b.3);
    let r = merge8(z);
	assert_eq!(x * y, r);
	println!("{x} * {y} = {r}");
}

/// Check addition for all allowed input values
#[test]
fn check_add() {
	for x in 0 ..= 15 {
		for y in 0 ..= 15 {
			let a = split4(x);
			let b = split4(y);
			let z = add4(a.0, a.1, a.2, a.3, b.0, b.1, b.2, b.3);
			if x + y > 15 {
				assert_eq!(z.4, true);
			} else {
				assert_eq!(z.4, false);
				let r = merge4(z.0, z.1, z.2, z.3);
				assert_eq!(x + y, r);
			}
		}
	}
}

/// Check multiplcation for all allowed values
#[test]
fn check_mul() {
	for x in 0 ..= 15 {
		for y in 0 ..= 15 {
			let a = split4(x);
			let b = split4(y);		
			let z = mul4x4(a.0, a.1, a.2, a.3, b.0, b.1, b.2, b.3);
			let r = merge8(z);
			assert_eq!(x * y, r);			
		}
	}
}


