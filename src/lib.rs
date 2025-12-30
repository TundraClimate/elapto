#![allow(unused, clippy::new_without_default)]
#![warn(missing_docs, unused_imports)]

//! The layer based TUI rendering library.
//!
//! **THIS CRATE IS CURRENTLY BETA VERSION**  
//! README and this docs.rs is BETA ver, Information will update always.  

mod hash_cell;
mod style;
mod tui;

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::io::{self, Write};
use std::sync::{Arc, RwLock};
use std::time::Duration;

use crossterm::execute;

pub use hash_cell::HashCell;
pub use tui::Restore;
pub use tui::TuiInitialize;

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
/// The enum that includes the available shapes.
pub enum Shape {
    /// The rectangle.
    Rect(Rect),

    /// The pointer.
    Point(Point),

    /// The line.
    Line(Line),
}

impl Shape {
    /// Create new rectangle.
    pub fn rect(p1: (u16, u16), p2: (u16, u16)) -> Self {
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

    /// Create new pointer.
    pub fn point(cols: u16, rows: u16) -> Self {
        Self::Point(Point { cols, rows })
    }

    /// Create new line from width.
    pub fn line(cols: u16, rows: u16, width: u16) -> Self {
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
/// A struct that represents rectangle area.
pub struct Rect {
    tl: Point,
    br: Point,
}

#[derive(Clone, Copy, PartialEq, Eq)]
/// A struct that represents 1x1 pointer.
pub struct Point {
    cols: u16,
    rows: u16,
}

#[derive(Clone, Copy, PartialEq, Eq)]
/// A struct that represents begin-end line.
pub struct Line {
    begin: Point,
    end: Point,
}

impl Rect {
    fn is_conflict(&self, other: Self) -> bool {
        let self_cols_range = self.tl.cols..=self.br.cols;
        let self_rows_range = self.tl.rows..=self.br.rows;
        let other_cols_range = other.tl.cols..=other.br.cols;
        let other_rows_range = other.tl.rows..=other.br.rows;

        let is_surrounded_by_self = self_cols_range.contains(&other.tl.cols)
            && self_cols_range.contains(&other.br.cols)
            && self_rows_range.contains(&other.tl.rows)
            && self_rows_range.contains(&other.br.rows);

        let is_surrounded_by_other = other_cols_range.contains(&self.tl.cols)
            && other_cols_range.contains(&self.br.cols)
            && other_rows_range.contains(&self.tl.rows)
            && other_rows_range.contains(&self.br.rows);

        let is_crossed = !(!self_cols_range.contains(&other.tl.cols)
            && !self_rows_range.contains(&other.tl.rows)
            && !self_cols_range.contains(&other.br.cols)
            && !self_rows_range.contains(&other.br.rows)
            && !other_cols_range.contains(&self.tl.cols)
            && !other_rows_range.contains(&self.tl.rows)
            && !other_cols_range.contains(&self.br.cols)
            && !other_rows_range.contains(&self.br.rows));

        is_surrounded_by_self || is_surrounded_by_other || is_crossed
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

impl Debug for Shape {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let shape: Box<dyn Debug> = match self {
            Self::Rect(rect) => Box::new(rect),
            Self::Point(point) => Box::new(point),
            Self::Line(line) => Box::new(line),
        };

        write!(f, "{:?}", shape)
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

#[derive(Clone, Copy, PartialEq, Eq)]
/// A source for compare with prev result.
pub struct Source {
    /// inner cell.
    pub cell: HashCell,
}

/// The enum that represents commands for drawing.
pub enum DrawCommand {
    /// Order the line.
    Line(String),

    /// Order the clear.
    Clear {
        /// rows.
        rows: u16,

        /// begin.
        begin: u16,

        /// end.
        end: u16,
    },
}

/// A struct that uses by renderer for drawing.
pub struct Canvas {
    shape: Shape,
    source: Source,
}

impl Canvas {
    fn new(shape: Shape, source: Source) -> Self {
        Self { shape, source }
    }

    /// Executes the draw commands.
    pub fn draw(&self, cmds: &[DrawCommand]) -> io::Result<()> {
        let mut lines = 0u16;

        let rect = self.shape.into_rect();
        let rel_cols = rect.tl.cols;
        let mut rel_rows = rect.tl.rows;
        let out_cols = rect.br.cols + 1;
        let legal_rows = rect.tl.rows..=rect.br.rows;

        for cmd in cmds.iter() {
            match cmd {
                DrawCommand::Line(line) if legal_rows.contains(&rel_rows) => {
                    draw_p((rel_cols, rel_rows), out_cols, line)?;

                    rel_rows += 1;
                    lines += 1;
                }
                DrawCommand::Clear { rows, begin, end } if legal_rows.contains(rows) => {
                    draw_v(*rows, *begin, *end)?
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Executes the draw commands with excludes.
    pub fn draw_with_excludes(&self, cmds: &[DrawCommand], excludes: &[Shape]) -> io::Result<()> {
        let conflicts = excludes
            .iter()
            .filter(|s| s.is_conflict(self.shape))
            .map(|s| s.into_rect())
            .collect::<Vec<_>>();

        if conflicts.is_empty() {
            return self.draw(cmds);
        }

        let rect = self.shape.into_rect();

        let mut line_exclude: BTreeMap<u16, Vec<u16>> = BTreeMap::new();

        conflicts.iter().for_each(|r| {
            let tl_cols = r.tl.cols.max(rect.tl.cols);
            let tl_rows = r.tl.rows.max(rect.tl.rows);
            let br_cols = r.br.cols.min(rect.br.cols);
            let br_rows = r.br.rows.min(rect.br.rows);

            for i in tl_rows..=br_rows {
                match line_exclude.get_mut(&i) {
                    Some(lines) => lines.extend((tl_cols..=br_cols).collect::<Vec<_>>()),
                    None => {
                        line_exclude.insert(i, (tl_cols..=br_cols).collect());
                    }
                }
            }
        });

        line_exclude.iter_mut().for_each(|(_, mut points)| {
            points.dedup();
            points.sort();
        });

        let excludes = line_exclude
            .into_iter()
            .map(|(l, pos)| {
                let mut start = pos[0];
                let mut prev = pos[0];
                let mut res = vec![];

                for &x in &pos[1..] {
                    if x != prev + 1 {
                        res.push(start..=prev);

                        start = x;
                    }

                    prev = x;
                }

                res.push(start..=prev);

                (l, res)
            })
            .collect::<BTreeMap<u16, _>>();

        let mut lines = 0u16;
        let rel_cols = rect.tl.cols;
        let mut rel_rows = rect.tl.rows;
        let out_cols = rect.br.cols + 1;
        let legal_rows = rect.tl.rows..=rect.br.rows;

        for cmd in cmds.iter() {
            match cmd {
                DrawCommand::Line(line) if legal_rows.contains(&rel_rows) => {
                    match excludes.get(&rel_rows) {
                        Some(ranges) => {
                            let mut rel_cols = rel_cols;
                            let mut line = line.clone();

                            for range in ranges {
                                let moveto = (rel_cols, rel_rows);
                                let rel_out_cols = *range.start();

                                draw_p(moveto, rel_out_cols, &line)?;

                                if range.end() >= &out_cols {
                                    break;
                                }

                                line.replace_range(
                                    (rel_cols as usize)..=(*range.end() as usize),
                                    "",
                                );
                                rel_cols = range.end() + 1;
                            }

                            draw_p((rel_cols, rel_rows), out_cols, &line)?;
                        }
                        None => draw_p((rel_cols, rel_rows), out_cols, line)?,
                    }

                    rel_rows += 1;
                    lines += 1;
                }
                DrawCommand::Clear { rows, begin, end } if legal_rows.contains(rows) => {
                    match excludes.get(rows) {
                        Some(ranges) => {
                            let mut begin = *begin;
                            let mut inner_end = *end;

                            for range in ranges {
                                inner_end = *range.start();

                                draw_v(*rows, begin, inner_end)?;

                                begin = range.end() + 1;
                            }

                            draw_v(*rows, begin, *end)?;
                        }
                        None => draw_v(*rows, *begin, *end)?,
                    }
                }
                _ => {}
            }
        }

        Ok(())
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

/// A struct that holding areas for canvas layout.
pub struct CanvasAllocator {
    indexes: RwLock<BTreeSet<usize>>,
    mem: RwLock<Vec<Layer>>,
}

impl CanvasAllocator {
    /// Create new allocator.
    pub fn new() -> Self {
        Self {
            indexes: RwLock::new(BTreeSet::new()),
            mem: RwLock::new(vec![]),
        }
    }

    fn insert_index(&self, z_index: usize) {
        self.indexes.write().unwrap().insert(z_index);
    }

    fn remove_index(&self, z_index: usize) {
        self.indexes.write().unwrap().remove(&z_index);
    }

    fn reset(&self) {
        self.mem.write().unwrap().clear();
        self.indexes.write().unwrap().clear();
    }

    /// Allocate the shape.
    pub fn allocate(&self, z_index: usize, shape: Shape, source: Source) -> Option<Arc<Canvas>> {
        let mems = &mut self.mem.write().unwrap();

        mems.iter()
            .filter(|layer| layer.z_index == z_index)
            .all(|layer| !layer.canvas.shape.is_conflict(shape))
            .then_some({
                let canvas = Arc::new(Canvas::new(shape, source));

                mems.push(Layer::new(z_index, canvas.clone()));
                self.insert_index(z_index);

                canvas
            })
    }

    /// Free areas by source.
    pub fn free(&self, z_index: usize, source: Source) {
        let mems = &mut self.mem.write().unwrap();

        mems.retain(|layer| layer.z_index != z_index || layer.canvas.source != source);

        if !mems.iter().any(|l| l.z_index == z_index) {
            self.remove_index(z_index);
        }
    }
}

fn draw<S: AsRef<str>>(moveto: (u16, u16), text: S) -> io::Result<()> {
    use crossterm::cursor::MoveTo;
    use crossterm::execute;
    use crossterm::style::Print;

    execute!(
        io::stdout(),
        MoveTo(moveto.0, moveto.1),
        Print(text.as_ref())
    )?;

    Ok(())
}

fn draw_p(moveto: (u16, u16), out_cols: u16, paragraph: &str) -> io::Result<()> {
    use unicode_width::UnicodeWidthStr;

    if out_cols <= moveto.0 {
        return Ok(());
    }

    let length = paragraph.width();
    let out_size = out_cols - moveto.0;

    let legal_length = length.min(out_size.into());

    draw(moveto, &paragraph[..legal_length])
}

fn draw_v(rows: u16, begin: u16, end: u16) -> io::Result<()> {
    use crossterm::style::ResetColor;

    if end <= begin {
        return Ok(());
    }

    let void_text = format!("{}{}", ResetColor, " ".repeat((end - begin).into()));

    draw((begin, rows), void_text)
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// A wrapper struct for tick speed.
pub struct TickSpeed(Duration);

impl TickSpeed {
    /// Create new wrapper from milliseconds.
    pub fn new(tick_ms: u64) -> Self {
        Self::from(Duration::from_millis(tick_ms))
    }

    /// Create new wrapper from seconds.
    pub fn new_secs(tick_secs: u64) -> Self {
        Self::from(Duration::from_secs(tick_secs))
    }
}

impl From<Duration> for TickSpeed {
    fn from(value: Duration) -> Self {
        Self(value)
    }
}

impl Default for TickSpeed {
    fn default() -> Self {
        Self::new(10)
    }
}

/// A struct contains what to do when terminal switch.
pub struct TerminalSwitch {
    init: TuiInitialize,
    restore: Restore,
}

impl TerminalSwitch {
    /// Create new terminal switch.
    pub fn new(init: TuiInitialize, restore: Restore) -> Self {
        Self { init, restore }
    }

    /// Init terminal writer with initialize switch.
    pub fn init(&self, writer: &mut impl Write) -> io::Result<()> {
        execute!(writer, &self.init)
    }

    /// Restore terminal writer with restore switch.
    pub fn restore(&self, writer: &mut impl Write) -> io::Result<()> {
        execute!(writer, &self.restore)
    }
}

impl Default for TerminalSwitch {
    fn default() -> Self {
        Self::new(
            TuiInitialize::new()
                .enter_alternate()
                .enable_raw_mode()
                .hide_cursor(),
            Restore::all(),
        )
    }
}

#[derive(Default)]
/// A struct of renderer engine.
pub struct Engine {
    tick_speed: TickSpeed,
    terminal_switch: TerminalSwitch,
}

impl Engine {
    /// Change `tick_speed` to new speed.
    pub fn tick_speed(mut self, tick_speed: TickSpeed) -> Self {
        self.tick_speed = tick_speed;

        self
    }

    /// Change `terminal_switch` to new that.
    pub fn terminal_switch(mut self, terminal_switch: TerminalSwitch) -> Self {
        self.terminal_switch = terminal_switch;

        self
    }
}
