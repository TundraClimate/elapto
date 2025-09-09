use crate::widget::Widget;
use std::hash::{DefaultHasher, Hash, Hasher};

pub struct Component<T: KeyGen + Widget> {
    widget: T,
}

impl<T: KeyGen + Widget> Component<T> {
    pub fn new(widget: T) -> Self {
        Self { widget }
    }
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
    T: KeyGen + Widget,
{
    fn gen_key(&self) -> u64 {
        self.widget.gen_key()
    }
}
