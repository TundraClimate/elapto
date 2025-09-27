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
use std::sync::Arc;

/// An identifier used to distinguish between the same Widget.
pub type Identity = &'static str;

/// A trait that defines the required values in [Widget].
///
pub trait WidgetProp: Default {
    /// An identifier used to distinguish between the same Widget.
    fn id(&self) -> Identity;
}

/// A trait for rendering unit.
///
pub trait Widget: Hash {
    /// Properties assigned to a `Widget`.
    type Prop: WidgetProp;

    /// Hashes the state of the `Widget`.
    ///
    /// The returned value is used to control rendering.  
    /// If the previously returned value matches the current one, rendering will be suppressed.
    fn key(&self) -> u64 {
        let mut hasher = DefaultHasher::new();

        self.hash(&mut hasher);

        hasher.finish()
    }

    /// Make a `Widget` from the `Self::Prop`.
    ///
    /// It is almost the same as `new()`.
    ///
    /// ## Example
    ///
    /// ```no_run
    /// struct FooWidget {
    ///     name: String,
    /// }
    ///
    /// impl Widget for FooWidget {
    ///     /* Other impls */
    ///
    ///     fn make(prop: Self::Prop) -> Self {
    ///         Self { name: prop.name }
    ///     }
    /// }
    /// ```
    fn make(prop: Self::Prop) -> Self;

    /// Represents the rendering process using a `Component`.
    ///
    /// Note that `self` is reinitialized on each render.
    ///
    /// ## Example
    ///
    /// ```no_run
    /// struct FooWidget {
    ///     name: String,
    /// }
    ///
    /// impl Widget for FooWidget {
    ///     /* Other impls */
    ///
    ///     fn render(&self, _children: &[Component]) -> Component {
    ///         // TODO: impl render
    ///     }
    /// }
    /// ```
    fn render(&self, _children: &[Component]) -> Component;
}

trait WidgetCore: Send + Sync {
    fn render_with(&self, children: &[Component]) -> Component;
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
    fn render_with(&self, children: &[Component]) -> Component {
        self.inner.render(children)
    }

    fn type_name(&self) -> &'static str {
        any::type_name::<W>()
    }

    fn key(&self) -> u64 {
        self.inner.key()
    }
}

/// A tree structure node.
///
/// A `Component` is a struct that wraps a [Widget] and represents the information required for rendering in a tree structure.  
/// For direct usage: please see [mk!] macro.
///
/// ## Example
///
/// TODO: Impl render()  
/// with `mk!` macro:
/// ```no_run
/// use elapto::mk;
/// use elapto::Widget;
/// # struct FooWidget;
///
/// impl Widget for FooWidget {
///     /* Other impls */
///
///     fn render(&self, _children: &[elapto::Component]) -> elapto::Component {
///         mk!(<>)
///     }
/// }
/// ```
///
/// with [make_component]:  
/// ```no_run
/// use elapto::mk;
/// use elapto::Widget;
/// # struct FooWidget;
///
/// impl Widget for FooWidget {
///     /* Other impls */
///
///     fn render(&self, _children: &[elapto::Component]) -> elapto::Component {
///         elapto::make_component::<, _>(|p| {
///             /* Edit property */
///         }, vec![])
///     }
/// }
/// ```
pub struct Component {
    widget: Arc<dyn WidgetCore>,
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

impl Clone for Component {
    fn clone(&self) -> Self {
        Self {
            widget: self.widget.clone(),
            children: self.children.clone(),
        }
    }
}

impl Component {
    fn new<W>(widget: W, children: Vec<Component>) -> Self
    where
        W: Widget + Send + Sync + 'static,
    {
        Self {
            widget: Arc::new(WidgetWrapper::new(widget)),
            children,
        }
    }

    fn render(&self) -> Component {
        self.widget.render_with(&self.children)
    }
}

/// Make a component while setups its properties.
///
/// Properties will be equals `Default` if not changed by `setup`.  
/// Consider using the [mk!] macro.
///
/// ## Example
///
/// TODO: Impl render()  
/// ```no_run
/// use elapto::mk;
/// use elapto::Widget;
/// # struct FooWidget;
///
/// impl Widget for FooWidget {
///     /* Other impls */
///
///     fn render(&self, _children: &[elapto::Component]) -> elapto::Component {
///         elapto::make_component::<, _>(|p| {
///             /* Edit property */
///         }, vec![])
///     }
/// }
/// ```
pub fn make_component<W, F>(setup: F, children: Vec<Component>) -> Component
where
    W: Widget + Send + Sync + 'static,
    F: FnOnce(&mut W::Prop),
{
    let mut prop = W::Prop::default();

    setup(&mut prop);

    Component::new(W::make(prop), children)
}

prop! {
    #[derive(Default)]
    struct ContainerProp {}
}

#[derive(Hash)]
struct Container {}

impl Widget for Container {
    type Prop = ContainerProp;

    fn render(&self, _children: &[Component]) -> Component {
        unreachable!()
    }

    fn make(_prop: Self::Prop) -> Self {
        Self {}
    }
}

prop! {
    #[derive(Default)]
    struct TextProp {
        v: &'static str,
    }
}

#[derive(Hash)]
struct Text {
    text: &'static str,
}

impl Widget for Text {
    type Prop = TextProp;

    fn render(&self, _children: &[Component]) -> Component {
        unreachable!()
    }

    fn make(prop: Self::Prop) -> Self {
        Self { text: prop.v }
    }
}
