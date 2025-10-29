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
fn fa(a: bool, b: bool, ci: bool) -> (bool, bool) { /* Full Adder */
    let so = a ^ b ^ ci; /* sum output */
    let co = a && b || ci && (a ^ b); /* carry output */
    
    (so, co)
}

/// 2-bit multiply with carry
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
    fa(si, x && y, ci) // previous sum, x&&y(current digit), previous carry
}

/// add two times 4 bits with carry
// allow : clippy
/*
   c3   c2   c1   c0
    ↑    ↑    ↑    ↑
  a3   a2   a1   a0
+ b3 + b2 + b1 + b0
---------------------
  s3   s2   s1   s0

*/
#[allow(clippy::too_many_arguments)] 
fn add4(
    a0: bool, a1: bool, a2: bool, a3: bool,
    b0: bool, b1: bool, b2: bool, b3: bool) 
    -> (bool, bool, bool, bool, bool) {
    let (s0, c0) = fa(a0, b0, false); // lsb. and there is no carry-in
    let (s1, c1) = fa(a1, b1, c0);
    let (s2, c2) = fa(a2, b2, c1);
    let (s3, c3) = fa(a3, b3, c2);   // c3 : overflow bit

    (s0, s1, s2, s3, c3)
}

/// multiply two times 4 bits giving 8 bits
/*
	- x0 : lsb of x
	- x3 : msb of x
	- y0 ~ y3 : bits of y

			x3  x2  x1  x0
		x   y3  y2  y1  y0
		-------------------
			p0  p0  p0  p0    (x * y0)
		p1  p1  p1  p1   0    (x * y1, shifted left by 1)
	p2  p2  p2  p2  0    0    (x * y2, shifted left by 2)
*/

#[allow(clippy::too_many_arguments)]
fn mul4x4(
    x0: bool, x1: bool, x2: bool, x3: bool,
    y0: bool, y1: bool, y2: bool, y3: bool) 
    -> (bool, bool, bool, bool, bool, bool, bool, bool) {

	// First partial product (X * y0)
	let (s0, s1, s2, s3, c0) = add4(x0 && y0, x1 && y0, x2 && y0, x3 && y0, false, false, false, false); // tuple  : efficient to use when there are many return valeus
	// Second partial product (X * y1), shift one space and then add
	let (s1b, s2b, s3b, s4, c1) = add4(s1, s2, s3, c0, x0 && y1, x1 && y1, x2 && y1, x3 && y1);
	//  Third partial product
	let (s2c, s3c, s4c, s5, c2) = add4(s2b, s3b, s4, c1, x0 && y2, x1 && y2, x2 && y2, x3 && y2);
	// Fourth partial product
	let (s3d, s4d, s5d, s6, c3) = add4(s3c, s4c, s5, c2,x0 && y3, x1 && y3, x2 && y3, x3 && y3);
	// 마지막 carry까지 합쳐서 8비트 결과 생성
	(s0, s1b, s2c, s3d, s4d, s5d, s6, c3)
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


