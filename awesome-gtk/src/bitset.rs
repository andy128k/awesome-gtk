pub trait BitSetIterExt {
    fn iter_asc<'bitset>(&'bitset self) -> BitsetForwardIterator<'bitset>;
    fn iter_desc<'bitset>(&'bitset self) -> BitsetBackwardIterator<'bitset>;
}

pub struct BitsetForwardIterator<'bitset> {
    inner: Option<gtk::BitsetIter<'bitset>>,
}

impl<'bitset> Iterator for BitsetForwardIterator<'bitset> {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        let inner = self.inner.as_mut()?;
        if inner.is_valid() {
            let value = inner.value();
            let _ignore = inner.next();
            Some(value)
        } else {
            None
        }
    }
}

pub struct BitsetBackwardIterator<'bitset> {
    inner: Option<gtk::BitsetIter<'bitset>>,
}

impl<'bitset> Iterator for BitsetBackwardIterator<'bitset> {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        let inner = self.inner.as_mut()?;
        if inner.is_valid() {
            let value = inner.value();
            let _ignore = inner.previous();
            Some(value)
        } else {
            None
        }
    }
}

impl BitSetIterExt for gtk::Bitset {
    fn iter_asc<'bitset>(&'bitset self) -> BitsetForwardIterator<'bitset> {
        match gtk::BitsetIter::init_first(self) {
            Some((iter, _)) => BitsetForwardIterator { inner: Some(iter) },
            None => BitsetForwardIterator { inner: None },
        }
    }

    fn iter_desc<'bitset>(&'bitset self) -> BitsetBackwardIterator<'bitset> {
        match gtk::BitsetIter::init_last(self) {
            Some((iter, _)) => BitsetBackwardIterator { inner: Some(iter) },
            None => BitsetBackwardIterator { inner: None },
        }
    }
}
