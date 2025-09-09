use std::hash::{DefaultHasher, Hash, Hasher};

pub struct Component<T: KeyGen> {
    t: T,
}

pub trait KeyGen {
    fn gen_key(&self) -> u64;
}

impl<T> KeyGen for T
where
    T: Hash,
{
    fn gen_key(&self) -> u64 {
        let mut hasher = DefaultHasher::new();

        self.hash(&mut hasher);

        hasher.finish()
    }
}

impl<T> KeyGen for Component<T>
where
    T: KeyGen,
{
    fn gen_key(&self) -> u64 {
        self.t.gen_key()
    }
}
