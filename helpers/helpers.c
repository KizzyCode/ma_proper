// Include `stdlib.h` for `size_t` and `stdio.h` for `fprintf`
#include <stdlib.h>
#include <stdio.h>
#include <stdint.h>

// Function-dependent includes
#if defined(USE_MEMSET_S)
	#define __STDC_WANT_LIB_EXT1__ 1
	#include <string.h>
#elif defined(USE_SECUREZEROMEMORY)
	#include <windows.h>
	#include <wincrypt.h>
#elif defined(USE_EXPLICIT_BZERO)
	#include <strings.h>
	#include <string.h>
#elif defined(USE_EXPLICIT_MEMSET)
	#include <string.h>
#endif


/// Prints trace information
///
/// \param prefix The prefix to print (indicates the action)
/// \param ptr The pointer that was managed
/// \param requested The requested bytes
/// \param allocated The allocated bytes
/// \param alignment The alignment
void trace(char prefix, uint8_t const* ptr, size_t requested, size_t allocated, size_t alignment) {
	fprintf(
		stderr, "%c %p  [%ld => %ld @%ld]\n", prefix, (void const*)ptr,
		(unsigned long)requested, (unsigned long)allocated, (unsigned long)alignment
	);
}


/// Prints a message and aborts the process
///
/// \param message The message to print
void die(char const* message) {
	fprintf(stderr, "%s\n", message);
	abort();
}


/// Securely erase memory
///
/// \param ptr A pointer to the memory to erase
/// \param len The length of the memory to erase
void ma_proper_memzero(uint8_t* const ptr, const size_t len) {
	#if defined(USE_MEMSET_S)
		if (len != 0 && memset_s(ptr, (rsize_t)len, 0, (rsize_t)len) != 0) die("`memset_s` failed");
	#elif defined(USE_SECUREZEROMEMORY)
		SecureZeroMemory(ptr, len);
	#elif defined(USE_EXPLICIT_BZERO)
		explicit_bzero(ptr, len);
	#elif defined(USE_EXPLICIT_MEMSET)
		explicit_memset(ptr, 0, len);
	#elif defined(USE_VOLATILE_POINTERS)
		#warning "No secure memset implementation known; using volatile poiners"
		volatile uint8_t* volatile ptr_ = (volatile uint8_t* volatile)ptr;
		for (size_t i = 0; i < len; i++) ptr_[i] = 0x00;
	#else
		#error "No secure memset implementation specified (this is a build-system error)"
	#endif
}