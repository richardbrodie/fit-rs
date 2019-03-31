use super::DefinedMessage;
use super::Message;

/// An iterator over the parsed Records
pub struct MessageIterator<'a> {
    pub i: usize,
    pub v: &'a Vec<Message>,
}
impl<'a> MessageIterator<'a> {
    pub fn filter_name(self, name: &'a str) -> FilterMessageIterator<'a, Self> {
        FilterMessageIterator { k: name, i: self }
    }
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
pub struct FilterMessageIterator<'a, MessageIterator> {
    pub k: &'a str,
    pub i: MessageIterator,
}
impl<'a, I: Iterator<Item = &'a Message>> Iterator for FilterMessageIterator<'a, I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<I::Item> {
        for x in &mut self.i {
            if x.name() == self.k {
                return Some(x);
            }
        }
        None
    }
}
