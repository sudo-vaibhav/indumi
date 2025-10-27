use crossterm::event::{KeyCode, KeyEvent};
use crate::calc::Calculator;
use std::cell::RefCell;

#[derive(Debug)]
pub struct Editor {
    pub lines: Vec<String>,
    pub cursor_line: usize,
    pub cursor_col: usize,
    pub calculator: RefCell<Calculator>,
}

impl Editor {
    pub fn new(calculator: Calculator) -> Self {
        Self {
            lines: vec![String::new()],
            cursor_line: 0,
            cursor_col: 0,
            calculator: RefCell::new(calculator),
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char(c) => self.insert_char(c),
            KeyCode::Backspace => self.backspace(),
            KeyCode::Delete => self.delete(),
            KeyCode::Enter => self.new_line(),
            KeyCode::Left => self.move_left(),
            KeyCode::Right => self.move_right(),
            KeyCode::Up => self.move_up(),
            KeyCode::Down => self.move_down(),
            KeyCode::Home => self.move_home(),
            KeyCode::End => self.move_end(),
            _ => {}
        }
    }

    fn insert_char(&mut self, c: char) {
        let line = &mut self.lines[self.cursor_line];
        line.insert(self.cursor_col, c);
        self.cursor_col += 1;
    }

    fn backspace(&mut self) {
        if self.cursor_col > 0 {
            let line = &mut self.lines[self.cursor_line];
            line.remove(self.cursor_col - 1);
            self.cursor_col -= 1;
        } else if self.cursor_line > 0 {
            // Join with previous line
            let current_line = self.lines.remove(self.cursor_line);
            self.cursor_line -= 1;
            self.cursor_col = self.lines[self.cursor_line].len();
            self.lines[self.cursor_line].push_str(&current_line);
        }
    }

    fn delete(&mut self) {
        let line = &mut self.lines[self.cursor_line];
        if self.cursor_col < line.len() {
            line.remove(self.cursor_col);
        } else if self.cursor_line < self.lines.len() - 1 {
            // Join with next line
            let next_line = self.lines.remove(self.cursor_line + 1);
            self.lines[self.cursor_line].push_str(&next_line);
        }
    }

    fn new_line(&mut self) {
        let line = &self.lines[self.cursor_line];
        let remainder = line[self.cursor_col..].to_string();
        self.lines[self.cursor_line].truncate(self.cursor_col);
        self.cursor_line += 1;
        self.lines.insert(self.cursor_line, remainder);
        self.cursor_col = 0;
    }

    fn move_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        } else if self.cursor_line > 0 {
            self.cursor_line -= 1;
            self.cursor_col = self.lines[self.cursor_line].len();
        }
    }

    fn move_right(&mut self) {
        let line_len = self.lines[self.cursor_line].len();
        if self.cursor_col < line_len {
            self.cursor_col += 1;
        } else if self.cursor_line < self.lines.len() - 1 {
            self.cursor_line += 1;
            self.cursor_col = 0;
        }
    }

    fn move_up(&mut self) {
        if self.cursor_line > 0 {
            self.cursor_line -= 1;
            let line_len = self.lines[self.cursor_line].len();
            if self.cursor_col > line_len {
                self.cursor_col = line_len;
            }
        }
    }

    fn move_down(&mut self) {
        if self.cursor_line < self.lines.len() - 1 {
            self.cursor_line += 1;
            let line_len = self.lines[self.cursor_line].len();
            if self.cursor_col > line_len {
                self.cursor_col = line_len;
            }
        }
    }

    fn move_home(&mut self) {
        self.cursor_col = 0;
    }

    fn move_end(&mut self) {
        self.cursor_col = self.lines[self.cursor_line].len();
    }
}
