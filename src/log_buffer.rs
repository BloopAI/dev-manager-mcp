use std::collections::VecDeque;

const MAX_BYTES: usize = 512 * 1024;
const MAX_TAIL_LINES: usize = 100;

#[derive(Clone)]
pub struct LogBuffer {
    logs: VecDeque<String>,
    total_bytes: usize,
}

impl LogBuffer {
    pub fn new() -> Self {
        Self {
            logs: VecDeque::new(),
            total_bytes: 0,
        }
    }

    pub fn push(&mut self, line: String) {
        let line_bytes = line.len();
        self.total_bytes += line_bytes;
        self.logs.push_back(line);

        while self.total_bytes > MAX_BYTES && !self.logs.is_empty() {
            if let Some(old_line) = self.logs.pop_front() {
                self.total_bytes -= old_line.len();
            }
        }
    }

    pub fn tail(&self) -> (String, bool) {
        let len = self.logs.len();
        let start = len.saturating_sub(MAX_TAIL_LINES);
        
        let mut out = String::new();
        for line in self.logs.iter().skip(start) {
            out.push_str(line);
        }
        
        let truncated = len > MAX_TAIL_LINES;
        (out, truncated)
    }
}
