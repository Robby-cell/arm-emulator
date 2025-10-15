//! Memory management for the emulator.
//! Handles the memory that is allocated,
//! and the memory that is being given around to each peripheral/process.

use std::{fmt, ops::RangeInclusive, sync::Arc};

use thiserror::Error;
pub use unmanaged_chunk::{
    UnmanagedReadOnlyChunk, UnmanagedReadWriteChunk,
};

mod unmanaged_chunk;

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
pub struct LittleEndian;

/// An abstraction over endianness for reading and writing words.
/// This is a big-endian implementation.
/// See [LittleEndian] for the little-endian version.
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
        bytes[offset as usize..].copy_from_slice(&value.to_le_bytes());
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
}

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
#[derive(Debug)]
pub struct Bus {
    /// Main system RAM, represented as a simple byte vector.
    ram: Vec<u8>,
    /// A list of peripherals and the address ranges they occupy.
    peripherals: Vec<MemoryMappedPeripheral>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Endian {
    Little,
    Big,
}

impl Bus {
    pub fn get_read_only_memory_view(&self) -> &Bytes {
        &self.ram
    }

    pub fn get_mapped_peripherals(&self) -> &[MemoryMappedPeripheral] {
        &self.peripherals
    }

    /// Create a new bus with the given RAM size in bytes.
    /// The RAM is initialized to zero.
    /// No peripherals are connected initially.
    /// See [Bus::add_peripheral] to connect peripherals.
    pub fn new(ram_size: Word) -> Self {
        Self {
            ram: vec![0; ram_size as _],
            peripherals: Vec::new(),
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
    fn read32_ram_with_reader<Reader: BasicRead>(
        &self,
        addr: Word,
    ) -> MemoryAccessResult<u32> {
        tracing::trace!("Reading RAM at address: {addr:#X}");
        if addr % 4 != 0 {
            tracing::error!(
                "Unaligned access attempt at address: {addr:#X}"
            );
            return Err(MemoryAccessError::UnalignedAccess);
        }

        if addr + 4 < self.ram.len() as _ {
            Reader::read32(&self.ram, addr)
        } else {
            Err(MemoryAccessError::InvalidReadPermission { addr })
        }
    }

    /// Read a 32-bit word from the bus with the specified reader for endianness.
    fn read32_with_reader<Reader: BasicRead>(
        &self,
        addr: Word,
    ) -> MemoryAccessResult<u32> {
        // Check if the address belongs to a peripheral
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

        // If not a peripheral, assume it's RAM
        self.read32_ram_with_reader::<Reader>(addr)
    }

    /// Read the bytes at the address specified, as it would be on a little-endian system.
    pub fn read32_le(&self, addr: Word) -> MemoryAccessResult<u32> {
        self.read32_with_reader::<LittleEndian>(addr)
    }

    /// Read the bytes at the address specified, as it would be on a big-endian system.
    pub fn read32_be(&self, addr: Word) -> MemoryAccessResult<u32> {
        self.read32_with_reader::<BigEndian>(addr)
    }

    /// Same as [Bus::write32_with_writer], except it does not check for mapped peripherals,
    /// just writes to the address.
    fn write32_ram_with_writer<Writer: BasicWrite>(
        &mut self,
        addr: Word,
        value: u32,
    ) -> MemoryAccessResult<()> {
        tracing::trace!("Writing {value:#X} to RAM at address: {addr:#X}");
        // Right now: Unaligned access is not allowed
        if addr % 4 != 0 {
            tracing::error!(
                "Unaligned write attempt at address: {addr:#X}"
            );
            return Err(MemoryAccessError::UnalignedAccess);
        }

        if addr + 4 < self.ram.len() as _ {
            Writer::write32(&mut self.ram, addr, value)
        } else {
            Err(MemoryAccessError::InvalidWritePermission { addr })
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
        // Check if the address belongs to a peripheral
        for MemoryMappedPeripheral { range, peripheral } in
            self.peripherals.iter()
        {
            if range.contains(&addr) {
                let offset = addr - range.start();

                tracing::trace!(
                    "Writing {value:#X} to peripheral mapped to {range:?} at offset: {offset:#}",
                );

                return peripheral.write32(offset, value);
            }
        }

        // If not a peripheral, assume it's RAM
        self.write32_ram_with_writer::<Writer>(addr, value)
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
    fn read_byte_ram_with_reader<Reader: BasicRead>(
        &self,
        addr: Word,
    ) -> MemoryAccessResult<u8> {
        if addr < self.ram.len() as _ {
            let offset = addr % 4;
            Reader::read_byte(
                &self.ram[(addr - offset) as usize..],
                offset,
            )
        } else {
            Err(MemoryAccessError::InvalidReadPermission { addr })
        }
    }

    fn read_byte_with_reader<Reader: BasicRead>(
        &self,
        addr: Word,
    ) -> MemoryAccessResult<u8> {
        // Check if the address belongs to a peripheral
        for MemoryMappedPeripheral { range, peripheral } in
            self.peripherals.iter()
        {
            if range.contains(&addr) {
                let offset = addr - range.start();

                tracing::trace!(
                    "Reading byte from peripheral mapped to {range:?} at offset: {offset:#X}",
                );

                // return peripheral.read(offset);
                return Ok(0);
            }
        }

        self.read_byte_ram_with_reader::<Reader>(addr)
    }

    pub fn read_byte_le(&self, addr: Word) -> MemoryAccessResult<u8> {
        self.read_byte_with_reader::<LittleEndian>(addr)
    }

    pub fn read_byte_be(&self, addr: Word) -> MemoryAccessResult<u8> {
        self.read_byte_with_reader::<BigEndian>(addr)
    }

    /// Write a single byte to the bus.
    /// This does not consider endianness, it is just a single byte.
    fn write_byte_ram_with_writer<Writer: BasicWrite>(
        &mut self,
        addr: Word,
        value: u8,
    ) -> MemoryAccessResult<()> {
        if addr < self.ram.len() as _ {
            let offset = addr % 4;
            Writer::write_byte(
                &mut self.ram[(addr - offset) as usize..],
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
        // Check if the address belongs to a peripheral
        for MemoryMappedPeripheral { range, peripheral } in
            self.peripherals.iter()
        {
            if range.contains(&addr) {
                let offset = addr - range.start();

                tracing::trace!(
                    "Writing byte {value:#X} to peripheral mapped to {range:?} at offset: {offset:#}",
                );

                // return peripheral.write(offset, value);
                return Ok(()); // The write is handled, so we are done
            }
        }

        self.write_byte_ram_with_writer::<Writer>(addr, value)
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
        Self::new(DEFAULT_MEMORY_SIZE)
    }
}

pub const fn as_bytes<T>(value: &T) -> &Bytes {
    let slice = std::slice::from_ref(value);
    unsafe {
        std::slice::from_raw_parts(
            slice.as_ptr() as *const u8,
            slice.len() * size_of::<T>(),
        )
    }
}

pub const fn as_bytes_mut<T>(value: &mut T) -> &mut Bytes {
    let slice = std::slice::from_mut(value);
    unsafe {
        std::slice::from_raw_parts_mut(
            slice.as_ptr() as *mut u8,
            slice.len() * size_of::<T>(),
        )
    }
}
