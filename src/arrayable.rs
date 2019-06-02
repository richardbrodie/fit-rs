#[derive(Clone, Copy)]
pub struct VArray<T: Arrayable> {
    length: usize,
    stack: [T; 16],
}
impl<T: Arrayable> VArray<T> {
    pub fn from_slice(v: &[T]) -> Option<Self> {
        let mut a: [T; 16] = [Default::default(); 16];
        a[..v.len()].clone_from_slice(&v[..]);
        Some(Self {
            length: v.len(),
            stack: a,
        })
    }
}
impl<T: Arrayable> std::fmt::Debug for VArray<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Array [")?;
        for i in 0..(self.length - 1) {
            write!(f, "{:?},", self.stack[i])?;
        }
        write!(f, "{:?}]", self.stack[self.length])
    }
}

pub trait Arrayable: Copy + Default + std::fmt::Debug {}
impl Arrayable for u8 {}
impl Arrayable for u16 {}

