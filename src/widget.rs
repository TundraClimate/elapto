use crate::{
    component::{Component, KeyGen},
    property::Property,
};

pub trait Widget {
    type Prop: Default;

    fn make_widget(prop: Property<Self::Prop>) -> Self;

    fn with_children<T: Widget + KeyGen>(self, _child: Component<T>) -> Self
    where
        Self: Sized,
    {
        self
    }
}
