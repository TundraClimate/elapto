use std::collections::HashMap;
use std::str::FromStr;

/// Parsing error type.
pub type ParseError = String;

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
    pub fn try_append<S: AsRef<str>>(mut self, ty: S, style: Style) -> Result<Self, ParseError> {
        self.blocks.insert(BlockType::from_str(ty.as_ref())?, style);

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
    /// A display prop.
    pub display: OuterDisplay,
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

impl TryFrom<&str> for OuterDisplay {
    type Error = ParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let constant = value.trim().to_ascii_uppercase();

        let (pre, post) = constant.split_once("_").unzip();

        const OUTER_WORDS: [&str; 3] = ["BLOCK", "INLINE", "NONE"];
        const INNER_WORDS: [&str; 3] = ["FLOW", "FLEX", "GRID"];

        if let Some(pre) = pre
            && let Some(post) = post
        {
            let is_valid_words = OUTER_WORDS.contains(&pre) && INNER_WORDS.contains(&post);

            if is_valid_words {
                let inner = match post {
                    "FLOW" => InnerDisplay::Flow,
                    "FLEX" => InnerDisplay::Flex,
                    "GRID" => InnerDisplay::Grid,
                    _ => unreachable!(),
                };

                let outer = match pre {
                    "BLOCK" => OuterDisplay::Block(inner),
                    "INLINE" => OuterDisplay::Inline(inner),
                    _ => unreachable!(),
                };

                Ok(outer)
            } else {
                Err(format!("combined value {} is invalid", constant))
            }
        } else {
            let contains_words = OUTER_WORDS.contains(&constant.as_str())
                || INNER_WORDS.contains(&constant.as_str());

            if contains_words {
                let disp = match constant.as_str() {
                    "BLOCK" | "FLOW" => OuterDisplay::Block(InnerDisplay::Flow),
                    "INLINE" => OuterDisplay::Inline(InnerDisplay::Flow),
                    "NONE" => OuterDisplay::None,
                    "FLEX" => OuterDisplay::Block(InnerDisplay::Flex),
                    "GRID" => OuterDisplay::Block(InnerDisplay::Grid),
                    _ => unreachable!(),
                };

                Ok(disp)
            } else {
                Err(format!("{} is invalid", constant))
            }
        }
    }
}
