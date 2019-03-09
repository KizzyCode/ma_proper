use ma_proper::MAProper;
use std::{ slice, mem };


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


struct TestVector(String);
impl TestVector {
	pub fn test(&mut self) {
		// Shrink the vector
		self.0.shrink_to_fit();
		
		// Get a pointer to the metadata
		let ptr = self.0.as_ptr() as *const u8;
		let metadata =
			unsafe{ slice::from_raw_parts(ptr.sub(16), 16) };
		
		// Compare the metadata
		let expected = make_metadata(16 + self.0.capacity());
		assert_eq!(metadata, expected);
		
		// Overwrite the entire string
		ma_proper::erase_slice(unsafe{ self.0.as_bytes_mut() });
	}
}


#[test]
fn test() {
	TestVector("Testolope".to_string()).test();
	TestVector("Lorem ipsum dolor si amet, consequetor blubbeldidubbus...".to_string()).test();
}