extern crate cmake;

fn main() {
	let mut build_out = cmake::build("helpers");
	build_out.push("lib");
	
	println!("cargo:rustc-link-search=native={}", build_out.display());
	println!("cargo:rustc-link-lib=static=helpers");
}