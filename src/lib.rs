#![allow(unused)]
#![warn(missing_docs, unused_imports)]

//! The layer based TUI rendering library.
//!
//! **THIS CRATE IS CURRENTLY BETA VERSION**  
//! README and this docs.rs is BETA ver, Information will update always.  

mod hash_cell;
mod style;
mod tui;

use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, RwLock};

pub use hash_cell::HashCell;

/// Create new [Component] from literals with parser.
pub use elapto_macros::mk;

/// Applies of WidgetInfo for target.
pub use elapto_macros::widget;

#[derive(Hash, Clone)]
/// An identfier of Component.
pub struct Identifier {
    /// The ID string.
    pub id: String,
}

#[derive(Hash, Clone)]
/// A class ident of Component.
pub struct Class {
    /// The class string.
    pub class: String,
}

/// A trait that implement for rendering.
pub trait Widget: WidgetInfo {
    /// An implment for rendering.
    fn render(&self, children: Vec<Component>) -> Component;

    /// Generate rendering node.
    fn to_dom(&self) -> Option<DomNode> {
        None
    }
}

/// A trait that get Widget infomation.
pub trait WidgetInfo {
    /// Get name of widget type.
    fn type_name(&self) -> &'static str;

    /// Get list of key-value properties.
    fn properties(&self) -> Vec<(&str, String)>;

    /// Generate a hash by widget.
    fn gen_hash(&self) -> HashCell;
}

#[derive(Clone, Hash)]
/// An extra properties.
pub struct SubProperties {
    id: Option<Identifier>,
    class: Option<Class>,
    children: Vec<Component>,
}

#[derive(Clone)]
/// A widget wrapper with [SubProperties].
pub struct Component {
    widget: Arc<dyn Widget>,
    sub_props: SubProperties,
}

impl SubProperties {
    /// Appends id for Self.
    pub fn with_id<T: Into<Identifier>>(mut self, id: T) -> Self {
        self.id = Some(id.into());

        self
    }

    /// Appends class for Self.
    pub fn with_class<T: Into<Class>>(mut self, class: T) -> Self {
        self.class = Some(class.into());

        self
    }

    /// Appends children for Self.
    pub fn with_children(mut self, children: Vec<Component>) -> Self {
        self.children = children;

        self
    }
}

impl Component {
    /// Create new Component with a widget.
    pub fn new<W: Widget + 'static>(widget: W) -> Self {
        Self {
            widget: Arc::new(widget),
            sub_props: SubProperties {
                id: None,
                class: None,
                children: vec![],
            },
        }
    }

    /// Appends a [SubProperties] for Self.
    pub fn with_sub_props<F: FnOnce(SubProperties) -> SubProperties>(mut self, f: F) -> Self {
        self.sub_props = f(self.sub_props);

        self
    }

    fn gen_hash(&self) -> HashCell {
        self.widget.gen_hash().combine(&self.sub_props)
    }

    fn to_dom_node(&self) -> Option<DomNode> {
        self.widget.to_dom()
    }

    fn render(&self) -> Component {
        self.widget.render(self.sub_props.children.clone())
    }
}

impl Debug for Component {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let id = match self.sub_props.id {
            Some(Identifier { ref id }) => format!(" id={id}"),
            None => "".to_string(),
        };

        let class = match self.sub_props.class {
            Some(Class { ref class }) => format!(" class={class}"),
            None => "".to_string(),
        };

        let props = self
            .widget
            .properties()
            .into_iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join(" ");

        let children = self
            .sub_props
            .children
            .iter()
            .map(|cpnt| format!("\t{cpnt:?}\n"))
            .collect::<String>();

        let (children, close) = if children.is_empty() {
            (children, "")
        } else {
            (format!("\n{children}"), "</>")
        };

        write!(
            f,
            "<{}{}{} {}>{}{}",
            self.widget.type_name(),
            id,
            class,
            props,
            children,
            close,
        )
    }
}

impl Hash for Component {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.gen_hash().hash(state);
    }
}

impl PartialEq for Component {
    fn eq(&self, other: &Self) -> bool {
        self.gen_hash() == other.gen_hash()
    }
}

/// A trait that needs implement for expand to Component.
pub trait Expand {
    /// Expands Self to a Component.
    fn expand(self) -> Component;
}

macro_rules! impl_expand_to_string {
    ($($ty:ty),*) => {
        $(impl Expand for $ty {
            fn expand(self) -> Component {
                Component::new(Text::new(self))
            }
        })*
    };
}

macro_rules! impl_expand_iter {
    ($($ty:ty),*) => {
        $(impl Expand for $ty {
            fn expand(self) -> Component {
                let arr = self.into_iter().collect::<Vec<Component>>();

                if arr.len() == 1 {
                    arr[0].clone()
                } else {
                    Component::new(Fragment::new(arr))
                }
            }
        })*
    };
}

impl Expand for Component {
    fn expand(self) -> Component {
        self
    }
}

impl<const N: usize> Expand for [Component; N] {
    fn expand(self) -> Component {
        if N == 1 {
            self[0].clone()
        } else {
            Component::new(Fragment::new_from_iter(self))
        }
    }
}

impl<T: Expand> Expand for (T, T) {
    fn expand(self) -> Component {
        Component::new(Fragment::new_from_iter([self.0.expand(), self.1.expand()]))
    }
}

impl<T: Expand> Expand for (T, T, T) {
    fn expand(self) -> Component {
        Component::new(Fragment::new_from_iter([
            self.0.expand(),
            self.1.expand(),
            self.2.expand(),
        ]))
    }
}

impl_expand_to_string!(
    String, &str, usize, u8, u16, u32, u64, u128, isize, i8, i16, i32, i64, i128, bool, f32, f64,
    char
);
impl_expand_iter!(Vec<Component>);

#[widget(default)]
/// A widget that only has children.
pub struct Fragment {
    pub children: Vec<Component>,
}

impl Widget for Fragment {
    fn render(&self, children: Vec<Component>) -> Component {
        panic!("Cannot call a 'render' method in the Fragment widget")
    }

    fn to_dom(&self) -> Option<DomNode> {
        if self.children.is_empty() {
            Some(DomNode::Ignore)
        } else {
            Some(DomNode::Vector(
                self.children
                    .iter()
                    .map(|cpnt| parse_component(cpnt.clone()))
                    .collect::<Vec<_>>(),
            ))
        }
    }
}

impl Fragment {
    /// Create new Fragment from children.
    pub fn new(children: Vec<Component>) -> Self {
        Self { children }
    }

    fn new_from_iter<I: IntoIterator<Item = Component>>(children: I) -> Self {
        Self::new(children.into_iter().collect::<Vec<_>>())
    }
}

#[widget(default)]
/// A termination widget with text.
pub struct Text {
    pub value: String,
}

impl Widget for Text {
    fn render(&self, children: Vec<Component>) -> Component {
        panic!("Cannot call a 'render' method in the Text widget")
    }

    fn to_dom(&self) -> Option<DomNode> {
        Some(DomNode::Text(self.value.clone()))
    }
}

impl Text {
    /// Create new Text from strings.
    pub fn new<S: ToString>(value: S) -> Self {
        Self {
            value: value.to_string(),
        }
    }
}

#[derive(Debug, PartialEq)]
struct DomContainer(DomAst);

#[derive(Debug, PartialEq)]
/// A dom node with hash.
pub struct DomAst(HashCell, DomNode);

#[derive(Debug, PartialEq)]
/// A node variants.
pub enum DomNode {
    /// Include single-node.
    Layer(Box<DomAst>),

    /// Include multiple-node.
    Vector(Vec<DomAst>),

    /// Include text.
    Text(String),

    /// Goto new line within same layer.
    NewLine,

    /// This node is ignored.
    Ignore,
}

impl DomAst {
    fn new(cell: HashCell, node: DomNode) -> Self {
        Self(cell, node)
    }
}

fn parse_layer(original_component: Component) -> DomContainer {
    let root_hash = original_component.gen_hash();
    let expanded_root = original_component.render();

    let ast = DomAst::new(
        root_hash,
        DomNode::Layer(Box::new(parse_component(expanded_root))),
    );

    DomContainer(ast)
}

fn parse_component(cpnt: Component) -> DomAst {
    let cell = cpnt.gen_hash();

    if let Some(dom) = cpnt.to_dom_node() {
        return DomAst::new(cell, dom);
    }

    DomAst::new(
        cell,
        DomNode::Layer(Box::new(parse_component(cpnt.render()))),
    )
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Shape {
    Rect(Rect),
    Point(Point),
    Line(Line),
}

impl Shape {
    fn rect(p1: (u16, u16), p2: (u16, u16)) -> Self {
        Self::Rect(Rect {
            tl: Point {
                cols: p1.0.min(p2.0),
                rows: p1.1.min(p2.1),
            },
            br: Point {
                cols: p1.0.max(p2.0),
                rows: p1.1.max(p2.1),
            },
        })
    }

    fn point(cols: u16, rows: u16) -> Self {
        Self::Point(Point { cols, rows })
    }

    fn line(cols: u16, rows: u16, width: u16) -> Self {
        Self::Line(Line {
            begin: Point { cols, rows },
            end: Point {
                cols: cols + (width.max(1) - 1),
                rows,
            },
        })
    }

    fn is_conflict(&self, other: Self) -> bool {
        self.into_rect().is_conflict(other.into_rect())
    }

    fn into_rect(self) -> Rect {
        match self {
            Self::Rect(rect) => rect,
            Self::Point(pos) => pos.into(),
            Self::Line(line) => line.into(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Rect {
    tl: Point,
    br: Point,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Point {
    cols: u16,
    rows: u16,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Line {
    begin: Point,
    end: Point,
}

impl Rect {
    fn is_conflict(&self, other: Self) -> bool {
        let cols_range = self.tl.cols..=self.br.cols;
        let rows_range = self.tl.rows..=self.br.rows;

        cols_range.contains(&other.tl.cols)
            || cols_range.contains(&other.br.cols)
            || rows_range.contains(&other.tl.rows)
            || rows_range.contains(&other.br.rows)
    }
}

impl From<Point> for Rect {
    fn from(value: Point) -> Self {
        Rect {
            tl: value,
            br: value,
        }
    }
}

impl From<Line> for Rect {
    fn from(value: Line) -> Self {
        Rect {
            tl: value.begin,
            br: value.end,
        }
    }
}

impl Debug for Rect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Rect({:?}, {:?})", self.tl, self.br)
    }
}

impl Debug for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.cols, self.rows)
    }
}

impl Debug for Line {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Line({:?}, {:?})", self.begin, self.end)
    }
}

struct Canvas {
    shape: Shape,
}

impl Canvas {
    fn new(shape: Shape) -> Self {
        Self { shape }
    }
}

struct Layer {
    z_index: usize,
    canvas: Arc<Canvas>,
}

impl Layer {
    fn new(z_index: usize, canvas: Arc<Canvas>) -> Self {
        Self { z_index, canvas }
    }
}

struct CanvasAllocator {
    mem: RwLock<Vec<Layer>>,
}

impl CanvasAllocator {
    fn new() -> Self {
        Self {
            mem: RwLock::new(vec![]),
        }
    }

    fn allocate(&self, z_index: usize, shape: Shape) -> Option<Arc<Canvas>> {
        let mems = &mut self.mem.write().unwrap();

        mems.iter()
            .filter(|layer| layer.z_index == z_index)
            .all(|layer| !layer.canvas.shape.is_conflict(shape))
            .then_some({
                let canvas = Arc::new(Canvas::new(shape));

                mems.push(Layer::new(z_index, canvas.clone()));

                canvas
            })
    }

    fn free(&self, z_index: usize, shape: Shape) {
        let mems = &mut self.mem.write().unwrap();

        mems.retain(|layer| layer.z_index != z_index || !layer.canvas.shape.is_conflict(shape))
    }
}

struct Engine {}

fn draw() {}
