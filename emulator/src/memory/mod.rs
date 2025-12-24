//! Memory management for the emulator.
//! Handles the memory that is allocated,
//! and the memory that is being given around to each peripheral/process.

use std::{fmt, ops::RangeInclusive, sync::Arc};

use thiserror::Error;

#[cfg(test)]
mod tests;

const KIBI: Word = 1 << 10;

/// 1 KiB
pub const KIBIBYTE: Word = 1 * KIBI;

/// 1 MiB
pub const MEBIBYTE: Word = KIBIBYTE * KIBI;

/// 1 GiB
pub const GIBIBYTE: Word = MEBIBYTE * KIBI;

// Too large. Overflow
// pub const TEBIBYTE: Word = GIBIBYTE * KIBI;

/// Give 1MiB memory by default.
/// This is for if no specific amount is given.
pub const DEFAULT_MEMORY_SIZE: Word = 1 * MEBIBYTE;

pub type Bytes = [u8];

/// System word type. On a 32-bit system, 32 bits
pub type Word = u32;

pub type Ram = Vec<u8>;

#[derive(Debug, Error, Clone)]
pub enum MemoryAccessError {
    #[error("invalid read permission ({addr:#X})")]
    InvalidReadPermission { addr: Word },

    #[error("invalid write permission ({addr:#X})")]
    InvalidWritePermission { addr: Word },

    #[error("unaligned access")]
    UnalignedAccess,

    #[error("invalid offset ({offset:#X})")]
    InvalidOffset { offset: Word },

    #[error("invalid peripheral read at offset {offset:#X}")]
    InvalidPeripheralRead { offset: Word },

    #[error("invalid peripheral write at offset {offset:#X}")]
    InvalidPeripheralWrite { offset: Word },
}

pub type MemoryAccessResult<T> = Result<T, MemoryAccessError>;

/// System word size (for ARM, 32 bits/4 bytes) amount of bytes
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(align(4))]
#[must_use]
pub struct WordBytes {
    data: [u8; 4],
}

impl WordBytes {
    pub fn new() -> Self {
        Default::default()
    }
}

impl WordBytes {
    fn read32_with_reader<Reader: BasicRead>(
        self,
    ) -> MemoryAccessResult<u32> {
        Reader::read32(&self.data, 0)
    }

    pub fn read32_be(self) -> MemoryAccessResult<u32> {
        self.read32_with_reader::<BigEndian>()
    }

    pub fn read32_le(self) -> MemoryAccessResult<u32> {
        self.read32_with_reader::<LittleEndian>()
    }

    fn write32_with_writer<Writer: BasicWrite>(
        &mut self,
        value: u32,
    ) -> MemoryAccessResult<()> {
        Writer::write32(&mut self.data, 0, value)
    }

    pub fn write32_be(&mut self, value: u32) -> MemoryAccessResult<()> {
        self.write32_with_writer::<BigEndian>(value)
    }

    pub fn write32_le(&mut self, value: u32) -> MemoryAccessResult<()> {
        self.write32_with_writer::<LittleEndian>(value)
    }
}

// Enforce that they are the same size at compile time.
const _: () = assert!(
    std::mem::size_of::<WordBytes>() == std::mem::size_of::<Word>()
);

/// An abstraction over endianness for reading and writing words.
/// This is a little-endian implementation.
/// See [BigEndian] for the big-endian version.
#[must_use]
pub struct LittleEndian;

/// An abstraction over endianness for reading and writing words.
/// This is a big-endian implementation.
/// See [LittleEndian] for the little-endian version.
#[must_use]
pub struct BigEndian;

/// A trait for reading words with a specific implementation (endianness).
pub trait BasicRead {
    /// Read a 32 bit word from the given bytes.
    /// Return the 4 bytes as a u32.
    fn read32(bytes: &Bytes, offset: Word) -> MemoryAccessResult<u32>;

    fn read_byte(bytes: &Bytes, offset: Word) -> MemoryAccessResult<u8>;
}

/// A trait for writing words with a specific implementation (endianness).
pub trait BasicWrite {
    /// Write a 4 byte word to the given bytes.
    fn write32(
        bytes: &mut Bytes,
        offset: Word,
        value: u32,
    ) -> MemoryAccessResult<()>;

    fn write_byte(
        bytes: &mut Bytes,
        offset: Word,
        value: u8,
    ) -> MemoryAccessResult<()>;
}

impl BasicRead for LittleEndian {
    fn read32(bytes: &Bytes, offset: Word) -> MemoryAccessResult<u32> {
        Ok(u32::from_le_bytes(
            bytes[offset as usize..][0..4].try_into().unwrap(),
        ))
    }

    fn read_byte(bytes: &Bytes, offset: Word) -> MemoryAccessResult<u8> {
        Ok(bytes[(3 - offset) as usize])
    }
}

impl BasicWrite for LittleEndian {
    fn write32(
        bytes: &mut Bytes,
        offset: Word,
        value: u32,
    ) -> MemoryAccessResult<()> {
        bytes[offset as usize..][0..4]
            .copy_from_slice(&value.to_le_bytes());
        Ok(())
    }

    fn write_byte(
        bytes: &mut Bytes,
        offset: Word,
        value: u8,
    ) -> MemoryAccessResult<()> {
        bytes[(3 - offset) as usize] = value;
        Ok(())
    }
}

impl BasicRead for BigEndian {
    fn read32(bytes: &Bytes, offset: Word) -> MemoryAccessResult<u32> {
        Ok(u32::from_be_bytes(
            bytes[offset as usize..][0..4].try_into().unwrap(),
        ))
    }

    fn read_byte(bytes: &Bytes, offset: Word) -> MemoryAccessResult<u8> {
        Ok(bytes[offset as usize])
    }
}

impl BasicWrite for BigEndian {
    fn write32(
        bytes: &mut Bytes,
        offset: Word,
        value: u32,
    ) -> MemoryAccessResult<()> {
        bytes[offset as usize..].copy_from_slice(&value.to_be_bytes());
        Ok(())
    }

    fn write_byte(
        bytes: &mut Bytes,
        offset: Word,
        value: u8,
    ) -> MemoryAccessResult<()> {
        bytes[offset as usize] = value;
        Ok(())
    }
}

/// A trait for any memory-mapped peripheral.
///
/// Each peripheral is responsible for handling reads and writes to its own
/// address space and simulating its internal logic and side effects.
pub trait Peripheral {
    /// Handles a read from the peripheral's memory-mapped region.
    /// `offset` is the address relative to the start of this peripheral's region.
    /// Does not handle endianness. Should be handled inside the [Bus].
    fn read32(&self, offset: u32) -> MemoryAccessResult<u32>;

    /// Handles a write to the peripheral's memory-mapped region.
    /// `offset` is the address relative to the start of this peripheral's region.
    /// Does not handle endianness. Should be handled inside the [Bus].
    fn write32(&self, offset: u32, value: u32) -> MemoryAccessResult<()>;

    fn read_byte(&self, offset: u32) -> MemoryAccessResult<u8>;

    fn write_byte(&self, offset: u32, value: u8)
    -> MemoryAccessResult<()>;

    fn reset(&self);
}

#[must_use]
pub struct MemoryMappedPeripheral {
    pub range: RangeInclusive<u32>,
    pub peripheral: Arc<dyn Peripheral + Send + Sync>,
}

impl fmt::Debug for MemoryMappedPeripheral {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.range)
    }
}

impl MemoryMappedPeripheral {
    pub fn new(
        range: RangeInclusive<u32>,
        peripheral: Arc<dyn Peripheral + Send + Sync>,
    ) -> Self {
        Self { range, peripheral }
    }
}

/// Connects the CPU to RAM and peripherals.
/// Routes reads and writes to the appropriate location.
#[must_use]
pub struct Bus {
    /// Main system RAM, represented as a simple byte vector.
    /// 0x00000000 - 0x1FFFFFFF
    code: Vec<u8>,

    /// .data, .bss, heap, stack.
    /// 0x20000000 - 0x3FFFFFFF
    sram: Vec<u8>,

    /// A list of peripherals and the address ranges they occupy.
    /// 0x40000000 - 0x5FFFFFFF
    peripherals: Vec<MemoryMappedPeripheral>,

    /// External memory and devices
    /// 0x60000000 -
    external: Vec<u8>,
}

impl fmt::Debug for Bus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Bus")
            .field("code", &self.code)
            .field("sram", &[0; 0])
            .field("peripherals", &self.peripherals)
            .field("external", &[0; 0])
            .finish()
    }
}

impl Bus {
    pub const CODE_BEGIN: u32 = 0x00000000;
    pub const CODE_END: u32 = 0x1FFFFFFF;
    pub const CODE_SIZE: u32 = Self::CODE_END - Self::CODE_BEGIN + 1;

    pub const SRAM_BEGIN: u32 = 0x20000000;
    pub const SRAM_END: u32 = 0x3FFFFFFF;
    pub const SRAM_SIZE: u32 = Self::SRAM_END - Self::SRAM_BEGIN + 1;

    pub const PERIPHERAL_BEGIN: u32 = 0x40000000;
    pub const PERIPHERAL_END: u32 = 0x5FFFFFFF;
    pub const PERIPHERAL_SIZE: u32 =
        Self::PERIPHERAL_END - Self::PERIPHERAL_BEGIN + 1;

    pub const EXTERNAL_BEGIN: u32 = 0x60000000;
    pub const EXTERNAL_END: u32 = u32::MAX;
    pub const EXTERNAL_SIZE: u32 =
        Self::EXTERNAL_END - Self::EXTERNAL_BEGIN + 1;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[must_use]
pub enum Endian {
    Little,
    Big,
}

impl Bus {
    pub fn load_code(&mut self, code: &[u8]) {
        tracing::info!("loading code...");
        self.code = code.to_vec();
    }

    pub fn load_sram(&mut self, sram: &[u8]) {
        tracing::info!("loading sram...");
        self.sram = sram.to_vec();
    }

    pub fn load_external(&mut self, external: &[u8]) {
        tracing::info!("loading external...");
        self.external = external.to_vec();
    }

    pub fn reserve_sram(&mut self, size: u32) {
        self.sram.reserve_exact(size as _);
        self.sram.resize(self.sram.capacity(), 0);
    }

    pub fn reserve_exact_sram(&mut self, size: u32) {
        if size as usize > self.sram.len() {
            self.reserve_sram(size - self.sram.len() as u32);
        }
    }
}

impl Bus {
    pub fn reset(&mut self) {
        self.code = Vec::new();
        self.sram = Vec::new();
        self.peripherals = Vec::new();
        self.external = Vec::new();
    }

    #[must_use]
    pub fn get_read_only_memory_view(&self) -> &Bytes {
        &self.code
    }

    #[must_use]
    pub fn get_read_only_memory_view_mut(&mut self) -> &mut Bytes {
        &mut self.code
    }

    #[must_use]
    pub fn get_read_write_memory_view(&self) -> &Bytes {
        &self.sram
    }

    #[must_use]
    pub fn get_read_write_memory_view_mut(&mut self) -> &mut Bytes {
        &mut self.sram
    }

    #[must_use]
    pub fn get_mapped_peripherals(&self) -> &[MemoryMappedPeripheral] {
        &self.peripherals
    }

    /// Create a new bus with the given RAM size in bytes.
    /// The RAM is initialized to zero.
    /// No peripherals are connected initially.
    /// See [Bus::add_peripheral] to connect peripherals.
    pub fn new(
        code_size: Word,
        sram_size: Word,
        external_size: Word,
    ) -> Self {
        Self {
            code: vec![0; code_size as _],
            sram: vec![0; sram_size as _],
            peripherals: Vec::new(),
            external: vec![0; external_size as _],
        }
    }

    /// Connect a peripheral to the bus at a specific address range.
    pub fn add_peripheral(
        &mut self,
        mapped_peripheral: MemoryMappedPeripheral,
    ) {
        tracing::trace!(
            "Adding peripheral at range: {:?}",
            mapped_peripheral.range
        );
        self.peripherals.push(mapped_peripheral);
    }

    /// Same as [Bus::read32_with_reader], except it does not check for mapped peripherals,
    /// just reads from the address.
    #[must_use]
    fn read32_ram_with_reader<Reader: BasicRead>(
        pool: &Bytes,
        offset: Word,
    ) -> MemoryAccessResult<u32> {
        tracing::trace!("Reading RAM at offset: {offset:#X}");
        if offset % 4 != 0 {
            tracing::error!(
                "Unaligned access attempt at offset: {offset:#X}"
            );
            return Err(MemoryAccessError::UnalignedAccess);
        }

        if offset + 4 <= pool.len() as _ {
            Reader::read32(pool, offset)
        } else {
            Err(MemoryAccessError::InvalidReadPermission { addr: offset })
        }
    }

    /// Read a 32-bit word from the bus with the specified reader for endianness.
    #[must_use]
    fn read32_with_reader<Reader: BasicRead>(
        &self,
        addr: Word,
    ) -> MemoryAccessResult<u32> {
        match addr {
            Self::CODE_BEGIN..=Self::CODE_END => {
                tracing::trace!(
                    "Reading word from code memory at address {addr:#X}"
                );
                Self::read32_ram_with_reader::<Reader>(
                    &self.code,
                    addr - Self::CODE_BEGIN,
                )
            }
            Self::SRAM_BEGIN..=Self::SRAM_END => {
                tracing::trace!(
                    "Reading word from SRAM memory at address {addr:#X}"
                );
                Self::read32_ram_with_reader::<Reader>(
                    &self.sram,
                    addr - Self::SRAM_BEGIN,
                )
            }
            Self::PERIPHERAL_BEGIN..=Self::PERIPHERAL_END => {
                tracing::trace!(
                    "Reading word from peripheral memory at address {addr:#X}"
                );
                for MemoryMappedPeripheral { range, peripheral } in
                    self.peripherals.iter()
                {
                    if range.contains(&addr) {
                        let offset = addr - range.start();

                        tracing::trace!(
                            "Reading peripheral mapped to {range:?} at offset: {offset:#X}",
                        );

                        return peripheral.read32(offset);
                    }
                }
                Err(MemoryAccessError::InvalidReadPermission { addr })
            }
            Self::EXTERNAL_BEGIN..=Self::EXTERNAL_END => {
                tracing::trace!(
                    "Reading word from external memory at address {addr:#X}"
                );
                Self::read32_ram_with_reader::<Reader>(
                    &self.external,
                    addr - Self::EXTERNAL_BEGIN,
                )
            }
        }
    }

    /// Read the bytes at the address specified, as it would be on a little-endian system.
    #[must_use]
    pub fn read32_le(&self, addr: Word) -> MemoryAccessResult<u32> {
        self.read32_with_reader::<LittleEndian>(addr)
    }

    /// Read the bytes at the address specified, as it would be on a big-endian system.
    #[must_use]
    pub fn read32_be(&self, addr: Word) -> MemoryAccessResult<u32> {
        self.read32_with_reader::<BigEndian>(addr)
    }

    /// Same as [Bus::write32_with_writer], except it does not check for mapped peripherals,
    /// just writes to the address.
    fn write32_ram_with_writer<Writer: BasicWrite>(
        pool: &mut Bytes,
        offset: Word,
        value: u32,
    ) -> MemoryAccessResult<()> {
        tracing::trace!(
            "Writing word {value:#X} to RAM at offset: {offset:#X}"
        );
        // Right now: Unaligned access is not allowed
        if offset % 4 != 0 {
            tracing::error!(
                "Unaligned write attempt at offset: {offset:#X}"
            );
            return Err(MemoryAccessError::UnalignedAccess);
        }

        if offset + 4 <= pool.len() as _ {
            Writer::write32(pool, offset, value)
        } else {
            Err(MemoryAccessError::InvalidWritePermission { addr: offset })
        }
    }

    /// Write a 32-bit word to the bus, with the specified writer for endianness.
    /// This gives a zero-cost abstraction over endianness.
    /// We can use the appropriate writer based on what the user selects.
    /// This method is private; use the exposed methods for specific endianness.
    fn write32_with_writer<Writer: BasicWrite>(
        &mut self,
        addr: Word,
        value: u32,
    ) -> MemoryAccessResult<()> {
        match addr {
            Self::CODE_BEGIN..=Self::CODE_END => {
                tracing::trace!(
                    "Writing word {value:#X} to code at address {addr:#X}"
                );
                Self::write32_ram_with_writer::<Writer>(
                    &mut self.code,
                    addr - Self::CODE_BEGIN,
                    value,
                )
            }
            Self::SRAM_BEGIN..=Self::SRAM_END => {
                tracing::trace!(
                    "Writing word {value:#X} to SRAM at address {addr:#X}"
                );
                Self::write32_ram_with_writer::<Writer>(
                    &mut self.sram,
                    addr - Self::SRAM_BEGIN,
                    value,
                )
            }
            Self::PERIPHERAL_BEGIN..=Self::PERIPHERAL_END => {
                tracing::trace!(
                    "Writing word {value:#X} to peripheral at address {addr:#X}"
                );
                for MemoryMappedPeripheral { range, peripheral } in
                    self.peripherals.iter()
                {
                    tracing::trace!(
                        "Checking peripheral mapped to {range:?}"
                    );
                    if range.contains(&addr) {
                        let offset = addr - range.start();

                        tracing::trace!(
                            "Writing word {value:#X} to peripheral mapped to {range:?} at offset: {offset:#X}",
                        );

                        return peripheral.write32(offset, value);
                    }
                }
                Err(MemoryAccessError::InvalidWritePermission { addr })
            }
            Self::EXTERNAL_BEGIN..=Self::EXTERNAL_END => {
                tracing::trace!(
                    "Writing word {value:#X} to external memory at address {addr:#X}"
                );
                Self::write32_ram_with_writer::<Writer>(
                    &mut self.external,
                    addr - Self::EXTERNAL_BEGIN,
                    value,
                )
            }
        }
    }

    /// Write the `value` to the address specified, as it would be on a little-endian system.
    pub fn write32_le(
        &mut self,
        addr: Word,
        value: u32,
    ) -> MemoryAccessResult<()> {
        self.write32_with_writer::<LittleEndian>(addr, value)
    }

    /// Write the `value` to the address specified, as it would be on a big-endian system.
    pub fn write32_be(
        &mut self,
        addr: Word,
        value: u32,
    ) -> MemoryAccessResult<()> {
        self.write32_with_writer::<BigEndian>(addr, value)
    }

    /// Read a single byte from the bus.
    /// This does not consider endianness, it is just a single byte.
    #[must_use]
    fn read_byte_ram_with_reader<Reader: BasicRead>(
        pool: &Bytes,
        addr: Word,
    ) -> MemoryAccessResult<u8> {
        if addr < pool.len() as _ {
            let offset = addr % 4;
            Reader::read_byte(&pool[(addr - offset) as usize..], offset)
        } else {
            Err(MemoryAccessError::InvalidReadPermission { addr })
        }
    }

    #[must_use]
    fn read_byte_with_reader<Reader: BasicRead>(
        &self,
        addr: Word,
    ) -> MemoryAccessResult<u8> {
        match addr {
            Self::CODE_BEGIN..=Self::CODE_END => {
                Self::read_byte_ram_with_reader::<Reader>(&self.code, addr)
            }
            Self::SRAM_BEGIN..=Self::SRAM_END => {
                Self::read_byte_ram_with_reader::<Reader>(&self.sram, addr)
            }
            Self::PERIPHERAL_BEGIN..=Self::PERIPHERAL_END => {
                for MemoryMappedPeripheral { range, peripheral } in
                    self.peripherals.iter()
                {
                    if range.contains(&addr) {
                        let offset = addr - range.start();

                        tracing::trace!(
                            "Reading byte from peripheral mapped to {range:?} at offset: {offset:#X}",
                        );

                        return peripheral.read_byte(offset);
                    }
                }
                Err(MemoryAccessError::InvalidReadPermission { addr })
            }
            Self::EXTERNAL_BEGIN..=Self::EXTERNAL_END => {
                Self::read_byte_ram_with_reader::<Reader>(
                    &self.external,
                    addr,
                )
            }
        }
    }

    #[must_use]
    pub fn read_byte_le(&self, addr: Word) -> MemoryAccessResult<u8> {
        self.read_byte_with_reader::<LittleEndian>(addr)
    }

    #[must_use]
    pub fn read_byte_be(&self, addr: Word) -> MemoryAccessResult<u8> {
        self.read_byte_with_reader::<BigEndian>(addr)
    }

    /// Write a single byte to the bus.
    /// This does not consider endianness, it is just a single byte.
    fn write_byte_ram_with_writer<Writer: BasicWrite>(
        pool: &mut Bytes,
        addr: Word,
        value: u8,
    ) -> MemoryAccessResult<()> {
        if addr < pool.len() as _ {
            let offset = addr % 4;
            Writer::write_byte(
                &mut pool[(addr - offset) as usize..],
                offset,
                value,
            )
        } else {
            Err(MemoryAccessError::InvalidWritePermission { addr })
        }
    }

    fn write_byte_with_writer<Writer: BasicWrite>(
        &mut self,
        addr: Word,
        value: u8,
    ) -> MemoryAccessResult<()> {
        match addr {
            Self::CODE_BEGIN..=Self::CODE_END => {
                Self::write_byte_ram_with_writer::<Writer>(
                    &mut self.code,
                    addr,
                    value,
                )
            }
            Self::SRAM_BEGIN..=Self::SRAM_END => {
                Self::write_byte_ram_with_writer::<Writer>(
                    &mut self.sram,
                    addr,
                    value,
                )
            }
            Self::PERIPHERAL_BEGIN..=Self::PERIPHERAL_END => {
                for MemoryMappedPeripheral { range, peripheral } in
                    self.peripherals.iter()
                {
                    if range.contains(&addr) {
                        let offset = addr - range.start();

                        tracing::trace!(
                            "Writing byte {value:#X} to peripheral mapped to {range:?} at offset: {offset:#X}",
                        );

                        return peripheral.write_byte(offset, value);
                    }
                }
                Err(MemoryAccessError::InvalidWritePermission { addr })
            }
            Self::EXTERNAL_BEGIN..=Self::EXTERNAL_END => {
                Self::write_byte_ram_with_writer::<Writer>(
                    &mut self.external,
                    addr,
                    value,
                )
            }
        }
    }

    pub fn write_byte_le(
        &mut self,
        addr: Word,
        value: u8,
    ) -> MemoryAccessResult<()> {
        self.write_byte_with_writer::<LittleEndian>(addr, value)
    }

    pub fn write_byte_be(
        &mut self,
        addr: Word,
        value: u8,
    ) -> MemoryAccessResult<()> {
        self.write_byte_with_writer::<BigEndian>(addr, value)
    }
}

impl Default for Bus {
    fn default() -> Self {
        Self::new(
            DEFAULT_MEMORY_SIZE,
            DEFAULT_MEMORY_SIZE,
            DEFAULT_MEMORY_SIZE,
        )
    }
}

impl Bus {
    pub fn with_ram_and_peripherals(
        code: Vec<u8>,
        sram: Vec<u8>,
        peripherals: Vec<MemoryMappedPeripheral>,
        external: Vec<u8>,
    ) -> Self {
        Self {
            code,
            sram,
            peripherals,
            external,
        }
    }

    pub fn with_ram(
        code: Vec<u8>,
        sram: Vec<u8>,
        external: Vec<u8>,
    ) -> Self {
        Self::with_ram_and_peripherals(
            code,
            sram,
            Default::default(),
            external,
        )
    }
}

pub const fn as_bytes<T>(value: &T) -> &Bytes {
    let slice = std::slice::from_ref(value);
    // SAFETY:
    // - `slice.as_ptr()` is derived from a valid reference `value`, so it is non-null,
    //   properly aligned, and points to initialized memory.
    // - The returned byte slice will have the same lifetime as `value`, ensuring that
    //   the memory does not get invalidated while the byte slice is in use.
    // - Casting `*const T` to `*const u8` is safe because we're only reinterpreting
    //   the raw memory as bytes without changing its provenance.
    // - `slice.len() * size_of::<T>()` correctly computes the size in bytes.
    //
    // NOTE: This function does not guarantee that the resulting byte slice has any
    // particular endianness or representation stability across compilers or platforms.
    unsafe {
        std::slice::from_raw_parts(
            slice.as_ptr() as *const u8,
            slice.len() * size_of::<T>(),
        )
    }
}

pub const fn as_bytes_mut<T>(value: &mut T) -> &mut Bytes {
    let slice = std::slice::from_mut(value);

    // SAFETY:
    // - `slice.as_mut_ptr()` comes from a valid, uniquely-owned `&mut T`, so the pointer is:
    //     - non-null,
    //     - properly aligned,
    //     - pointing to initialized memory.
    // - Because we have `&mut T`, we are guaranteed unique access to `value` for the duration
    //   of the returned `&mut [u8]`. No other references (mutable or immutable) to `value`
    //   may exist at the same time.
    // - Reinterpreting the memory of `T` as `u8` does not create aliasing violations, because
    //   raw bytes are allowed to alias any type.
    // - The computed length `slice.len() * size_of::<T>()` is correct for a slice of exactly
    //   one `T`.
    // - The returned `&mut [u8]` has the same lifetime as the input `&mut T`, ensuring the
    //   underlying memory stays valid while it is in use.
    //
    // NOTE: Mutating the returned byte slice may violate invariants of `T`. The caller must
    // ensure that any modifications leave `T` in a valid state.
    unsafe {
        std::slice::from_raw_parts_mut(
            slice.as_mut_ptr() as *mut u8,
            slice.len() * size_of::<T>(),
        )
    }
}

#[cfg(target_endian = "little")]
mod native_memory {
    #[must_use]
    pub const fn big_endian_to_native(instr: u32) -> u32 {
        u32::from_be_bytes(instr.to_le_bytes())
    }

    #[must_use]
    pub const fn little_endian_to_native(instr: u32) -> u32 {
        u32::from_le_bytes(instr.to_le_bytes())
    }

    #[must_use]
    pub const fn u32_to_native_bytes(value: u32) -> [u8; 4] {
        value.to_le_bytes()
    }

    #[must_use]
    pub const fn u32_from_native_bytes(bytes: [u8; 4]) -> u32 {
        u32::from_le_bytes(bytes)
    }
}

#[cfg(target_endian = "big")]
mod native_memory {
    #[must_use]
    pub const fn big_endian_to_native(instr: u32) -> u32 {
        u32::from_be_bytes(instr.to_be_bytes())
    }

    #[must_use]
    pub const fn little_endian_to_native(instr: u32) -> u32 {
        u32::from_le_bytes(instr.to_be_bytes())
    }

    #[must_use]
    pub const fn u32_to_native_bytes(value: u32) -> [u8; 4] {
        value.to_be_bytes()
    }

    #[must_use]
    pub const fn u32_from_native_bytes(bytes: [u8; 4]) -> u32 {
        u32::from_be_bytes(bytes)
    }
}

pub use native_memory::{
    big_endian_to_native, little_endian_to_native, u32_from_native_bytes,
    u32_to_native_bytes,
};
