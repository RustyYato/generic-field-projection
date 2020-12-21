use std::path::Path;

type Result = std::result::Result<(), Box<dyn std::error::Error>>;

#[path = "build/tuple.rs"]
mod tuple;

fn main() -> Result {
    let out_dir = std::env::var_os("OUT_DIR").ok_or("OUT_DIR not found!")?;
    let out_dir = Path::new(&out_dir);

    tuple::generate_impls(out_dir)?;

    Ok(())
}
