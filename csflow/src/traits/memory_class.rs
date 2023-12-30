use memflow::types::Address;

/// A trait that implements basic functions for a class represented by a single pointer
pub trait MemoryClass {
    fn ptr(&self) -> Address;
    fn new(ptr: Address) -> Self;
}