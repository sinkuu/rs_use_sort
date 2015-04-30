use std::collections::{BTreeMap, BTreeSet};
use std::io::{BufRead, Read, Write};
use std::iter::IntoIterator;

#[derive(PartialOrd, Ord, PartialEq, Eq, Copy, Clone)]
enum Privacy {
    Public,
    Private,
}

#[derive(Copy, Clone)]
enum Indent {
    Space(usize),
    Tab(usize),
}

impl Indent {
    fn to_string(&self) -> String {
        match self {
            &Indent::Space(l) => std::iter::repeat(' ').take(l),
            &Indent::Tab(l) => std::iter::repeat('\t').take(l),
        }.collect()
    }
}

fn format_uses<R: BufRead, W: Write>(input: &mut R, output: &mut W) {
    let mut uses: BTreeMap<(Privacy, Vec<String>), BTreeSet<String>> = BTreeMap::new();
    let mut indent = None;

    for line in input.lines() {
        let line = line.unwrap();
        if line.len() == 0 { continue; }
        if indent.is_none() {
            indent = Some(match line.chars().next().unwrap() {
                ' ' => Indent::Space(line.chars().take_while(|&c| c == ' ').count()),
                '\t' => Indent::Tab(line.chars().take_while(|&c| c == '\t').count()),
                _ => Indent::Space(0),
            });
        }
        let line_trimmed = line.trim_right_matches(&[' ', '\t', ';'][..]);
        if line_trimmed.len() == 0 { continue; }
        let us: String = line_trimmed.chars().skip_while(|c| c.is_whitespace()).collect();
        let (p, idx) = {
            if us.starts_with("use ") { (Privacy::Private, Some(4)) }
            else if us.starts_with("pub use ") { (Privacy::Public, Some(8)) }
            else { (Privacy::Private, None) }
        };
        if let Some(idx) = idx {
            let mut path: Vec<String> = us[idx..].split("::").map(|s| s.trim_matches(' ').to_owned())
                .collect();
            if path[path.len() - 1].as_bytes()[0] == b'{' ||
                path[path.len() - 1].chars().next().unwrap().is_uppercase() {
                let last = path.pop().unwrap();
                let last : Vec<_> = last
                    .trim_left_matches('{')
                    .trim_right_matches('}')
                    .split(',')
                    .map(|i| i.trim_matches(' '))
                    .collect();
                let key = (p, path);
                if uses.contains_key(&key) {
                    let mut entry = uses.get_mut(&key).unwrap();
                    for e in last.into_iter() {
                        entry.insert(e.to_owned());
                    }
                } else {
                    uses.insert(key, last.into_iter().map(|s| s.to_owned()).collect());
                }
            } else {
                let key = (p, path);
                if uses.contains_key(&key) {
                    let mut entry = uses.get_mut(&key).unwrap();
                    entry.insert(String::new());
                } else {
                    let mut set = BTreeSet::new();
                    set.insert(String::new());
                    uses.insert(key, set);
                }
            }
        } else {
            panic!("Expected 'use' or 'pub use'");
        }
    }

    //println!("{:?}", uses);

    let indent_str = match indent {
        Some(indent) => indent.to_string(),
        None => String::new(),
    };
    for ((p, path), mut l) in uses.into_iter() {
        let mut pit = path.into_iter();
        let p0 = pit.next().unwrap();
        let path_s =
            pit.fold(p0, |mut acc, item| {
                acc.push_str("::");
                acc.push_str(&item);
                acc
            });
        if l.contains("") {
            write!(output, "{}", indent_str).unwrap();
            match p {
                Privacy::Public => writeln!(output, "pub use {};", path_s).unwrap(),
                Privacy::Private => writeln!(output, "use {};", path_s).unwrap(),
            }
            l.remove("");
        }

        if l.len() > 0 {
            if l.len() == 1 {
                write!(output, "{}", indent_str).unwrap();
                writeln!(output, "use {}::{};", path_s, l.into_iter().next().unwrap()).unwrap();
            } else {
                let mut lit = l.into_iter();
                let l0 = lit.next().unwrap();
                let last_s =
                    lit.into_iter().fold(l0, |mut acc, item| {
                        acc.push_str(", ");
                        acc.push_str(&item);
                        acc
                    });
                write!(output, "{}", indent_str).unwrap();
                writeln!(output, "use {}::{{{}}};", path_s, last_s).unwrap();
            }
        }
    }
}

#[cfg(not(test))]
fn main() {
    use std::io;

    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let mut stdout = io::stdout();
    format_uses(&mut stdin, &mut stdout);
}

#[cfg(test)]
mod test {
    #[test]
    fn test_sort() {
        let mut output = vec![];
        super::format_uses(&mut 
           "    use a::b::{C, B};
                use a::b::A;
                use a::b;
                use c::d::T;
                pub use p;
            ".as_bytes(),
            &mut output);
        let output = String::from_utf8(output).unwrap();
        assert_eq!(output, "    pub use p;\n    use a::b;\n    use a::b::{A, B, C};\n    use c::d::T;\n");
    }
}
