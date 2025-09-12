mod macros;

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
}

struct Component {}
