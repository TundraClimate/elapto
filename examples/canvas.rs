use elapto::*;

#[widget(default)]
struct Foo {
    name: String,
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
