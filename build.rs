/// Checks if the current glibc version supports `explicit_bzero`
#[cfg(target_os = "linux")]
fn linux_check_explicit_bzero() -> Option<&'static str> {
	// Get libc version
	use std::{ u32, ffi::CStr, os::raw::c_char, str::FromStr };
	extern "C" {
		// const char *gnu_get_libc_version(void);
		fn gnu_get_libc_version() -> *const c_char;
	}
	let v: Vec<u32> = unsafe{ CStr::from_ptr(gnu_get_libc_version()) }.to_str().unwrap()
		.split(".").map(|s|  u32::from_str(s).unwrap()).collect();
	
	// Validate version
	match (v[0], v[1]) {
		(2...u32::MAX, 25...u32::MAX) => Some("USE_EXPLICIT_BZERO"),
		_ => None
	}
}


fn main() {
	// Determine which secure memset implementation to use
	#[allow(unused_assignments)]
	let mut secure_memset = None;
	
	#[cfg(target_os = "macos")] { secure_memset = Some("USE_MEMSET_S") }
	#[cfg(target_os = "ios")] { secure_memset = Some("USE_MEMSET_S") }
	#[cfg(target_os = "windows")] { secure_memset = Some("USE_SECUREZEROMEMORY") }
	#[cfg(target_os = "freebsd")] { secure_memset = Some("USE_EXPLICIT_BZERO") }
	#[cfg(target_os = "openbsd")] { secure_memset = Some("USE_EXPLICIT_BZERO") }
	#[cfg(target_os = "netbsd")] { secure_memset = Some("USE_EXPLICIT_MEMSET") }
	#[cfg(target_os = "linux")] { secure_memset = linux_check_explicit_bzero() }
	
	// Check if we have a secure memset implementation
	let secure_memset = match secure_memset {
		Some(secure_memset) => secure_memset,
		None if cfg!(feature = "volatile_fallback") => "USE_VOLATILE_POINTERS",
		None => panic!("No secure memset implementation known for your target platform")
	};
	
	// Compile and link the library
	cc::Build::new()
		.file("helpers/helpers.c")
		.define(secure_memset, None)
		.compile("helpers");
	println!("cargo:rustc-link-lib=static=helpers");
}