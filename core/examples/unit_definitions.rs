use solveng_core::{num::Num, units::{PREFIXES, Units, display_name}};

fn main() {
    let units = Units::builtin_table();
    let mut names: Vec<String> = units.map.keys().cloned().collect();
    for base in units.map.keys() {
        for (prefix, _) in PREFIXES {
            let name = format!("{prefix}{base}");
            if units.prefixed_form(&name) { names.push(name); }
        }
    }
    names.sort();
    names.dedup();
    for name in names {
        let Some((factor, dim)) = units.lookup(&name) else { continue };
        let number = match factor {
            Num::R(r) => format!("frac({}, {})", r.numer(), r.denom()),
            Num::F(f) => format!("{f:.70}").trim_end_matches('0').trim_end_matches('.').to_string(),
        };
        let bases = dim.0.iter().map(|(n, e)| format!("#{}^({})", solveng_core::dim::BASE[solveng_core::dim::IDENTITIES.iter().position(|id| *id == n).unwrap()], e / 60)).collect::<Vec<_>>().join(" ");
        println!("{}", serde_json::json!({"name": name, "expression": format!("{number} {bases}"), "display": display_name(&name, &units)}));
    }
}
