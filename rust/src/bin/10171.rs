// 20.10.2025
// June Park
// escape letter : \\
// " : \"
// raw string : \나 "가 많으면 raw string 추천
// - r#"..."# 안에서는 \나 "를 그대로 쓸 수 있음.
// - # 개수 늘리면 내부 "가 더 많아도 문제 없음.
fn main() {
    // Use raw string with #
    let art = r#"
    \    /\
     )  ( ')
    (  /  )
     \(__)|
    "#;
    println!("{}", art);
}