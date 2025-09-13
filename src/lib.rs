mod macros;

use std::any;
use std::fmt::Debug;

trait Widget: 'static + Send + Sync {
    type Prop: Default + Send + Sync + Debug + 'static;

    fn make(prop: Self::Prop) -> Self;

    #[allow(unused_mut)]
    fn with_children(mut self, _child: Component) -> Self
    where
        Self: Sized,
    {
        self
    }

    fn render(&self) -> Component;
}

trait WidgetCore: Send + Sync {
    fn render_with(&self) -> Component;
    fn type_name(&self) -> &'static str;
}

struct WidgetWrapper<W: Widget> {
    inner: W,
}

impl<W: Widget> WidgetWrapper<W> {
    fn new(inner: W) -> Self {
        Self { inner }
    }
}

impl<W: Widget> WidgetCore for WidgetWrapper<W> {
    fn render_with(&self) -> Component {
        self.inner.render()
    }

    fn type_name(&self) -> &'static str {
        any::type_name::<W>()
    }
}

struct Component {
    widget: Box<dyn WidgetCore>,
    children: Vec<Component>,
}

impl Debug for Component {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Component")
            .field("type", &self.widget.type_name())
            .field("children", &format!("{:?}", &self.children))
            .finish()
    }
}

impl Component {
    fn new<W: Widget>(widget: W) -> Self {
        Self {
            widget: Box::new(WidgetWrapper::new(widget)),
            children: vec![],
        }
    }

    fn with_children(mut self, children: Component) -> Self {
        self.children.push(children);

        self
    }

    fn render(&self) -> Component {
        self.widget.render_with()
    }
}

fn make_component<W, F>(setup: F) -> Component
where
    W: Widget,
    F: FnOnce(&mut W::Prop),
{
    let mut prop = W::Prop::default();

    setup(&mut prop);

    Component::new(W::make(prop))
}
