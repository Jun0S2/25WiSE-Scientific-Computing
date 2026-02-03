use std::fmt::Display;

// fn longest_with_an_announcement_without_lifetime<T>(
//     x: &str,
//     y: &str,
//     ann: T,
// ) -> &str
// where
//     T: Display,
// {
//     println!("Announcement! {ann}");
//     if x.len() > y.len() { x } else { y }
// }
// fn longest(x: &str, y: &str) -> &str { 
//     if x.len() > y.len() { x } else { y }
//  }
fn main() {
    let string1 = String::from("long string is long");
    let string3 = String::from("short");
    let string4 = String::from("longer string");
    let num = 2;
    // let result = longest_with_an_announcement_without_lifetime(
    //     string3.as_str(),
    //     string4.as_str(),
    //     "Without lifetime annotation",
    // );
    println!("{}", num.to_string());
    // let result = longest(string3.as_str(), string4.as_str());
    // println!("The longest string is {}", result);
}
