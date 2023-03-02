use std::{fs::File, io::Write};

#[cfg(debug_assertions)]
fn main() -> std::io::Result<()> {
    let out_dir = std::env::var("OUT_DIR").unwrap();

    let mut path = std::path::PathBuf::new();
    path.push(&out_dir);
    path.push("type-table.md");

    let generated = zod_core::docs::generate();
    let mut file = File::create(path)?;
    file.write_all(generated.as_bytes())?;
    Ok(())
}

#[cfg(not(debug_assertions))]
fn main() {}
