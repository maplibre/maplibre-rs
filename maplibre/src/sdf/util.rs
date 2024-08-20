use std::hash::{DefaultHasher, Hash, Hasher};

pub fn hash_combine<T: Hash>(seed: &mut u64, v: &T) {
    let mut hasher = DefaultHasher::new(); // TODO previously used std::hash https://en.cppreference.com/w/cpp/utility/hash
    v.hash(&mut hasher);
    *seed ^= hasher.finish() + 0x9e3779b9 + (*seed << 6) + (*seed >> 2);
}

pub fn hash<T: Hash>(args: &[T]) -> u64 {
    let mut seed = 0;

    for arg in args {
        hash_combine(&mut seed, arg);
    }
    return seed;
}
