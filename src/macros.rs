#[macro_export]
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
        use $crate::Component;
        use $crate::Widget;

        let widget = <$tag as Widget>::make_widget(make_prop!(<$tag as Widget>::Prop, $($($pk=$pv),*)?))
            $($(.with_children(make_component!($($inner),+)))*)?;

        Component::new(widget)
    }};
}

macro_rules! make_prop {
    ($propty:ty $(,)?) => {{ $crate::Property::default() }};

    (
        $propty:ty
        $(, prop={ $($pk:ident=$pv:expr),* $(,)? })?
        $(,)?
    ) => {{
        #[allow(unused_mut)]
        let mut prop = $crate::Property::default();

        $(
            prop.prop = <$propty>::default();
            $(
                prop.prop.$pk = $pv;
            )*
        )?

        prop
    }};
}
