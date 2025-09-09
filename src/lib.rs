mod component;
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
        stx_args!($($arg),*)
    };
}

macro_rules! stx_args {
    ($tag:ident $(, [])?) => {{
        format!("<{:?}>", $tag)
    }};

    ($tag:ident, [
        <$($inner:tt),+>
    ]) => {{
        format!("<{:?} [ {:?} ]>", $tag, stx_args!($($inner),+))
    }};

    (
        $tag:ident, [
            <$($first:tt),+>
            $(<$($inner:tt),+>)+
        ]
    ) => {{
        format!("<{:?} [ {:?}, {:?} ]>",
            $tag, stx_args!($($first),+), stx_args!($($($inner),+)+))
    }};

    ($tag:ident, { $($pk:ident=$pv:tt),* $(,)? } $(, [])?) => {{
        format!("<{:?} {{ {:?} }}>", $tag, prop_args!(PropSub, $($pk=$pv),*))
    }};

    ($tag:ident, { $($pk:ident=$pv:tt),* $(,)? }, [
        <$($inner:tt),+>
    ]) => {{
        format!("<{:?} {{ {:?} }} [ {:?} ]>", $tag, prop_args!(PropSub, $($pk=$pv),*), stx_args!($($inner),+))
    }};

    (
        $tag:ident, { $($pk:ident=$pv:tt),* $(,)? }, [
            <$($first:tt),+>
            $(<$($inner:tt),+>)+
        ]
    ) => {{
        format!("<{:?} {{ {:?} }} [ {:?}, {:?} ]>",
            $tag, prop_args!(PropSub, $($pk=$pv),*), stx_args!($($first),+), stx_args!($($($inner),+)+))
    }};
}

macro_rules! prop_args {
    () => {{ Prop::default() }};

    (
        $propty:ident,
        $(id=$id:expr)?
        $(, name=$name:expr)?
        $(, prop={ $($pk:ident=$pv:expr),* $(,)? })?
        $(,)?
    ) => {{
        #[allow(unused_mut)]
        let mut prop = Prop::default();

        $(prop.id = $id;)?
        $(prop.name = $name;)?
        $(
            prop.prop = $propty::default();
            $(
                prop.prop.$pk = $pv;
            )*
        )?

        prop
    }};
}

#[derive(Debug)]
struct Tag;

#[derive(Debug, PartialEq, Default)]
struct Prop {
    id: &'static str,
    name: &'static str,
    prop: PropSub,
}

#[derive(Debug, PartialEq, Default)]
struct PropSub {
    day: usize,
    tulip: &'static str,
}

#[test]
fn test() {
    let dbg = stx! {
        <Tag, { id="", name="", prop={ day=12, tulip="Biggest" } }, [
            <Tag, [
                <Tag>
                <Tag, []>
            ]>
            <Tag, [
                <Tag>
            ]>
        ]>
    };

    assert_eq!(String::from(""), dbg);
}
