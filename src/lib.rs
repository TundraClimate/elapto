#![warn(missing_docs)]
#![allow(unused)]

//! The layer based TUI rendering library.
//!
//! **THIS CRATE IS CURRENTLY BETA VERSION**  
//! README and this docs.rs is BETA ver, Information will update always.  

mod macros;
mod style;
mod tui;

use crossterm::execute;
use std::any;
use std::fmt::Debug;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;
use style::StyleSheet;
use tui::{Restore, TuiInitialize};

/// An identifier used to distinguish between the same Widget.
pub type Identity = &'static str;

/// A class used to specify the style.
pub type Class = &'static str;

/// A trait that defines the required values in [Widget].
///
pub trait WidgetProp: Default {
    /// An identifier used to distinguish between the same Widget.
    fn id(&self) -> Identity;

    /// A class used to specify the style.
    fn class(&self) -> Class;
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
    /// ```ignore
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
    /// ```ignore
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
/// ```ignore
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
/// ```ignore
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
/// ```ignore
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
        v: String
    }
}

#[derive(Hash)]
struct Text {
    text: String,
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

struct EngineBuilder {
    initialize: Option<TuiInitialize>,
    restore: Option<Restore>,
    render_tick: Duration,
    style: StyleSheet,
}

impl Default for EngineBuilder {
    fn default() -> Self {
        Self {
            initialize: None,
            restore: None,
            render_tick: Duration::from_millis(60),
            style: StyleSheet::new(),
        }
    }
}

impl EngineBuilder {
    fn new() -> Self {
        Self::default()
    }

    fn set_initialize(mut self, init: TuiInitialize) -> Self {
        self.initialize = Some(init);

        self
    }

    fn set_restore(mut self, restore: Restore) -> Self {
        self.restore = Some(restore);

        self
    }

    fn set_tick(mut self, tick_ms: u64) -> Self {
        self.render_tick = Duration::from_millis(tick_ms);

        self
    }

    fn set_style(mut self, stylesheet: StyleSheet) -> Self {
        self.style = stylesheet;

        self
    }

    fn build<R: Widget + Send + Sync + 'static>(self) -> Engine<R> {
        Engine::<R>::new(self.initialize, self.restore, self.render_tick, self.style)
    }
}

struct Engine<R: Widget> {
    active_state: Arc<AtomicBool>,
    initialize: Option<TuiInitialize>,
    restore: Option<Restore>,
    previous_root: RwLock<Component>,
    render_tick: Duration,
    style: StyleSheet,
    phantom: PhantomData<R>,
}

impl<R: Widget + Sync + Send + 'static> Engine<R> {
    fn new(
        initialize: Option<TuiInitialize>,
        restore: Option<Restore>,
        render_tick: Duration,
        style: StyleSheet,
    ) -> Self {
        Self {
            active_state: Arc::new(AtomicBool::new(true)),
            initialize,
            restore,
            previous_root: RwLock::new(mk!(<R>)),
            render_tick,
            style,
            phantom: PhantomData,
        }
    }

    fn render_start(&self) -> io::Result<()> {
        if let Some(ref initialize) = self.initialize {
            execute!(io::stdout(), initialize)?;
        }

        if self.active_state.load(Ordering::SeqCst) {
            return Ok(());
        }

        let state = self.active_state.clone();

        thread::spawn(move || while state.load(Ordering::SeqCst) {});

        Ok(())
    }

    fn render_end(&self) -> io::Result<()> {
        self.active_state.store(false, Ordering::SeqCst);

        match self.restore {
            Some(ref restore) => execute!(io::stdout(), restore),
            None => Ok(()),
        }
    }

    fn render(&self, component: Component) {
        unimplemented!()
    }
}

#[test]
fn test() {
    use std::time::Duration;
    use style::Style;

    #[derive(Hash)]
    struct Root;

    impl Widget for Root {
        type Prop = ContainerProp;

        fn make(_prop: Self::Prop) -> Self {
            Self
        }

        fn render(&self, _children: &[Component]) -> Component {
            mk!(<Text, { v={"Hello, World!".to_string()} }>)
        }
    }

    let initialize = TuiInitialize::new()
        .enable_raw_mode()
        .enter_alternate()
        .hide_cursor()
        .disable_line_wrap();

    let sheet = Ok(StyleSheet::new())
        .and_then(|s| s.try_append(".text", Style::default()))
        .and_then(|s| s.try_append(".p", Style::default()))
        .expect("Sheet parsing failed");

    let engine = EngineBuilder::new()
        .set_tick(60)
        .set_style(sheet)
        .set_initialize(initialize)
        .set_restore(Restore::all())
        .build::<Root>();

    engine.render_start().ok();

    thread::sleep(Duration::from_millis(3000));

    engine.render_end().ok();
}
