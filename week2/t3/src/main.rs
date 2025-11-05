fn main() {
    let x = f32::from_bits(u32::from_str_radix("3FDDB3D7", 16).unwrap());
    println!("{}", x);  
}