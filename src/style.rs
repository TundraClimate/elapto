/// A wrapper of [Vec<StyleBlock>], It's similar to the .css files.
pub struct StyleSheet {
    blocks: Vec<StyleBlock>,
}

/// A unit of styling.
pub struct StyleBlock {
    ty: BlockType,
    style: Style,
}

/// The enum in style specification types.
pub enum BlockType {
    Id(String),
    Class(String),
}

/// A style apply to [crate::Component].
pub struct Style {
    display: DisplayStyle,
}

/// The enum in `display` properties.
pub enum DisplayStyle {
    /// Block style
    Block,

    /// Inline style
    Inline,

    /// Flex style
    Flex,

    /// Grid style
    Grid,

    /// Disable style
    None,
}
