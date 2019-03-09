//! This crate provides the cleaning memory allocator `MAProper`.
//!
//! ## What is `MAProper`
//! `MAProper` is an extension around `std::alloc::System` which ensures that the allocated memory
//! is always erased before it is deallocated by using one of
//! `memset_s`/`SecureZeroMemory`/`explicit_bzero`/`explicit_memset`.
//!
//! ## Whats the purpose of `MAProper`
//! `MAProper` becomes handy if you're dealing with a lot of sensitive data: because the memory
//! management of dynamically allocating types like `Vec` or `String` is opaque, you basically have
//! no real chance to reliably erase their sensitive contents.
//!
//! However they all use the global allocator – so all ways lead to Rome (or in this case to the
//! global allocator's `alloc` and `dealloc` functions) – which is where `MAProper` is sitting and
//! waiting to take care of the discarded memory.
//!
//! ## Using `MAProper` as global allocator (example)
//! ```
//! # use ma_proper::MAProper;
//! #[global_allocator]
//! static MA_PROPER: MAProper = MAProper;
//!
//! fn main() {
//! 	// This `Vec` will allocate memory through `MA_PROPER` above
//! 	let mut v = Vec::new();
//! 	v.push(1);
//! }
//! ```
//!
//! ## Important
//! Please note that `MAProper` only erases memory that is deallocated properly. This especially
//! means that:
//!  - stack items are __not overwritten__ by this allocator (therefore we expose `MAProper::erase`
//!    or `MAProper::erase_ptr<T>` so that you can erase them manually if necessary)
//!  - depending on your panic-policy and your `Rc`/`Arc` use (retain-cycles), the destructor (and
//!    thus the deallocator) may never be called
//!
//! ## ⚠️ Alpha-Warning ⚠️
//! This crate is in an early alpha state; so be careful and don't rely on it if you haven't checked
//! it by yourself!

use std::{ mem, ptr, os::raw::c_char, alloc::{ GlobalAlloc, System, Layout, handle_alloc_error } };


// Validate that the `usize` byte length is supported
#[cfg(not(any(target_pointer_width = "64", target_pointer_width = "32")))]
	compile_error!("Unsupported pointer width");


// Define the sizes
/// The byte length of an `usize`
const USIZE_LEN: usize = mem::size_of::<usize>();
/// The metadata length (this __MUST__ be a power of two)
const META_LEN: usize = 16;


/// A struct that implements the information necessary to handle a pointers metadata
struct Metadata;
impl Metadata {
	/// Computes a CRC-64-code (`CRC-64/XZ`) over `data`
	pub fn crc64(data: &[u8]) -> u64 {
		!data.iter().fold(0xFFFFFFFFFFFFFFFFu64, |crc, b| {
			(0..8).fold(crc ^ *b as u64, |crc, _| {
				let mask = (!(crc & 1)).wrapping_add(1);
				(crc >> 1) ^ (0xC96C5795D7870F42 & mask)
			})
		})
	}
	
	/// Returns the metadata-length necessary to maintain the alignment
	pub fn aligned_len(layout: Layout) -> Option<usize> {
		// Validate the alignment
		if !layout.align().is_power_of_two() { return None }
		
		// Compute the meta-length necessary to maintain the alignment
		if layout.align() < META_LEN { Some(META_LEN) }
			else { Some(layout.align()) }
	}
	
	/// Creates the metadata for `allocated` and writes it to `ptr`
	pub unsafe fn write(ptr: *mut u8, allocated: usize) {
		// Compute CRC-64
		let len: [u8; USIZE_LEN] = allocated.to_ne_bytes();
		let crc64: [u8; 8] = Self::crc64(&len).to_ne_bytes();
		
		// Zero the memory and copy the length and CRC
		erase_ptr(ptr, META_LEN);
		ptr::copy(len.as_ptr(), ptr, USIZE_LEN);
		ptr::copy(crc64.as_ptr(), ptr.add(META_LEN - 8), 8);
	}
	
	/// Decodes the metadata
	pub unsafe fn read(ptr: *const u8) -> Option<usize> {
		// Decode the length
		let mut len = [0u8; USIZE_LEN];
		ptr::copy(ptr, len.as_mut_ptr(), USIZE_LEN);
		
		// Compute and validate the checksum
		let mut crc64 = [0u8; 8];
		ptr::copy(ptr.add(META_LEN - 8), crc64.as_mut_ptr(), 8);
		
		if Self::crc64(&len) != u64::from_ne_bytes(crc64) { None }
			else { Some(usize::from_ne_bytes(len)) }
	}
}


/// The `MAProper` memory allocator
///
/// This memory allocator is an extension around `std::alloc::System` which ensures that the
/// allocated memory is always erased before it is deallocated.
///
/// ## Using `MAProper` as global allocator
/// ```
/// # use ma_proper::MAProper;
/// #[global_allocator]
/// static MA_PROPER: MAProper = MAProper;
///
/// fn main() {
/// 	// This `Vec` will allocate memory through `MA_PROPER` above
/// 	let mut v = Vec::new();
/// 	v.push(1);
/// }
/// ```
///
/// ## How it works
///
/// ### Allocation
/// To ensure that we have enough information to erase everything, we allocate slightly more memory
/// than requested and prepend some checksummed metadata to it. So a final chunk looks like this:
/// ```asciiart
/// Layout: [ metadata | alignment padding | requested memory ]
/// Length:   META_LEN |      dynamic      |  user specified
/// ```
///
/// Then we increment the pointer so that it points to `requested memory` and return it.
///
/// ## Deallocation
/// Once the pointer is to be deallocated, we rewind the pointer so that it points to
/// `metadata/length info` again to read and verify it. Once we know the length, we overwrite the
/// _entire_ allocated space using one of
/// `memset_s`/`SecureZeroMemory`/`explicit_bzero`/`explicit_memset`.
///
/// Then we deallocate it.
///
/// ## Important
/// Please note that this allocator only erases memory that is deallocated properly. This especially
/// means that:
///  - stack items are __not overwritten__ by this allocator (you have to call `MAProper::erase` or
///    `MAProper::erase_ptr<T>` manually if those items contain sensitive data)
///  - depending on your panic-policy and your `Rc`/`Arc` use (retain-cycles), the destructor (and
///    thus the deallocator) may never be called
pub struct MAProper;
unsafe impl GlobalAlloc for MAProper {
	unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
		// Check for `0` allocation
		if layout.size() == 0 { return ptr::null_mut() };
		
		// Precompute the meta length
		let meta_len = match Metadata::aligned_len(layout) {
			Some(meta_len) => meta_len,
			None => die(b"Invalid layout\0")
		};
		
		// Allocate and zero memory
		let to_allocate = meta_len + layout.size();
		let ptr = match Layout::from_size_align(to_allocate, layout.align()) {
			Ok(layout) => GlobalAlloc::alloc(&System, layout),
			Err(_) => handle_alloc_error(layout)
		};
		
		// Write the metadata and return the usable pointer
		Metadata::write(ptr, to_allocate);
		trace(
			'+', ptr,
			layout.size(), to_allocate, layout.align()
		);
		ptr.add(meta_len)
	}
	
	unsafe fn dealloc(&self, mut ptr: *mut u8, layout: Layout) {
		// Precompute the meta length and decrement the pointer
		let meta_len = match Metadata::aligned_len(layout) {
			Some(meta_len) => meta_len,
			None => die(b"Invalid layout\0")
		};
		ptr = ptr.sub(meta_len);
		
		// Read the allocate length and erase the pointer
		let allocated = match Metadata::read(ptr) {
			Some(allocated) => allocated,
			None => die(b"Invalid CRC for metadata\0")
		};
		erase_ptr(ptr, allocated);
		
		// Free the pointer
		match Layout::from_size_align(allocated, layout.align()) {
			Ok(layout) => GlobalAlloc::dealloc(&System, ptr, layout),
			Err(_) => die(b"Invalid layout\0")
		};
		trace(
			'-', ptr,
			layout.size(), allocated, layout.align()
		);
	}
}


/// Erases a byte slice
pub fn erase_slice(mut s: impl AsMut<[u8]>) {
	let s = s.as_mut();
	unsafe{ erase_ptr(s.as_mut_ptr(), s.len()) }
}
/// Erases `element_count` elements of type `T` referenced by `ptr`
pub unsafe fn erase_ptr<T>(ptr: *mut T, element_count: usize) {
	// Create the `u8` pointer and compute it's length
	let ptr = ptr as *mut u8;
	let len = element_count * mem::size_of::<T>();
	
	// Erase the pointer
	extern "C" {
		fn ma_proper_memzero(ptr: *mut u8, len: usize);
	}
	ma_proper_memzero(ptr, len);
}


/// Prints tracing information if the `trace` feature is enabled
#[cfg(feature = "trace")]
fn trace(prefix: char, ptr: *const u8, requested: usize, allocated: usize, alignment: usize) {
	extern "C" {
		fn trace(
			prefix: c_char, ptr: *const u8,
			requested: usize, allocated: usize, alignment: usize
		);
	}
	unsafe{ trace(prefix as c_char, ptr, requested, allocated, alignment) }
}

/// No-op if the `trace` feature is disabled
#[cfg(not(feature = "trace"))]
fn trace(_prefix: char, _ptr: *const u8, _requested: usize, _allocated: usize, _alignment: usize) {}


/// Prints information about an error and aborts the process
fn die(message: &'static[u8]) -> ! {
	extern "C" {
		fn die(message: *const c_char) -> !;
	}
	unsafe{ die(message.as_ptr() as *const c_char) }
}

