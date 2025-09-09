mod component;
mod property;
mod widget;

use crate::{
    component::{Component, KeyGen},
    widget::Widget,
};

fn make_component<T: KeyGen + Widget>() -> Component<T> {
    unimplemented!()
}

macro_rules! stx {
    (
        <$($arg:tt),*>
    ) => {
        make_component!($($arg),*)
    };
}

macro_rules! make_component {
    (
        $tag:ident $(, { $($pk:ident=$pv:tt),* $(,)? })? $(, [
            $(<$($inner:tt),+>)*
        ])?
    ) => {{
        use $crate::component::Component;
        use $crate::widget::Widget;

        let widget = <$tag as Widget>::make_widget(make_prop!(<$tag as Widget>::Prop, $($($pk=$pv),*)?))
            $($(.with_children(make_component!($($inner),+)))*)?;

        Component::new(widget)
    }};
}

macro_rules! make_prop {
    ($propty:ty $(,)?) => {{ $crate::property::Property::default() }};

    (
        $propty:ty
        $(, prop={ $($pk:ident=$pv:expr),* $(,)? })?
        $(,)?
    ) => {{
        #[allow(unused_mut)]
        let mut prop = $crate::property::Property::default();

        $(
            prop.prop = <$propty>::default();
            $(
                prop.prop.$pk = $pv;
            )*
        )?

        prop
    }};
}

#[test]
fn test() {
    #[derive(Debug, Hash)]
    struct Tag;

    #[derive(Debug, PartialEq, Default)]
    struct Prop {
        day: usize,
        tulip: &'static str,
    }

    impl Widget for Tag {
        type Prop = Prop;

        fn make_widget(_prop: property::Property<Self::Prop>) -> Self {
            Self {}
        }
    }

    let dbg = stx! {
        <Tag, { prop={ day=12, tulip="Biggest" } }, [
            <Tag, [
                <Tag>
                <Tag, []>
            ]>
            <Tag, [
                <Tag>
            ]>
        ]>
    };

    /* assert_eq!(String::from(""), dbg); */
}
