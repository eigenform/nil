//! Types for defining memory layout on the host machine.

use std::ffi::CString;
use std::convert::TryInto;

extern crate libc;
use libc::{
    shm_open, shm_unlink, mmap, ftruncate,
    O_CREAT, O_RDWR, O_EXCL,
    MAP_SHARED, MAP_FIXED, MAP_FAILED,
    PROT_READ, PROT_WRITE,PROT_EXEC,
    c_void, c_char,
};


#[allow(dead_code)]
pub struct MemRegion {
    /// Pointer to this memory region.
    pub ptr: &'static mut [u8],
    /// Guest physical address associated with this memory region.
    pub addr: u32,
    /// Length of the memory region.
    pub len: usize,
    /// Associated file descriptor.
    fd: i32,
}

pub const ARENA_BASE: usize = 0x0000_1337_0000_0000;
impl MemRegion {
    /// Create a new memory region.
    pub fn new(name: &str, addr: u32, len: usize) -> Self {
        let address = ARENA_BASE + addr as usize;
        let name = CString::new(name).unwrap();
        let fd = unsafe { MemRegion::create_shm(name.as_ptr(), len) };
        let ptr = unsafe { MemRegion::mmap(fd, address, len) };
        MemRegion {
            ptr, addr, len, fd
        }
    }

    unsafe fn create_shm(name: *const c_char, len: usize) -> i32 {
        let fd = shm_open(name, O_RDWR | O_CREAT | O_EXCL, 0o600);
        if fd == 1 {
            panic!("shm_open for object {:?} failed", name);
        } else {
            shm_unlink(name);
        }
        if ftruncate(fd, len.try_into().unwrap()) < 0 {
            panic!("ftruncate() for {:?} ({:x?} bytes) failed", name, len);
        } else {
            fd
        }
    }

    unsafe fn mmap(shm_fd: i32, vaddr: usize, len: usize) -> &'static mut [u8] {
        let addr = vaddr as *mut c_void;
        let res = mmap(addr, len, 
            PROT_READ | PROT_WRITE | PROT_EXEC, 
            MAP_FIXED | MAP_SHARED, shm_fd, 0
        );
        if res == MAP_FAILED { panic!("mmap() failed {:?}", addr); }
        std::slice::from_raw_parts_mut(res as *mut u8, len)
    }

}

impl MemRegion {
    pub fn write_buf(&mut self, off: u32, buf: &[u8]) {
        let off = off as usize;
        self.ptr[off..off + buf.len()].copy_from_slice(buf);
    }
    pub fn read32(&self, off: usize) -> u32 {
        u32::from_be_bytes(self.ptr[off..off + 4].try_into().unwrap())
    }
    pub fn read16(&self, off: usize) -> u16 {
        u16::from_be_bytes(self.ptr[off..off + 2].try_into().unwrap())
    }
    pub fn read8(&self, off: usize) -> u8 {
        self.ptr[off]
    }
}
