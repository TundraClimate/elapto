use crossterm::Command;
use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::terminal::{
    self, Clear, ClearType, DisableLineWrap, EnableLineWrap, EnterAlternateScreen,
    LeaveAlternateScreen, SetSize, SetTitle,
};
use std::fmt;

struct TuiInitialize {
    raw_mode: bool,
    inner: Result<String, fmt::Error>,
}

impl TuiInitialize {
    pub fn new() -> Self {
        Self {
            raw_mode: false,
            inner: Ok(String::new()),
        }
    }

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

    pub fn enter_alternate(self) -> Self {
        self.wrap(EnterAlternateScreen)
    }

    pub fn hide_cursor(self) -> Self {
        self.wrap(Hide)
    }

    pub fn set_cursor_pos(self, cols: u16, rows: u16) -> Self {
        self.wrap(MoveTo(cols, rows))
    }

    pub fn disable_line_wrap(self) -> Self {
        self.wrap(DisableLineWrap)
    }

    pub fn set_title(self, title: &str) -> Self {
        self.wrap(SetTitle(title))
    }

    pub fn clear_all(self) -> Self {
        self.wrap(Clear(ClearType::All))
    }

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

struct Restore {
    raw_mode: bool,
    inner: Result<String, fmt::Error>,
}

impl Restore {
    pub fn new() -> Self {
        Self {
            raw_mode: false,
            inner: Ok(String::new()),
        }
    }

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

    pub fn leave_alternate(self) -> Self {
        self.wrap(LeaveAlternateScreen)
    }

    pub fn show_cursor(self) -> Self {
        self.wrap(Show)
    }

    pub fn set_cursor_pos(self, cols: u16, rows: u16) -> Self {
        self.wrap(MoveTo(cols, rows))
    }

    pub fn enable_line_wrap(self) -> Self {
        self.wrap(EnableLineWrap)
    }

    pub fn set_default_title(self) -> Self {
        self.wrap(SetTitle(""))
    }

    pub fn clear_all(self) -> Self {
        self.wrap(Clear(ClearType::All))
    }

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
