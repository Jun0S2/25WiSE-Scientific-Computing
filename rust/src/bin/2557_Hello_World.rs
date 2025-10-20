// 20.10.2025
// TIL : Println 은 문자열을 기대하는데, str은 변수라 컴파일 에러가 난다.
// JAVA : 124 ms, Rust : 0ms
// June Park

fn main() {
	let str = "Hello World!";
	println!("{}",str);
}
