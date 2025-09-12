use std::hash::{DefaultHasher, Hash, Hasher};

#[derive(Default)]
struct Property<T> {
    pub prop: T,
}

trait Widget {
    type Prop: Default;

    fn make_widget(prop: Property<Self::Prop>) -> Self;

    fn with_children<T: Widget + KeyGen>(self, _child: Component<T>) -> Self
    where
        Self: Sized,
    {
        self
    }
}

struct Component<T: KeyGen + Widget> {
    widget: T,
}

impl<T: KeyGen + Widget> Component<T> {
    pub fn new(widget: T) -> Self {
        Self { widget }
    }
}

trait KeyGen {
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

        fn make_widget(_prop: Property<Self::Prop>) -> Self {
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
