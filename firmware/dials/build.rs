use std::{env, fs, io::Write};

fn main() {
    let outdir = env::var("OUT_DIR").unwrap();
    let outfile = format!("{}/../../../dials-version.txt", outdir);

    let mut fh = fs::File::create(&outfile).unwrap();
    write!(fh, "{}", chrono::Local::now()).ok();
}
