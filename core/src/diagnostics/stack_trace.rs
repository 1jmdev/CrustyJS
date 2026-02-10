#[derive(Debug, Clone)]
pub struct CallFrame {
    pub function_name: String,
    pub file: String,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone, Default)]
pub struct CallStack {
    frames: Vec<CallFrame>,
}

impl CallStack {
    pub fn push_frame(&mut self, frame: CallFrame) {
        self.frames.push(frame);
    }

    pub fn pop_frame(&mut self) {
        self.frames.pop();
    }

    pub fn snapshot(&self) -> Vec<CallFrame> {
        self.frames.clone()
    }

    pub fn format_trace(&self) -> String {
        let mut out = String::new();
        for frame in self.frames.iter().rev() {
            out.push_str(&format!(
                "    at {} ({}:{}:{})\n",
                frame.function_name, frame.file, frame.line, frame.col
            ));
        }
        out
    }
}
