// cargo run ~/treeG.txt ~/treeF.txt > out.txt

use std::fs::File;
use std::io::{BufRead, BufReader};
use similar::{Algorithm, capture_diff_slices, ChangeTag};

// Ord required for similar, others for Ord:
// https://doc.rust-lang.org/std/cmp/trait.Ord.html#how-can-i-implement-ord
#[derive(Debug, Default, Clone, Hash, Eq, PartialOrd, Ord)]
struct Line {
    path: String,
    size: i64,
    indent: usize,
}

impl PartialEq for Line {
    fn eq(&self, other: &Self) -> bool {
        let d = self.size - other.size;
        // 3584 seems to be the size of a folder on exfat (vs ntfs)
        self.path == other.path && self.indent == other.indent && d % 3584 == 0
    }
}

impl std::fmt::Display for Line {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}[{:>14}] {}", "\t".repeat(self.indent), self.size, self.path)
    }
}

impl From<String> for Line {
    fn from(value: String) -> Self {
        let opn = value.chars().position(|c| c == '[');
        if opn.is_none() {
            eprintln!("Missing [ -- skipping: '{value}'");
            return Self::default();
        }
        let opn = opn.unwrap();

        let cls = value.chars().position(|c| c == ']').unwrap();

        if opn % 4 != 0 {
            panic!("Invalid indent: {value}");
        }

        let indent = opn / 4;

        let path = value
            .chars()
            .skip(cls + 1)
            .collect::<String>()
            .trim()
            .to_owned();

        let size = value
            .chars()
            .skip(opn + 1)
            .take(cls - opn - 1)
            .collect::<String>();

        let size = size
            .trim()
            .parse()
            .expect(&format!("Invalid size: {} in {value}", size));

        Self { path, size, indent }
    }
}

fn main() {
    let mut args = std::env::args().skip(1);

    let a = args.next().expect("Need filename A");
    let b = args.next().expect("Need filename B");

    let a = File::open(a).expect("Filename A must be readable");
    let b = File::open(b).expect("Filename B must be readable");

    let a: Vec<_> = BufReader::new(a).lines().flatten().map(Line::from).collect();
    let b: Vec<_> = BufReader::new(b).lines().flatten().map(Line::from).collect();

    println!("Diffing...");

    let ops = capture_diff_slices(Algorithm::Myers, &a, &b);
    for op in ops {
        let changes = op.iter_changes(&a, &b);
        for change in changes {
            match change.tag() {
                ChangeTag::Delete => println!("-{}", change.value()),
                ChangeTag::Insert => println!("+{}", change.value()),
                ChangeTag::Equal => {},
            }
        }
    }

    println!("Finished!");
}
