use std::collections::HashMap;

/// A wrapper of [HashMap<BlockType, Style>], It's similar to the .css files.
pub struct StyleSheet {
    blocks: HashMap<BlockType, Style>,
}

/// The enum in style specification types.
pub enum BlockType {
    Id(String),
    Class(String),
}

/// A style apply to [crate::Component].
pub struct Style {
    display: OuterDisplay,
}

/// The enum in `inner-display` properties.
pub enum InnerDisplay {
    /// Default style
    Flow,

    /// Flex style
    Flex,

    /// Grid style
    Grid,
}

/// The enum in `display` properties.
pub enum OuterDisplay {
    /// Block style
    Block(InnerDisplay),

    /// Inline style
    Inline(InnerDisplay),

    /// Disable rendering
    None,
}
