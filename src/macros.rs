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

#[test]
fn test() {
    use crate::Widget;

    struct Paragraph {
        text: String,
    }

    #[derive(Default, Debug)]
    struct ParagraphProp {
        text: String,
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

    let component1 = mk!(<Paragraph, { text={"Hi, World!".to_string()} }>);
    let component2 = mk!(<Paragraph>);

    assert_eq!(component1, component2);
}
