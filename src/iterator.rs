use super::Message;

/// An iterator over the parsed Records
pub struct MessageIterator<'a> {
    pub i: usize,
    pub v: &'a Vec<Message>,
}
impl<'a> Iterator for MessageIterator<'a> {
    type Item = &'a Message;

    fn next(&mut self) -> Option<Self::Item> {
        match self.v.get(self.i) {
            Some(item) => {
                self.i += 1;
                Some(item)
            }
            _ => None,
        }
    }
}
impl<'a> MessageIterator<'a> {
    pub fn filter(self, kind: MessageType) -> FilterMessageIterator<'a, Self> {
        FilterMessageIterator { k: kind, i: self }
    }
}pub struct FilterMessageIterator<'a, MessageIterator> {
    pub k: MessageType,
    pub i: MessageIterator,
}
impl<'a, I: Iterator<Item = &'a Message>> Iterator for FilterMessageIterator<'a, I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<I::Item> {
        for x in &mut self.i {
            if x.kind == self.k {
                return Some(x);
            }
        }
        None
    }
}
