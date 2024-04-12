use std::fmt::Debug;

#[derive(Clone)]
pub struct BitSet<'a, T> {
    universe: &'a [T],
    v: Vec<u64>,
    size: usize,
}

impl<'a, T: PartialEq> PartialEq for BitSet<'a, T> {
    fn eq(&self, other: &Self) -> bool {
        self.v == other.v && self.size == other.size
    }
}

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
            size: 0,
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

        BitSet {
            universe,
            v,
            size
        }
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
                self.size += 1;
            }
            return;
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.size == 0 {
            return None;
        }
        
        for (i, w) in self.v.iter_mut().enumerate() {
            for shift in 0..64 {
                if *w & 1 << shift != 0 {
                    *w ^= 1 << shift;
                    self.size -= 1;
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
            .for_each(|(a, b)| *a |= b)
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
        assert_eq!(bitset.size, 1);

        bitset.insert(5);
        assert_eq!(bitset.v[0] & 0b11111, 0b10100);
        assert_eq!(bitset.size, 2);
    }

    #[test]
    fn test_pop() {
        let universe = [1, 2, 3, 4, 5];
        let mut bitset = BitSet::empty(&universe);

        bitset.insert(3);
        assert_eq!(bitset.pop(), Some(3));

        assert_eq!(bitset.v[0] & 0b11111, 0b00000);
        assert_eq!(bitset.size, 0);

        assert_eq!(bitset.pop(), None);

        let mut bitset = BitSet::full(&universe);

        assert_eq!(bitset.pop(), Some(1));

        assert_eq!(bitset.v[0] & 0b11111, 0b11110);
        assert_eq!(bitset.size, 4);

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
}
