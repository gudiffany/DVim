const TAB_STOP: usize = 8;

pub struct Raw {
    pub chars: String,
    pub render: String,
}

impl Raw {
    pub fn new(chars: String) -> Self {
        let render = Raw::render_raw(&chars);
        Self { chars, render }
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
    pub fn insert_char(&mut self, at: usize, c: char) {
        if at >= self.chars.len() {
            self.chars.push(c);
        } else {
            self.chars.insert(at, c);
        }
        self.render = Raw::render_raw(&self.chars);
    }

    pub fn del_char(&mut self, at: usize) -> bool {
        if at >= self.chars.len() {
            false
        } else {
            self.chars.remove(at);
            self.render = Raw::render_raw(&self.chars);
            true
        }
    }
    pub fn split(&mut self, at: usize) -> String {
        let result = self.chars.split_off(at);
        self.render =  Raw::render_raw(&self.chars);
        result
    }
    pub fn append_string(&mut self, s: &str) {
        self.chars.push_str(s);
        self.render = Raw::render_raw(&self.chars);
    }

    pub fn render_raw(chars: &String) -> String {
        let mut render = String::new();
        let mut idx = 0;
        for c in chars.chars() {
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
        render
    }
}
