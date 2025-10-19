use std::collections::HashMap;
use std::str::FromStr;

/// Parsing error type.
type ParseError = String;

/// A wrapper of [HashMap<BlockType, Style>], It's similar to the .css files.
pub struct StyleSheet {
    blocks: HashMap<BlockType, Style>,
}

impl StyleSheet {
    /// Create a new sheet.
    pub fn new() -> Self {
        Self {
            blocks: HashMap::new(),
        }
    }

    /// Get style with ident.
    pub fn get(&self, ty: BlockType) -> Option<&Style> {
        self.blocks.get(&ty)
    }

    /// Style block appends to sheet.
    pub fn append(mut self, ty: BlockType, style: Style) -> Self {
        self.blocks.insert(ty, style);

        self
    }

    /// Style block appends to sheet, but error occurs if try parsing failed.
    pub fn try_append(mut self, ty: &str, style: Style) -> Result<Self, ParseError> {
        self.blocks.insert(BlockType::from_str(ty)?, style);

        Ok(self)
    }
}

#[derive(Eq, PartialEq, Hash)]
/// The enum in style specification types.
pub enum BlockType {
    /// Identifier
    Id(String),

    /// Class
    Class(String),
}

impl FromStr for BlockType {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() < 2 {
            return Err(format!("value '{s}' too short"));
        }

        match &s[0..1] {
            "#" => Ok(Self::Id(s[1..].to_string())),
            "." => Ok(Self::Class(s[1..].to_string())),
            _ => Err(format!("{s} isn't available")),
        }
    }
}

/// A style apply to [crate::Component].
pub struct Style {
    display: OuterDisplay,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            display: OuterDisplay::Block(InnerDisplay::Flow),
        }
    }
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
