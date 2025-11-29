use elapto::{Component, Widget, mk, widget};

#[widget]
#[derive(Default, Hash)]
struct Foo {
    name: &'static str,
    number: usize,
    sick: bool,
}

impl Widget for Foo {
    fn render(&self, children: Vec<Component>) -> Component {
        mk!(
            <>
                "Foo: " { children }
            </>
        )
    }
}

fn main() {
    let tag =
        mk!(<Foo name="John" number={ 12 + 8 } sick>"Hello" { [mk!(""), mk!(",")] } "World"</Foo>);

    println!("{:?}", tag);
}
