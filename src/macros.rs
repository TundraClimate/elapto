#[macro_export]
/// A macro for creating Widgets using tag syntax.
///
macro_rules! mk {
    (
        <$($arg:tt),*>
    ) => {
        make_component!($($arg),*)
    };
}

#[macro_export]
macro_rules! make_component {
    (
        _ $(, { $($pk:ident=$pv:tt),* $(,)? })? $(, [
            $(<$($inner:tt),+>)*
        ])?
    ) => {{
        #[allow(unused_braces)]
        $crate::make_component::<$crate::Container, _>(
            |_p| { $($(_p.$pk = $pv);*)? },
            vec![ $($(make_component!($($inner),+)),*)? ]
        )
    }};

    (
        $tag:ident $(, { $($pk:ident=$pv:tt),* $(,)? })? $(, [
            $(<$($inner:tt),+>)*
        ])?
    ) => {{
        #[allow(unused_braces)]
        $crate::make_component::<$tag, _>(
            |_p| { $($(_p.$pk = $pv);*)? },
            vec![ $($(make_component!($($inner),+)),*)? ]
        )
    }};
}

#[macro_export]
/// A macro for implement the properties
///
macro_rules! prop {
    (
        $(#[$attr:meta])?
        $v:vis struct $pname:ident {
            $($ivis:vis $inner_id:ident : $inner_ty:ty),* $(,)?
        }
    ) => {
        $(#[$attr])?
        $v struct $pname {
            $v id: $crate::Identity,
            $($ivis $inner_id: $inner_ty),*
        }

        impl $crate::WidgetProp for $pname {
            fn id(&self) -> $crate::Identity {
                self.id
            }
        }
    };
}

#[test]
fn test() {
    use crate::Widget;

    #[derive(Hash)]
    struct Paragraph {
        text: String,
    }

    prop! {
        #[derive(Default)]
        struct ParagraphProp {
            text: String,
        }
    }

    impl Widget for Paragraph {
        type Prop = ParagraphProp;

        fn make(prop: Self::Prop) -> Self {
            Self { text: prop.text }
        }

        fn render(&self, _children: &[crate::Component]) -> crate::Component {
            use crate::Text;

            mk!(<_, [
                <Text, { v={"Paragraph: ".to_string()} }>
                <Text, { v={self.text.clone()} }>
            ]>)
        }
    }

    let component1 = mk!(<Paragraph, { text={"Hi, World!".to_string()}, id="Id" }>);
    let component2 = mk!(<Paragraph, []>);
    let component3 = mk!(<Paragraph, { text={"Hi, World!".to_string()}, id="Id" }, [ <_> ]>);

    assert_ne!(component1, component2);
    assert_ne!(component1.widget.key(), component2.widget.key());

    assert_ne!(component1, component3);
    assert_eq!(component1.widget.key(), component3.widget.key());
}
