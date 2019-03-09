use ma_proper::MAProper;
use std::{ slice, cmp::max, alloc::{ Layout, GlobalAlloc } };


struct TestVectorOk {
	pub size: usize,
	pub align: usize,
	pub metadata: &'static[u8; 16]
}
impl TestVectorOk {
	pub fn test(&self) {
		// Allocate pointer
		let layout = Layout::from_size_align(self.size, self.align).unwrap();
		let ptr = unsafe{ MAProper.alloc(layout) };
		
		// Validate metadata
		let aligned_len = max(16, self.align);
		let metadata =
			unsafe{ slice::from_raw_parts(ptr.sub(aligned_len), 16) };
		assert_eq!(metadata, self.metadata);
		
		// Deallocate pointer
		unsafe{ MAProper.dealloc(ptr, layout) }
	}
}


struct TestVectorErr {
	pub size: usize,
	pub align: usize
}
impl TestVectorErr {
	pub fn test(&self) {
		// Allocate pointer
		let layout = Layout::from_size_align(self.size, self.align).unwrap();
		let ptr = unsafe{ MAProper.alloc(layout) };
		
		// Introduce fault
		let aligned_len = max(16, self.align);
		let metadata =
			unsafe{ slice::from_raw_parts_mut(ptr.sub(aligned_len), 16) };
		metadata[14] = !metadata[14];
		
		// Deallocate pointer
		unsafe{ MAProper.dealloc(ptr, layout) }
	}
}


#[test] #[allow(unreachable_patterns)]
fn test_ok() {
	TestVectorOk {
		size: 9, align: 1,
		metadata: match true {
			cfg!(all(target_pointer_width = "64", target_endian = "big")) =>
				b"\x00\x00\x00\x00\x00\x00\x00\x19\x8F\x7E\x19\xEA\xEF\x08\x4F\x80",
			cfg!(all(target_pointer_width = "32", target_endian = "big")) =>
				b"\x00\x00\x00\x19\x00\x00\x00\x00\xCD\xB1\xEC\xBA\xB3\x91\x1A\x0B",
			cfg!(all(target_pointer_width = "64", target_endian = "little")) =>
				b"\x19\x00\x00\x00\x00\x00\x00\x00\x39\x0B\x0C\xAA\x90\x7B\xB6\x5D",
			cfg!(all(target_pointer_width = "32", target_endian = "little")) =>
				b"\x19\x00\x00\x00\x00\x00\x00\x00\x32\x33\x96\xA0\x53\x54\x0F\x4A",
			_ => panic!("Unsupported pointer width")
		},
	}.test();
	
	TestVectorOk {
		size: 67_108_864, align: 64,
		metadata: match true {
			cfg!(all(target_pointer_width = "64", target_endian = "big")) =>
				b"\x00\x00\x00\x00\x04\x00\x00\x40\xA5\x66\x71\xD7\x21\xF3\xCA\x11",
			cfg!(all(target_pointer_width = "32", target_endian = "big")) =>
				b"\x04\x00\x00\x40\x00\x00\x00\x00\xE7\xA9\x84\x87\x7D\x6A\x9F\x9A",
			cfg!(all(target_pointer_width = "64", target_endian = "little")) =>
				b"\x40\x00\x00\x04\x00\x00\x00\x00\x06\x0C\xA2\x6F\x98\xC2\xBC\x5C",
			cfg!(all(target_pointer_width = "32", target_endian = "little")) =>
				b"\x40\x00\x00\x04\x00\x00\x00\x00\xE9\xD2\x8A\x79\xBC\xC1\x6D\x2D",
			_ => panic!("Unsupported pointer width")
		}
	}.test();
	
	TestVectorOk {
		size: 67_108_879, align: 4,
		metadata: match true {
			cfg!(all(target_pointer_width = "64", target_endian = "big")) =>
				b"\x00\x00\x00\x00\x04\x00\x00\x1F\x77\x90\xC5\x41\x30\x75\x36\x98",
			cfg!(all(target_pointer_width = "32", target_endian = "big")) =>
				b"\x04\x00\x00\x1F\x00\x00\x00\x00\x35\x5F\x30\x11\x6C\xEC\x63\x13",
			cfg!(all(target_pointer_width = "64", target_endian = "little")) =>
				b"\x1F\x00\x00\x04\x00\x00\x00\x00\xCB\x74\x13\xAA\xA7\x85\x35\xD7",
			cfg!(all(target_pointer_width = "32", target_endian = "little")) =>
				b"\x1F\x00\x00\x04\x00\x00\x00\x00\xC9\xA9\xF2\x93\x13\xAA\xB8\x7D",
			_ => panic!("Unsupported pointer width")
		}
	}.test();
}


#[test] #[ignore]
fn test_err() {
	TestVectorErr{ size: 67_108_879, align: 64 }.test();
}