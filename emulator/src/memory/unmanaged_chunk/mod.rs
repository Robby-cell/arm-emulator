use std::ptr::NonNull;

mod index;

/// A chunk of memory that is not managed by the emulator.
/// This is just memory that is being given by the host.
/// The host allocates the memory, and the emulator has a start and end index into that memory.
#[derive(Debug)]
pub struct UnmanagedReadOnlyChunk {
    memory: NonNull<[u8]>,
}

impl UnmanagedReadOnlyChunk {
    /// Create a new unmanaged chunk.
    /// The `start` and `end` parameters are the start and end addresses of the chunk.
    /// The `start` address is inclusive, and the `end` address is exclusive.
    /// `start` must be less than `end`.
    pub fn new(memory: NonNull<[u8]>) -> Self {
        let addr = memory.as_ptr() as *const u8 as usize;
        tracing::trace!("Creating UnmanagedReadOnlyChunk: {addr:#X}",);
        Self { memory }
    }

    pub fn len(&self) -> u32 {
        self.memory.len() as _
    }
}

impl From<&[u8]> for UnmanagedReadOnlyChunk {
    fn from(value: &[u8]) -> Self {
        let memory = NonNull::from(value);
        Self::new(memory)
    }
}

#[derive(Debug)]
pub struct UnmanagedReadWriteChunk {
    memory: NonNull<[u8]>,
}

impl UnmanagedReadWriteChunk {
    /// Create a new unmanaged chunk.
    /// The `start` and `end` parameters are the start and end addresses of the chunk.
    /// The `start` address is inclusive, and the `end` address is exclusive.
    /// `start` must be less than `end`.
    pub fn new(memory: NonNull<[u8]>) -> Self {
        let addr = memory.as_ptr() as *const u8 as usize;
        tracing::trace!("Creating UnmanagedReadWriteChunk: {addr:#X}",);
        Self { memory }
    }

    pub fn len(&self) -> u32 {
        self.memory.len() as _
    }
}

impl From<&mut [u8]> for UnmanagedReadOnlyChunk {
    fn from(value: &mut [u8]) -> Self {
        let memory = NonNull::from(value);
        Self::new(memory)
    }
}
