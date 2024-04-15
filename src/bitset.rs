use std::marker::PhantomData;

#[derive(Clone, Hash)]
pub struct BitSet<T> {
    inner: Vec<u64>,
    universe_len: usize,
    mark: PhantomData<T>,
}

impl<'a, T: PartialEq> PartialEq for BitSet<T> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<T: Eq> Eq for BitSet<T> {}

impl<'a, T: PartialEq + Copy> BitSet<T> {
    pub fn empty(universe_len: usize) -> BitSet<T> {
        BitSet {
            inner: vec![0; (universe_len / 64) + 1],
            universe_len,
            mark: PhantomData,
        }
    }

    pub fn full(universe_len: usize) -> BitSet<T> {
        let mut inner = vec![0; (universe_len / 64) + 1];

        for (i, w) in inner.iter_mut().enumerate() {
            for shift in 0..64 {
                if i * 64 + shift < universe_len {
                    *w |= 1 << shift;
                }
            }
        }

        BitSet { universe_len, inner, mark: PhantomData }
    }

    pub fn is_empty(&self) -> bool {
        for w in self.inner.iter() {
            if *w != 0 {
                return false
            }
        }

        true
    }

    pub fn contains(&self, index: usize) -> bool {
        return self.inner[index / 64] & 1 << (index % 64) != 0;
    }

    pub fn insert(&mut self, index: usize) -> bool {
        let prev = self.inner[index / 64] & 1 << (index % 64) == 0;
        self.inner[index / 64] |= 1 << (index % 64);

        prev
    }

    pub fn pop(&mut self) -> Option<usize> {
        if self.is_empty() {
            return None;
        }

        for (i, w) in self.inner.iter_mut().enumerate() {
            for shift in 0..64 {
                if *w & 1 << shift != 0 {
                    *w ^= 1 << shift;
                    return Some(i * 64 + shift);
                }
            }
        }

        None
    }

    pub fn union(&mut self, other: &BitSet<T>) {
        self.inner
            .iter_mut()
            .zip(other.inner.iter())
            .for_each(|(a, b)| *a |= b);
    }

    pub fn iter(&self) -> BitSetIterator<T> {
        BitSetIterator {
            bitset: self,
            index: 0,
        }
    }
}

pub struct BitSetIterator<'a, T> {
    bitset: &'a BitSet<T>,
    index: usize,
}

impl<'a, T: Copy> Iterator for BitSetIterator<'a, T> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.bitset.universe_len {
            if self.bitset.inner[self.index / 64] & 1 << (self.index % 64) != 0 {
                let result = Some(self.index);
                self.index += 1;
                return result;
            }
            self.index += 1;
        }

        None
    }
}
/* 
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert() {
        let universe = [1, 2, 3, 4, 5];
        let mut bitset = BitSet::empty(&universe);

        bitset.insert(3);

        assert_eq!(bitset.v[0] & 0b11111, 0b00100);

        bitset.insert(5);
        assert_eq!(bitset.v[0] & 0b11111, 0b10100);
    }

    #[test]
    fn test_pop() {
        let universe = [1, 2, 3, 4, 5];
        let mut bitset = BitSet::empty(&universe);

        bitset.insert(3);
        assert_eq!(bitset.pop(), Some(3));

        assert_eq!(bitset.v[0] & 0b11111, 0b00000);
        assert!(bitset.is_empty());

        assert_eq!(bitset.pop(), None);

        let mut bitset = BitSet::full(&universe);

        assert_eq!(bitset.pop(), Some(1));

        assert_eq!(bitset.v[0] & 0b11111, 0b11110);

        bitset.pop();
        bitset.pop();
        bitset.pop();
        bitset.pop();
        assert_eq!(bitset.pop(), None);
    }

    #[test]
    fn test_union() {
        let universe1 = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let mut bitset1 = BitSet::empty(&universe1);

        // bitset1.union(&bitset2);

        let expected_v = vec![0; (universe1.len() / 64) + 1]; // Update this with the expected result
        assert_eq!(bitset1.v, expected_v);
    }

    #[test]
    fn test_iterator() {
        let universe = [1, 2, 3, 4, 5];
        let mut bitset = BitSet::empty(&universe);

        bitset.insert(3);
        bitset.insert(5);

        let iter = bitset.iter();
        let mut elements: Vec<_> = iter.collect();
        elements.sort();

        assert_eq!(elements, vec![3, 5]);
    }
} */
