use std::collections::HashSet;
use std::fmt::Display;

struct Progress<I>
where
    I: Iterator,
{
    iter: I,
    current: usize,
    total: Option<usize>,
}

impl<I> Progress<I>
where
    I: Iterator,
{
    fn new(iter: I, total: Option<usize>) -> Self {
        Progress { iter, current: 0, total }
    }

    fn draw_bar(&self) {
        if let Some(total) = self.total {
            let width = 30; // bar width
            let progress_ratio = self.current as f32 / total as f32;
            let filled = (width as f32 * progress_ratio) as usize;
            let empty = width - filled;

            let bar = format!(
                "[{}{}]",
                "█".repeat(filled),
                "░".repeat(empty)
            );

            let percent = progress_ratio * 100.0;

            print!("\r{} {:6.2}% ({}/{})",
                   bar,
                   percent,
                   self.current,
                   total,
            );
        } else {
            // total size unknown
            print!("\rProcessed: {}", self.current);
        }

        use std::io::Write;
        std::io::stdout().flush().unwrap();
    }
}

impl<I> Iterator for Progress<I>
where
    I: Iterator,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.iter.next()?;
        self.current += 1;
        self.draw_bar();
        Some(item)
    }
}

// Extension trait
// Ext convention : extension of standard traits
trait ProgressExt: Iterator + Sized {
    fn progress(self) -> Progress<Self> {
        let total = self.size_hint().1; // get upper bound
        Progress::new(self, total)
    }
}

impl<I> ProgressExt for I where I: Iterator {}

fn main() {
    let numbers = vec![1, 2, 3, 4, 5];

    for n in numbers.iter().progress() {
        do_something();
    }
    println!("\nDone!");

    let mut words = HashSet::new();
    words.insert("hello".to_string());
    words.insert("world".to_string());

    for w in words.into_iter().progress() {
        do_something();
    }
    println!("\nWords Done!");
}

fn do_something() {
    std::thread::sleep(std::time::Duration::from_millis(500));
}
