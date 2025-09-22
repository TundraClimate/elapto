#![warn(missing_docs)]

//! The layer based TUI rendering library.
//!
//! **THIS CRATE IS CURRENTLY BETA VERSION**  
//! README and this docs.rs is BETA ver, Information will update always.  

mod macros;
mod tui;

use std::any;
use std::fmt::Debug;
use std::hash::{DefaultHasher, Hash, Hasher};

pub type Identity = &'static str;

pub trait WidgetProp: Default {
    fn id(&self) -> Identity;
}

pub trait Widget: Hash {
    type Prop: WidgetProp;

    #[allow(unused_mut)]
    fn with_children(mut self, _child: Component) -> Self
    where
        Self: Sized,
    {
        self
    }

    fn key(&self) -> u64 {
        let mut hasher = DefaultHasher::new();

        self.hash(&mut hasher);

        hasher.finish()
    }

    fn make(prop: Self::Prop) -> Self;

    fn render(&self) -> Component;
}

trait WidgetCore: Send + Sync {
    fn render_with(&self) -> Component;
    fn type_name(&self) -> &'static str;
    fn key(&self) -> u64;
}

struct WidgetWrapper<W: Widget> {
    inner: W,
}

impl<W: Widget> WidgetWrapper<W> {
    fn new(inner: W) -> Self {
        Self { inner }
    }
}

impl<W: Widget + Send + Sync> WidgetCore for WidgetWrapper<W> {
    fn render_with(&self) -> Component {
        self.inner.render()
    }

    fn type_name(&self) -> &'static str {
        any::type_name::<W>()
    }

    fn key(&self) -> u64 {
        self.inner.key()
    }
}

pub struct Component {
    widget: Box<dyn WidgetCore>,
    children: Vec<Component>,
}

impl Hash for Component {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.widget.key().hash(state);
        self.children.iter().for_each(|c| c.hash(state))
    }
}

impl Debug for Component {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Component")
            .field("type", &self.widget.type_name())
            .field("key", &self.widget.key())
            .field("children", &format!("{:?}", &self.children))
            .finish()
    }
}

impl PartialEq for Component {
    fn eq(&self, other: &Self) -> bool {
        self.widget.type_name() == other.widget.type_name()
            && self.widget.key() == other.widget.key()
            && self.children.len() == other.children.len()
            && self
                .children
                .iter()
                .zip(other.children.iter())
                .all(|(a, b)| a == b)
    }
}

impl Eq for Component {}

impl Component {
    fn new<W>(widget: W) -> Self
    where
        W: Widget + Send + Sync + 'static,
    {
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

pub fn make_component<W, F>(setup: F) -> Component
where
    W: Widget + Send + Sync + 'static,
    F: FnOnce(&mut W::Prop),
{
    let mut prop = W::Prop::default();

    setup(&mut prop);

    Component::new(W::make(prop))
}
