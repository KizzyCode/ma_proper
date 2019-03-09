use ma_proper::MAProper;
use std::{ slice, mem, cmp::max };


#[global_allocator]
static GLOBAL: MAProper = MAProper;


/// Compute the metadata for a length
fn make_metadata(len: usize) -> [u8; 16] {
	// Compute CRC
	let crc: u64 = !len.to_ne_bytes().iter().fold(0xFFFFFFFFFFFFFFFFu64, |crc, b| {
		(0..8).fold(crc ^ *b as u64, |crc, _| {
			let mask = (!(crc & 1)).overflowing_add(1).0;
			(crc >> 1) ^ (0xC96C5795D7870F42 & mask)
		})
	});
	
	// Create metadata
	let mut metadata = [0u8; 16];
	metadata[..mem::size_of::<usize>()].copy_from_slice(&len.to_ne_bytes());
	metadata[8..].copy_from_slice(&crc.to_ne_bytes());
	
	metadata
}


struct TestVector<T>(Vec<T>);
impl<T> TestVector<T> {
	pub fn test(&mut self) {
		// Shrink the vector
		self.0.shrink_to_fit();
		
		// Get a pointer to the metadata
		let aligned_len = max(16, mem::size_of::<T>());
		let ptr = self.0.as_ptr() as *const u8;
		let metadata =
			unsafe{ slice::from_raw_parts(ptr.sub(aligned_len), 16) };
		
		// Compare the metadata
		let expected =
			make_metadata(aligned_len + (self.0.capacity() * mem::size_of::<T>()));
		assert_eq!(metadata, expected);
		
		// Overwrite the entire vector
		unsafe{ ma_proper::erase_ptr(self.0.as_mut_ptr(), self.0.len()) }
	}
}


#[test]
fn test() {
	// A test struct that has an alignment of 64
	#[repr(align(64))] #[derive(Copy, Clone)]
	struct Aligned(u64);
	
	// Run tests
	TestVector(b"Testolope".to_vec()).test();
	TestVector(vec![Aligned(7); 1_108_864]).test();
	TestVector(vec![42u64; 1_108_879]).test();
}