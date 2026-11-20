//! Embeds every sprite under `assets/sprites/` into the binary.
//!
//! For each name it records the text grid (`name.txt`) and, when present, a
//! PNG of the same name (`name.png`) that overrides the grid. Adding,
//! changing or removing a file there triggers a rebuild, so dropping in a PNG
//! needs no code change.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;

fn main() {
    let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let dir = Path::new(&manifest).join("assets").join("sprites");
    println!("cargo:rerun-if-changed={}", dir.display());

    // name -> (text path, png path)
    let mut found: BTreeMap<String, (Option<String>, Option<String>)> = BTreeMap::new();
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let (Some(stem), Some(ext)) = (path.file_stem(), path.extension()) else {
                continue;
            };
            let name = stem.to_string_lossy().into_owned();
            let full = path.display().to_string();
            println!("cargo:rerun-if-changed={full}");
            match ext.to_string_lossy().to_ascii_lowercase().as_str() {
                "txt" => found.entry(name).or_default().0 = Some(full),
                "png" => found.entry(name).or_default().1 = Some(full),
                _ => {}
            }
        }
    }

    let mut out = String::from("&[\n");
    for (name, (text, png)) in &found {
        let text = match text {
            Some(p) => format!("Some(include_str!({p:?}))"),
            None => "None".into(),
        };
        let png = match png {
            Some(p) => format!("Some(include_bytes!({p:?}))"),
            None => "None".into(),
        };
        writeln!(
            out,
            "    SpriteSource {{ name: {name:?}, text: {text}, png: {png} }},"
        )
        .unwrap();
    }
    out.push(']');

    let dest = Path::new(&std::env::var("OUT_DIR").unwrap()).join("sprite_sources.rs");
    std::fs::write(dest, out).unwrap();
}
