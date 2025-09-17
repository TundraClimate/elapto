#[macro_export]
macro_rules! mk {
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
        #[allow(unused_braces)]
        $crate::make_component::<$tag, _>(|_p| { $($(_p.$pk = $pv);*)? })
            $($(.with_children(make_component!($($inner),+)))*)?
    }};
}

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

        fn render(&self) -> crate::Component {
            unimplemented!()
        }
    }

    let component1 = mk!(<Paragraph, { text={"Hi, World!".to_string()}, id="Id" }>);
    let component2 = mk!(<Paragraph>);

    assert_eq!(component1, component2);
}
