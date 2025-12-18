use crossterm::Command;
use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::terminal::{
    self, Clear, ClearType, DisableLineWrap, EnableLineWrap, EnterAlternateScreen,
    LeaveAlternateScreen, SetSize, SetTitle,
};
use std::fmt;

/// A command for terminal initialize.
pub struct TuiInitialize {
    raw_mode: bool,
    inner: Result<String, fmt::Error>,
}

impl TuiInitialize {
    /// Create new command.
    pub fn new() -> Self {
        Self {
            raw_mode: false,
            inner: Ok(String::new()),
        }
    }

    /// Enable raw_mode.
    pub fn enable_raw_mode(mut self) -> Self {
        self.raw_mode = true;

        self
    }

    fn wrap<C: Command>(mut self, c: C) -> Self {
        if let Ok(ref mut buffer) = self.inner
            && let Err(e) = c.write_ansi(buffer)
        {
            self.inner = Err(e);
        }

        self
    }

    /// Enter alternate.
    pub fn enter_alternate(self) -> Self {
        self.wrap(EnterAlternateScreen)
    }

    /// Hide cursor.
    pub fn hide_cursor(self) -> Self {
        self.wrap(Hide)
    }

    /// Set cursor pos.
    pub fn set_cursor_pos(self, cols: u16, rows: u16) -> Self {
        self.wrap(MoveTo(cols, rows))
    }

    /// Disable line wrap.
    pub fn disable_line_wrap(self) -> Self {
        self.wrap(DisableLineWrap)
    }

    /// Set terminal title.
    pub fn set_title(self, title: &str) -> Self {
        self.wrap(SetTitle(title))
    }

    /// Claer all lines.
    pub fn clear_all(self) -> Self {
        self.wrap(Clear(ClearType::All))
    }

    /// Set new terminal size.
    pub fn set_size(self, cols: u16, rows: u16) -> Self {
        self.wrap(SetSize(cols, rows))
    }
}

impl Command for TuiInitialize {
    fn write_ansi(&self, f: &mut impl fmt::Write) -> fmt::Result {
        if self.raw_mode && terminal::enable_raw_mode().is_err() {
            return Err(fmt::Error);
        }

        write!(f, "{}", self.inner.clone()?)
    }
}

/// A command for restore to original.
pub struct Restore {
    raw_mode: bool,
    inner: Result<String, fmt::Error>,
}

impl Restore {
    /// Create new command.
    pub fn new() -> Self {
        Self {
            raw_mode: false,
            inner: Ok(String::new()),
        }
    }

    /// Active to all flags.
    pub fn all() -> Self {
        Self::new()
            .disable_raw_mode()
            .show_cursor()
            .enable_line_wrap()
            .set_default_title()
            .clear_all()
            .leave_alternate()
            .disable_raw_mode()
    }

    /// Disable raw_mode.
    pub fn disable_raw_mode(mut self) -> Self {
        self.raw_mode = true;

        self
    }

    fn wrap<C: Command>(mut self, c: C) -> Self {
        if let Ok(ref mut buffer) = self.inner
            && let Err(e) = c.write_ansi(buffer)
        {
            self.inner = Err(e);
        }

        self
    }

    /// Leave alternate.
    pub fn leave_alternate(self) -> Self {
        self.wrap(LeaveAlternateScreen)
    }

    /// Show cursor.
    pub fn show_cursor(self) -> Self {
        self.wrap(Show)
    }

    /// Set cursor pos.
    pub fn set_cursor_pos(self, cols: u16, rows: u16) -> Self {
        self.wrap(MoveTo(cols, rows))
    }

    /// Enable line wrap.
    pub fn enable_line_wrap(self) -> Self {
        self.wrap(EnableLineWrap)
    }

    /// Reset terminal title.
    pub fn set_default_title(self) -> Self {
        self.wrap(SetTitle(""))
    }

    /// Claer all lines.
    pub fn clear_all(self) -> Self {
        self.wrap(Clear(ClearType::All))
    }

    /// Set terminal size.
    pub fn set_size(self, cols: u16, rows: u16) -> Self {
        self.wrap(SetSize(cols, rows))
    }
}

impl Command for Restore {
    fn write_ansi(&self, f: &mut impl fmt::Write) -> fmt::Result {
        if self.raw_mode && terminal::disable_raw_mode().is_err() {
            return Err(fmt::Error);
        }

        write!(f, "{}", self.inner.clone()?)
    }
}
