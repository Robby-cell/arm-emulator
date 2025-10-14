use super::{UnmanagedReadOnlyChunk, UnmanagedReadWriteChunk};
use std::ops::{Index, IndexMut, Range, RangeFrom, RangeFull, RangeTo};

impl Index<Range<u32>> for UnmanagedReadOnlyChunk {
    type Output = [u8];

    fn index(&self, index: std::ops::Range<u32>) -> &Self::Output {
        unsafe {
            &self.memory.as_ref()[index.start as usize..index.end as usize]
        }
    }
}

impl Index<RangeFull> for UnmanagedReadOnlyChunk {
    type Output = <Self as Index<Range<u32>>>::Output;
    fn index(&self, _index: RangeFull) -> &Self::Output {
        &self[0..self.len()]
    }
}

impl Index<RangeFrom<u32>> for UnmanagedReadOnlyChunk {
    type Output = <Self as Index<Range<u32>>>::Output;
    fn index(&self, index: RangeFrom<u32>) -> &Self::Output {
        &self[index.start..self.len()]
    }
}

impl Index<RangeTo<u32>> for UnmanagedReadOnlyChunk {
    type Output = <Self as Index<Range<u32>>>::Output;
    fn index(&self, index: RangeTo<u32>) -> &Self::Output {
        &self[0..index.end]
    }
}

impl Index<Range<u32>> for UnmanagedReadWriteChunk {
    type Output = [u8];

    fn index(&self, index: std::ops::Range<u32>) -> &Self::Output {
        unsafe {
            &self.memory.as_ref()[index.start as usize..index.end as usize]
        }
    }
}

impl Index<RangeFull> for UnmanagedReadWriteChunk {
    type Output = <Self as Index<Range<u32>>>::Output;
    fn index(&self, _index: RangeFull) -> &Self::Output {
        &self[0..self.len()]
    }
}

impl Index<RangeFrom<u32>> for UnmanagedReadWriteChunk {
    type Output = <Self as Index<Range<u32>>>::Output;
    fn index(&self, index: RangeFrom<u32>) -> &Self::Output {
        &self[index.start..self.len()]
    }
}

impl Index<RangeTo<u32>> for UnmanagedReadWriteChunk {
    type Output = <Self as Index<Range<u32>>>::Output;
    fn index(&self, index: RangeTo<u32>) -> &Self::Output {
        &self[0..index.end]
    }
}

impl IndexMut<Range<u32>> for UnmanagedReadWriteChunk {
    fn index_mut(&mut self, index: Range<u32>) -> &mut Self::Output {
        unsafe {
            &mut self.memory.as_mut()
                [index.start as usize..index.end as usize]
        }
    }
}

impl IndexMut<RangeFull> for UnmanagedReadWriteChunk {
    fn index_mut(&mut self, _index: RangeFull) -> &mut Self::Output {
        let end = self.len();
        &mut self[0..end]
    }
}

impl IndexMut<RangeFrom<u32>> for UnmanagedReadWriteChunk {
    fn index_mut(&mut self, index: RangeFrom<u32>) -> &mut Self::Output {
        let end = self.len();
        &mut self[index.start..end]
    }
}

impl IndexMut<RangeTo<u32>> for UnmanagedReadWriteChunk {
    fn index_mut(&mut self, index: RangeTo<u32>) -> &mut Self::Output {
        &mut self[0..index.end]
    }
}
