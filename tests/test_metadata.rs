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
			cfg!(target_pointer_width = "64") =>
				b"\x00\x00\x00\x00\x00\x00\x00\x19\x8F\x7E\x19\xEA\xEF\x08\x4F\x80",
			cfg!(target_pointer_width = "32") =>
				b"\x00\x00\x00\x19\x00\x00\x00\x00\xCD\xB1\xEC\xBA\xB3\x91\x1A\x0B",
			_ => panic!("Unsupported pointer width")
		}
	}.test();
	
	TestVectorOk {
		size: 67_108_864, align: 64,
		metadata: match true {
			cfg!(target_pointer_width = "64") =>
				b"\x00\x00\x00\x00\x04\x00\x00\x40\xA5\x66\x71\xD7\x21\xF3\xCA\x11",
			cfg!(target_pointer_width = "32") =>
				b"\x00\x00\x00\x00\x04\x00\x00\x40\xE7\xA9\x84\x87\x7D\x6A\x9F\x9A",
			_ => panic!("Unsupported pointer width")
		}
	}.test();
	
	TestVectorOk {
		size: 67_108_879, align: 4,
		metadata: match true {
			cfg!(target_pointer_width = "64") =>
				b"\x00\x00\x00\x00\x04\x00\x00\x1F\x77\x90\xC5\x41\x30\x75\x36\x98",
			cfg!(target_pointer_width = "32") =>
				b"\x00\x00\x00\x00\x04\x00\x00\x1F\x35\x5F\x30\x11\x6C\xEC\x63\x13",
			_ => panic!("Unsupported pointer width")
		}
	}.test();
}


#[test] #[ignore]
fn test_err() {
	TestVectorErr{ size: 67_108_879, align: 64 }.test();
}