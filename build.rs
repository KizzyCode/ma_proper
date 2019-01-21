extern crate cc;

fn main() {
	// Determine which secure memset implementation to use
	#[allow(unreachable_patterns)]
	let secure_memset = match true {
		cfg!(target_os = "macos")
			=> "USE_MEMSET_S",
		cfg!(target_os = "windows")
			=> "USE_SECUREZEROMEMORY",
		cfg!(target_os = "freebsd") | cfg!(target_os = "openbsd")
			=> "USE_EXPLICIT_BZERO",
		cfg!(target_os = "netbsd")
			=> "USE_EXPLICIT_MEMSET",
		
		// Check if we should use the fallback with volatile pointers
		_ if cfg!(feature = "volatile_fallback") => "USE_VOLATILE_POINTERS",
		_ => panic!("No secure memset implementation known")
	};
	
	// Compile the library
	cc::Build::new()
		.file("helpers/helpers.c")
		.define(secure_memset, None)
		.static_flag(true)
		.flag("-std=c11")
		.compile("helpers");
	
	// Link the library
	println!("cargo:rustc-link-lib=static=helpers");
}