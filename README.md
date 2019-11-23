[![BSD-2-Clause License](https://img.shields.io/badge/License-BSD--2--Clause-blue.svg)](https://opensource.org/licenses/BSD-2-Clause)
[![MIT License](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![docs.rs](https://docs.rs/ma_proper/badge.svg)](https://docs.rs/ma_proper)
[![crates.io](https://img.shields.io/crates/v/ma_proper.svg)](https://crates.io/crates/ma_proper)
[![Download numbers](https://img.shields.io/crates/d/ma_proper.svg)](https://crates.io/crates/ma_proper)
[![dependency status](https://deps.rs/crate/ma_proper/1.0.0/status.svg)](https://deps.rs/crate/ma_proper/1.0.0)
[![Travis CI](https://travis-ci.org/KizzyCode/ma_proper.svg?branch=master)](https://travis-ci.org/KizzyCode/ma_proper)
[![Appveyor CI](https://ci.appveyor.com/api/projects/status/github/KizzyCode/ma_proper?svg=true)](https://ci.appveyor.com/project/KizzyCode/ma-proper)


# MAProper
This crate provides the securely overwriting memory allocator `MAProper` 🧹


## What is `MAProper`
`MAProper` is an extension around `std::alloc::System` which ensures that the allocated memory is
always erased before it is deallocated by using one of
`memset_s`/`SecureZeroMemory`/`explicit_bzero`/`explicit_memset`.


## Whats the purpose of `MAProper`
`MAProper` becomes handy if you're dealing with a lot of sensitive data: because the memory
management of dynamically allocating types like `Vec` or `String` is opaque, you basically have no
real chance to reliably trace and erase their sensitive contents.

However they all use the global allocator – so all ways lead to Rome (or in this case to the global
allocator's `alloc` and `dealloc` functions) – which is where `MAProper` is sitting and waiting to
take care of the discarded memory.


## Using `MAProper` as global allocator (example)
```rust
#[global_allocator]
static MA_PROPER: ma_proper::MAProper = ma_proper::MAProper;

fn main() {
	// This `Vec` will allocate memory through `MA_PROPER` above
	let mut v = Vec::new();
	v.push(1);
}
```


## Important
Please note that `MAProper` only erases memory that is deallocated properly. This especially means
that:
 - stack items are __not erased__ by this allocator – to erase stack memory, we expose
   `MAProper::erase_slice` and `MAProper::erase_ptr<T>` so that you can erase them manually if
   necessary
 - depending on your panic-policy and your `Rc`/`Arc` use (retain-cycles), the destructor (and thus
   the deallocator) may never be called