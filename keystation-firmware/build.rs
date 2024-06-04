use std::{env, io::Write, fs};

fn main() {
    let outdir = env::var("OUT_DIR").unwrap();
    let outfile = format!("{}/../../../key-firmware-version.txt", outdir);

    let mut fh = fs::File::create(&outfile).unwrap();
    write!(fh, r#""{}""#, chrono::Local::now()).ok();
}