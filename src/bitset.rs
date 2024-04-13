use std::fmt::Debug;

#[derive(Clone, Hash)]
pub struct BitSet<'a, T> {
    universe: &'a [T],
    v: Vec<u64>,
}

impl<'a, T: PartialEq> PartialEq for BitSet<'a, T> {
    fn eq(&self, other: &Self) -> bool {
        self.v == other.v
    }
}

impl<'a, T: Eq> Eq for BitSet<'a, T> {}

impl<'a, T: PartialEq + Copy + Debug> Debug for BitSet<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_set()
            .entries(self.universe.iter().filter(|&&x| self.contains(x)))
            .finish()
    }
}

impl<'a, T: PartialEq + Copy + Debug> BitSet<'a, T> {
    pub fn empty(universe: &'a [T]) -> BitSet<'a, T> {
        BitSet {
            universe,
            v: vec![0; (universe.len() / 64) + 1],
        }
    }

    pub fn full(universe: &'a [T]) -> BitSet<'a, T> {
        let size = universe.len();
        let mut v = vec![0; (size / 64) + 1];

        for (i, w) in v.iter_mut().enumerate() {
            for shift in 0..64 {
                if i * 64 + shift < size {
                    *w |= 1 << shift;
                }
            }
        }

        BitSet { universe, v }
    }

    pub fn is_empty(&self) -> bool {
        for w in self.v.iter() {
            if *w != 0 {
                return false
            }
        }

        true
    }

    pub fn contains(&self, element: T) -> bool {
        if let Some(index) = self.universe.iter().position(|&x| x == element) {
            return self.v[index / 64] & 1 << (index % 64) != 0;
        }
        false
    }

    pub fn insert(&mut self, element: T) {
        if let Some(index) = self.universe.iter().position(|&x| x == element) {
            if self.v[index / 64] & 1 << (index % 64) == 0 {
                self.v[index / 64] |= 1 << (index % 64);
            }
            return;
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        for (i, w) in self.v.iter_mut().enumerate() {
            for shift in 0..64 {
                if *w & 1 << shift != 0 {
                    *w ^= 1 << shift;
                    return Some(self.universe[i * 64 + shift]);
                }
            }
        }

        None
    }

    pub fn union(&mut self, other: &BitSet<'a, T>) {
        debug_assert_eq!(self.universe, other.universe);

        self.v
            .iter_mut()
            .zip(other.v.iter())
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
    bitset: &'a BitSet<'a, T>,
    index: usize,
}

impl<'a, T: Copy> Iterator for BitSetIterator<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.bitset.universe.len() {
            if self.bitset.v[self.index / 64] & 1 << (self.index % 64) != 0 {
                let result = Some(self.bitset.universe[self.index]);
                self.index += 1;
                return result;
            }
            self.index += 1;
        }

        None
    }
}

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
}
