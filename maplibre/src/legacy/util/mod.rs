use std::{
    hash::{DefaultHasher, Hash, Hasher},
    ops::Range,
};

pub mod constants;
pub mod geo;
pub mod i18n;
pub mod math;

pub fn hash_combine<T: Hash>(seed: &mut u64, v: &T) {
    let mut hasher = DefaultHasher::new(); // TODO previously used std::hash https://en.cppreference.com/w/cpp/utility/hash
    v.hash(&mut hasher);
    *seed ^= hasher
        .finish()
        .overflowing_add(0x9e3779b9)
        .0
        .overflowing_add((*seed << 6))
        .0
        .overflowing_add((*seed >> 2))
        .0;
}

pub fn hash<T: Hash>(args: &[T]) -> u64 {
    let mut seed = 0;

    for arg in args {
        hash_combine(&mut seed, arg);
    }
    return seed;
}

fn split_in_half(range: &Range<usize>) -> (Range<usize>, Range<usize>) {
    let mid = (range.end - range.start) / 2 + range.start;

    ((range.start..mid), (mid..range.end))
}

pub fn lower_bound<T: PartialOrd>(v: &[T], elt: &T) -> usize {
    let mut range = 0..v.len();
    while !range.is_empty() {
        let (a, b) = split_in_half(&range);
        if v[b.start] < *elt {
            range = b.start + 1..b.end;
        } else {
            range = a;
        }
    }
    range.start
}

#[cfg(test)]
mod tests {
    use crate::legacy::util::lower_bound;

    #[test]
    fn lower_bound_test() {
        let mut input = [10, 20, 30, 30, 20, 10, 10, 20];

        input.sort();

        assert_eq!(lower_bound(&input, &20), 3);
        assert_eq!(lower_bound(&input, &15), 3);
        assert_eq!(lower_bound(&input, &5), 0);
    }
}
