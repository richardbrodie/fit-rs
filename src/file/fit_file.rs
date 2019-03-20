use super::file_header::FileHeader;
use crate::DefinedMessage;
use std::collections::HashMap;

/// An iterator over the parsed Records
pub struct MessageIterator<'a> {
    i: usize,
    v: &'a Vec<Box<dyn DefinedMessage>>,
}
impl<'a> MessageIterator<'a> {
    pub fn filter_name(self, name: &'a str) -> FilterMessageIterator<'a, Self> {
        FilterMessageIterator { k: name, i: self }
    }
}
impl<'a> Iterator for MessageIterator<'a> {
    type Item = &'a Box<dyn DefinedMessage>;

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
    k: &'a str,
    i: MessageIterator,
}
impl<'a, I: Iterator<Item = &'a Box<dyn DefinedMessage>>> Iterator
    for FilterMessageIterator<'a, I>
{
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
/// A wrapper around the sequence of Records parsed
pub struct FitFile {
    pub file_header: FileHeader,
    pub records: Vec<Box<dyn DefinedMessage>>,
}
impl FitFile {
    /// Return a summary of parsed messages
    ///
    pub fn message_counts(&self) -> HashMap<&str, u32> {
        self.records.iter().fold(HashMap::new(), |mut acc, x| {
            let c = acc.entry(x.name()).or_insert(0);
            *c += 1;
            acc
        })
    }

    /// Return an iterator over the parsed messages
    ///
    pub fn messages<'a>(&'a self) -> MessageIterator<'a> {
        MessageIterator {
            i: 0,
            v: &self.records,
        }
    }
}
