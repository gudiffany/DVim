use crossterm::style::Color;

const TAB_STOP: usize = 8;

#[derive(Clone, Copy, PartialEq)]
pub enum Highlight {
    Normal,
    Number,
}

pub struct Raw {
    pub chars: String,
    pub render: String,
    pub hl: Vec<Highlight>,
}

impl Highlight {
    pub fn synatx_to_color(&self) -> Color {
        match self {
            Highlight::Normal => Color::White,
            Highlight::Number => Color::Red,
        }
    }
}

impl Raw {
    pub fn new(chars: String) -> Self {
        let mut res = Self {
            chars,
            render: String::new(),
            hl: Vec::new(),
        };
        res.render_raw();
        res
    }
    pub fn render_len(&self) -> usize {
        self.render.len()
    }

    pub fn len(&self) -> usize {
        self.chars.len()
    }

    pub fn cx_to_rx(&self, cx: u16) -> u16 {
        let mut rx = 0;
        for c in self.chars.chars().take(cx as usize) {
            if c == '\t' {
                rx += (TAB_STOP - 1) - (rx % TAB_STOP);
            }
            rx += 1;
        }
        rx as u16
    }

    pub fn rx_to_cx(&self, rx: usize) -> u16 {
        let mut cur_rx = 0;
        for (cx, c) in self.chars.chars().enumerate() {
            if c == '\t' {
                cur_rx += (TAB_STOP - 1) - (cur_rx % TAB_STOP);
            }
            cur_rx += 1;
            if cur_rx > rx {
                return cx as u16;
            }
        }
        self.chars.len() as u16
    }
    pub fn insert_char(&mut self, at: usize, c: char) {
        if at >= self.chars.len() {
            self.chars.push(c);
        } else {
            self.chars.insert(at, c);
        }
        self.render_raw()
    }

    pub fn del_char(&mut self, at: usize) -> bool {
        if at >= self.chars.len() {
            false
        } else {
            self.chars.remove(at);
            self.render_raw();
            true
        }
    }
    pub fn split(&mut self, at: usize) -> String {
        let result = self.chars.split_off(at);
        self.render_raw();
        result
    }
    pub fn append_string(&mut self, s: &str) {
        self.chars.push_str(s);
        self.render_raw()
    }

    pub fn render_raw(&mut self) {
        let mut render = String::new();
        let mut idx = 0;
        for c in self.chars.chars() {
            match c {
                '\t' => {
                    render.push(' ');
                    idx += 1;
                    while idx % TAB_STOP != 0 {
                        render.push(' ');
                        idx += 1;
                    }
                }
                _ => {
                    render.push(c);
                    idx += 1;
                }
            }
        }
        self.render = render;
        self.update_synxtax();
    }

    fn update_synxtax(&mut self) {
        self.hl = vec![Highlight::Normal; self.render.len()];
        for (i, c) in self.render.chars().enumerate() {
            if c.is_digit(10) {
                self.hl[i] = Highlight::Number;
            }
        }
    }
}
