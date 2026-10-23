//! Prints every built-in unit name and every prefixed one, one per line, for scripts/unit-words.py.

use solveng_core::units::{PREFIXES, Units};

fn main() {
    let units = Units::default();
    let mut names: Vec<String> = units.map.keys().cloned().collect();
    for base in units.map.keys() {
        for (prefix, _) in PREFIXES {
            let name = format!("{prefix}{base}");
            if units.prefixed_form(&name) {
                names.push(name);
            }
        }
    }
    names.sort();
    names.dedup();
    for name in names {
        println!("{name}");
    }
}
