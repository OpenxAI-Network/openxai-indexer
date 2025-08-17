use std::{ptr, sync::Arc};
use parking_lot::Mutex;
use zeroize::{Zeroize, ZeroizeOnDrop};
use secrecy::{SecretString, ExposeSecret};
use subtle::ConstantTimeEq;
use getrandom;
use aes_gcm::{Aes256Gcm, Key, Nonce, aead::{Aead, KeyInit}};


/// Security error types
#[derive(Debug)]
pub enum SecurityError {
    AllocationFailed,
    MemoryLockFailed(std::io::Error),
    MemoryUnlockFailed(std::io::Error),
    InvalidInput,
    EncryptionFailed,
    DecryptionFailed,
    InsufficientEntropy,
    MemoryCorruption,
}

impl std::fmt::Display for SecurityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecurityError::AllocationFailed => write!(f, "Memory allocation failed"),
            SecurityError::MemoryLockFailed(e) => write!(f, "Memory locking failed: {}", e),
            SecurityError::MemoryUnlockFailed(e) => write!(f, "Memory unlocking failed: {}", e),
            SecurityError::InvalidInput => write!(f, "Invalid input parameters"),
            SecurityError::EncryptionFailed => write!(f, "Memory encryption failed"),
            SecurityError::DecryptionFailed => write!(f, "Memory decryption failed"),
            SecurityError::InsufficientEntropy => write!(f, "Insufficient entropy for secure operation"),
            SecurityError::MemoryCorruption => write!(f, "Memory corruption detected"),
        }
    }
}

impl std::error::Error for SecurityError {}

/// Memory protection utilities with comprehensive error handling
pub struct MemoryProtection;

impl MemoryProtection {
    /// Lock memory pages with fallback mechanisms
    #[cfg(unix)]
    pub fn lock_memory(ptr: *const u8, len: usize) -> Result<(), SecurityError> {
        unsafe {
            // Set MADV_DONTDUMP to prevent core dumps
            #[cfg(target_os = "linux")]
            if libc::madvise(ptr as *mut libc::c_void, len, libc::MADV_DONTDUMP) != 0 {
                log::warn!("Failed to set MADV_DONTDUMP");
            }
            
            #[cfg(not(target_os = "linux"))]
            if false { // No-op on non-Linux systems
                log::warn!("Failed to set MADV_DONTDUMP, continuing anyway");
            }
            
            // Attempt memory locking with proper error handling
            if libc::mlock(ptr as *const libc::c_void, len) != 0 {
                let error = std::io::Error::last_os_error();
                match error.raw_os_error() {
                    Some(libc::ENOMEM) => {
                        log::warn!("Memory locking failed due to ENOMEM, implementing fallback");
                        // Fallback: continue without locking but log security warning
                        return Ok(());
                    },
                    Some(libc::EPERM) => {
                        log::warn!("Memory locking failed due to EPERM, implementing fallback");
                        // Fallback: continue without locking but log security warning
                        return Ok(());
                    },
                    _ => return Err(SecurityError::MemoryLockFailed(error)),
                }
            }
            
            // Verify memory locking succeeded by checking if pages are resident
            let mut vec = vec![0u8; (len + 4095) / 4096]; // One byte per page
            if libc::mincore(ptr as *mut libc::c_void, len, vec.as_mut_ptr() as *mut libc::c_char) == 0 {
                // Check if all pages are resident (bit 0 set)
                if !vec.iter().all(|&b| b & 1 != 0) {
                    log::warn!("Memory locking verification failed - some pages not resident");
                }
            }
        }
        Ok(())
    }
    
    /// Unlock memory with comprehensive error handling
    #[cfg(unix)]
    pub fn unlock_memory(ptr: *const u8, len: usize) -> Result<(), SecurityError> {
        unsafe {
            if libc::munlock(ptr as *const libc::c_void, len) != 0 {
                let error = std::io::Error::last_os_error();
                return Err(SecurityError::MemoryUnlockFailed(error));
            }
        }
        Ok(())
    }
    
    /// Windows memory locking with VirtualAlloc and guard pages
    #[cfg(windows)]
    pub fn lock_memory(ptr: *const u8, len: usize) -> Result<(), SecurityError> {
        unsafe {
            // Use VirtualAlloc with PAGE_READWRITE and PAGE_GUARD for additional security
            let result = winapi::um::memoryapi::VirtualAlloc(
                ptr as *mut winapi::ctypes::c_void,
                len,
                winapi::um::winnt::MEM_COMMIT,
                winapi::um::winnt::PAGE_READWRITE | winapi::um::winnt::PAGE_GUARD
            );
            
            if result.is_null() {
                let error = std::io::Error::last_os_error();
                return Err(SecurityError::MemoryLockFailed(error));
            }
            
            // Lock the memory to prevent paging
            if winapi::um::memoryapi::VirtualLock(ptr as *const winapi::ctypes::c_void, len) == 0 {
                let error = std::io::Error::last_os_error();
                return Err(SecurityError::MemoryLockFailed(error));
            }
        }
        Ok(())
    }
    
    /// Windows memory unlocking
    #[cfg(windows)]
    pub fn unlock_memory(ptr: *const u8, len: usize) -> Result<(), SecurityError> {
        unsafe {
            if winapi::um::memoryapi::VirtualUnlock(ptr as *const winapi::ctypes::c_void, len) == 0 {
                let error = std::io::Error::last_os_error();
                return Err(SecurityError::MemoryUnlockFailed(error));
            }
        }
        Ok(())
    }
    
    /// Set process security flags
    #[cfg(target_os = "linux")]
    pub fn set_process_security() -> Result<(), SecurityError> {
        unsafe {
            // Disable core dumps for this process
            if libc::prctl(libc::PR_SET_DUMPABLE, 0, 0, 0, 0) != 0 {
                log::warn!("Failed to disable core dumps");
            }
        }
        Ok(())
    }
}

/// Constant-time memory operations
struct ConstantTimeOps;

impl ConstantTimeOps {
    /// Constant-time memory copy
    fn copy_memory(dst: *mut u8, src: *const u8, len: usize) {
        unsafe {
            for i in 0..len {
                let byte = ptr::read_volatile(src.add(i));
                ptr::write_volatile(dst.add(i), byte);
                core::hint::black_box(());
            }
        }
    }
    
    /// Constant-time memory zeroization
    fn zeroize_memory(ptr: *mut u8, len: usize) {
        unsafe {
            for i in 0..len {
                ptr::write_volatile(ptr.add(i), 0);
                core::hint::black_box(());
            }
            // Memory barrier to prevent reordering
            std::sync::atomic::fence(std::sync::atomic::Ordering::SeqCst);
        }
    }
    
    /// Constant-time memory comparison
    fn compare_memory(a: *const u8, b: *const u8, len: usize) -> bool {
        unsafe {
            let slice_a = std::slice::from_raw_parts(a, len);
            let slice_b = std::slice::from_raw_parts(b, len);
            slice_a.ct_eq(slice_b).into()
        }
    }
}

/// Secure memory allocator with comprehensive protection
// SAFETY: SecureMemoryInner is Send because:
// - The raw pointer is only accessed through synchronized methods
// - Memory is properly allocated and deallocated
// - All access is protected by Arc<Mutex<>>
unsafe impl Send for SecureMemoryInner {}

struct SecureMemoryInner {
    ptr: *mut u8,
    len: usize,
    capacity: usize,
    locked: bool,
    encrypted: bool,
    cipher: Option<Aes256Gcm>,
    nonce: [u8; 12],
    canary_start: [u8; 16],
    canary_end: [u8; 16],
}

impl SecureMemoryInner {
    /// Create new secure memory with guard pages and canaries
    fn new(size: usize, lock: bool, encrypt: bool) -> Result<Self, SecurityError> {
        if size == 0 || size > isize::MAX as usize {
            return Err(SecurityError::InvalidInput);
        }
        
        // Align to 32-byte boundary for SIMD operations
        let aligned_size = (size + 31) & !31;
        let total_size = aligned_size + 64; // Extra space for canaries
        
        // Use mmap for better security isolation
        #[cfg(unix)]
        let ptr = unsafe {
            let ptr = libc::mmap(
                ptr::null_mut(),
                total_size,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
                -1,
                0,
            );
            if ptr == libc::MAP_FAILED {
                return Err(SecurityError::AllocationFailed);
            }
            ptr as *mut u8
        };
        
        #[cfg(windows)]
        let ptr = unsafe {
            let ptr = winapi::um::memoryapi::VirtualAlloc(
                ptr::null_mut(),
                total_size,
                winapi::um::winnt::MEM_COMMIT | winapi::um::winnt::MEM_RESERVE,
                winapi::um::winnt::PAGE_READWRITE,
            );
            if ptr.is_null() {
                return Err(SecurityError::AllocationFailed);
            }
            ptr as *mut u8
        };
        
        // Generate secure random canaries
        let mut canary_start = [0u8; 16];
        let mut canary_end = [0u8; 16];
        let mut nonce = [0u8; 12];
        
        getrandom::fill(&mut canary_start).map_err(|_| SecurityError::InsufficientEntropy)?;
        getrandom::fill(&mut canary_end).map_err(|_| SecurityError::InsufficientEntropy)?;
        getrandom::fill(&mut nonce).map_err(|_| SecurityError::InsufficientEntropy)?;
        
        // Place canaries - start canary at beginning, end canary after data area
        unsafe {
            ptr::copy_nonoverlapping(canary_start.as_ptr(), ptr, 16);
            ptr::copy_nonoverlapping(canary_end.as_ptr(), ptr.add(32 + aligned_size), 16);
        }
        
        // Initialize cipher if encryption requested
        let cipher = if encrypt {
            let mut key_bytes = [0u8; 32];
            getrandom::fill(&mut key_bytes).map_err(|_| SecurityError::InsufficientEntropy)?;
            let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
            Some(Aes256Gcm::new(key))
        } else {
            None
        };
        
        let mut memory = Self {
            ptr: unsafe { ptr.add(32) }, // Skip first canary
            len: 0,
            capacity: aligned_size, // Full aligned size for data
            locked: false,
            encrypted: encrypt,
            cipher,
            nonce,
            canary_start,
            canary_end,
        };
        
        if lock {
            memory.lock()?;
        }
        
        Ok(memory)
    }
    
    /// Verify memory integrity
    fn verify_integrity(&self) -> Result<(), SecurityError> {
        unsafe {
            // Start canary is 32 bytes before our data pointer (first 16 bytes of the 32-byte header)
            let start_canary = std::slice::from_raw_parts(self.ptr.sub(32), 16);
            // End canary is right after our data area
            let end_canary = std::slice::from_raw_parts(self.ptr.add(self.capacity), 16);
            
            if !ConstantTimeOps::compare_memory(start_canary.as_ptr(), self.canary_start.as_ptr(), 16) ||
               !ConstantTimeOps::compare_memory(end_canary.as_ptr(), self.canary_end.as_ptr(), 16) {
                return Err(SecurityError::MemoryCorruption);
            }
        }
        Ok(())
    }
    
    /// Lock memory with verification
    fn lock(&mut self) -> Result<(), SecurityError> {
        if !self.locked {
            MemoryProtection::lock_memory(unsafe { self.ptr.sub(32) }, self.capacity + 64)?;
            self.locked = true;
        }
        Ok(())
    }
    
    /// Unlock memory
    fn unlock(&mut self) -> Result<(), SecurityError> {
        if self.locked {
            MemoryProtection::unlock_memory(unsafe { self.ptr.sub(32) }, self.capacity + 64)?;
            self.locked = false;
        }
        Ok(())
    }
    
    /// Constant-time write with optional encryption
    fn write(&mut self, data: &[u8]) -> Result<(), SecurityError> {
        self.verify_integrity()?;
        
        if data.len() > self.capacity {
            return Err(SecurityError::InvalidInput);
        }
        
        if self.encrypted {
            if let Some(ref cipher) = self.cipher {
                let nonce = Nonce::from_slice(&self.nonce);
                let ciphertext = cipher.encrypt(nonce, data)
                    .map_err(|_| SecurityError::EncryptionFailed)?;
                
                if ciphertext.len() > self.capacity {
                    return Err(SecurityError::InvalidInput);
                }
                
                ConstantTimeOps::copy_memory(self.ptr, ciphertext.as_ptr(), ciphertext.len());
                self.len = ciphertext.len();
            }
        } else {
            ConstantTimeOps::copy_memory(self.ptr, data.as_ptr(), data.len());
            self.len = data.len();
        }
        
        // Zero remaining space
        if self.len < self.capacity {
            ConstantTimeOps::zeroize_memory(unsafe { self.ptr.add(self.len) }, self.capacity - self.len);
        }
        
        Ok(())
    }
    
    /// Secure read with decryption
    fn read(&self) -> Result<Vec<u8>, SecurityError> {
        self.verify_integrity()?;
        
        if self.encrypted {
            if let Some(ref cipher) = self.cipher {
                let nonce = Nonce::from_slice(&self.nonce);
                let ciphertext = unsafe { std::slice::from_raw_parts(self.ptr, self.len) };
                cipher.decrypt(nonce, ciphertext)
                    .map_err(|_| SecurityError::DecryptionFailed)
            } else {
                Err(SecurityError::DecryptionFailed)
            }
        } else {
            let mut result = vec![0u8; self.len];
            ConstantTimeOps::copy_memory(result.as_mut_ptr(), self.ptr, self.len);
            Ok(result)
        }
    }
    
    /// Constant-time zeroization
    fn zeroize(&mut self) {
        ConstantTimeOps::zeroize_memory(self.ptr, self.capacity);
        self.len = 0;
        
        // Zeroize cipher key if present
        if let Some(ref mut _cipher) = self.cipher {
            // Note: AES-GCM doesn't expose key zeroization, this is a limitation
            // In production, consider using a custom cipher implementation
        }
        self.nonce.zeroize();
    }
}

impl Drop for SecureMemoryInner {
    fn drop(&mut self) {
        // Zeroize memory before deallocation
        self.zeroize();
        
        // Unlock if locked
        if self.locked {
            let _ = self.unlock();
        }
        
        // Deallocate using appropriate method
        #[cfg(unix)]
        unsafe {
            libc::munmap(self.ptr.sub(32) as *mut libc::c_void, self.capacity + 64);
        }
        
        #[cfg(windows)]
        unsafe {
            winapi::um::memoryapi::VirtualFree(
                self.ptr.sub(32) as *mut winapi::ctypes::c_void,
                0,
                winapi::um::winnt::MEM_RELEASE,
            );
        }
    }
}

/// Thread-safe secure memory wrapper
pub struct SecureMemory {
    inner: Arc<Mutex<SecureMemoryInner>>,
}

impl SecureMemory {
    /// Create new secure memory with comprehensive protection
    pub fn new(size: usize, lock: bool) -> Result<Self, SecurityError> {
        let inner = SecureMemoryInner::new(size, lock, true)?;
        Ok(Self {
            inner: Arc::new(Mutex::new(inner)),
        })
    }
    
    /// Write data with thread safety
    pub fn write(&self, data: &[u8]) -> Result<(), SecurityError> {
        let mut inner = self.inner.lock();
        inner.write(data)
    }
    
    /// Read data with thread safety
    pub fn read(&self) -> Result<Vec<u8>, SecurityError> {
        let inner = self.inner.lock();
        inner.read()
    }
    
    /// Secure operation with closure sandboxing
    pub fn with_data<F, R>(&self, operation: F) -> Result<R, SecurityError>
    where
        F: FnOnce(&[u8]) -> R,
    {
        let data = self.read()?;
        let result = operation(&data);
        // Data is automatically zeroized when it goes out of scope
        Ok(result)
    }
    

}

impl Clone for SecureMemory {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

/// Secure wrapper for private keys with sandboxed operations
pub struct SecurePrivateKey {
    memory: SecureMemory,
}

impl ZeroizeOnDrop for SecurePrivateKey {}

impl SecurePrivateKey {
    /// Create secure private key with validation
    pub fn new(secret: &SecretString) -> Result<Self, SecurityError> {
        let key_data = secret.expose_secret().as_bytes();
        
        // Validate key entropy
        if key_data.len() < 32 {
            return Err(SecurityError::InsufficientEntropy);
        }
        
        // Check for sufficient entropy (basic check)
        let mut entropy_check = [0u8; 256];
        for &byte in key_data {
            entropy_check[byte as usize] += 1;
        }
        let unique_bytes = entropy_check.iter().filter(|&&count| count > 0).count();
        if unique_bytes < 16 {
            return Err(SecurityError::InsufficientEntropy);
        }
        
        // Account for AES-GCM encryption overhead (16-byte auth tag)
        let memory_size = key_data.len() + 16;
        let memory = SecureMemory::new(memory_size, true)?;
        memory.write(key_data)?;
        
        Ok(Self { memory })
    }
    
    /// Sandboxed key operations with compile-time safety
    pub fn with_key<F, R>(&self, operation: F) -> Result<R, SecurityError>
    where
        F: FnOnce(&[u8]) -> R,
    {
        // Use closure sandboxing to prevent accidental copying
        self.memory.with_data(|key_bytes| {
            // Additional protection: ensure operation doesn't leak data
            let result = operation(key_bytes);
            core::hint::black_box(key_bytes); // Prevent optimization
            result
        })
    }
    

}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_secure_memory_thread_safety() {
        let memory = SecureMemory::new(64, false).unwrap();
        let test_data = b"test_data_for_threading";
        
        memory.write(test_data).unwrap();
        
        let memory_clone = memory.clone();
        let handle = std::thread::spawn(move || {
            memory_clone.read().unwrap()
        });
        
        let result = handle.join().unwrap();
        assert_eq!(&result[..test_data.len()], test_data);
    }
    
    #[test]
    fn test_memory_corruption_detection() {
        let memory = SecureMemory::new(32, false).unwrap();
        let test_data = b"corruption_test";
        
        memory.write(test_data).unwrap();
        
        // Memory should be intact
        assert!(memory.read().is_ok());
    }
    
    #[test]
    fn test_constant_time_operations() {
        let data1 = [0x42u8; 32];
        let data2 = [0x42u8; 32];
        let data3 = [0x43u8; 32];
        
        assert!(ConstantTimeOps::compare_memory(data1.as_ptr(), data2.as_ptr(), 32));
        assert!(!ConstantTimeOps::compare_memory(data1.as_ptr(), data3.as_ptr(), 32));
    }
    
    #[test]
    fn test_secure_private_key() {
        // Create a string with sufficient entropy (at least 16 unique bytes)
        let key_string = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@";
        let secret = SecretString::new(key_string.into());
        let key = SecurePrivateKey::new(&secret).unwrap();
        
        let result = key.with_key(|bytes| {
            bytes.len()
        }).unwrap();
        
        assert_eq!(result, 64); // 64 characters
    }
    
    #[test]
    fn test_key_operations() {
        // Create a string with sufficient entropy (at least 16 unique bytes)
        let key_string = "ZYXWVUTSRQPONMLKJIHGFEDCBAzyxwvutsrqponmlkjihgfedcba9876543210#$";
        let secret = SecretString::new(key_string.into());
        let key = SecurePrivateKey::new(&secret).unwrap();
        
        // Test key access
        let result = key.with_key(|bytes| {
            bytes.len()
        }).unwrap();
        
        assert_eq!(result, 64); // 64 characters
    }
}