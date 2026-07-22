use std::time::Instant;

/// Typing statistics: CPM counts characters, WPM counts words (space/enter ends a word).
#[derive(Debug, Default, Clone)]
pub struct TypingStats {
    pub chars: u64,
    pub words: u64,
    pub started: Option<Instant>,
    pending_word: bool,
}

impl TypingStats {
    pub fn on_key_press(&mut self, code: u16) {
        if is_word_end(code) {
            if self.pending_word {
                self.words += 1;
                self.pending_word = false;
            }
            return;
        }

        if is_typable(code) {
            if self.started.is_none() {
                self.started = Some(Instant::now());
            }
            self.chars += 1;
            self.pending_word = true;
        }
    }

    pub fn cpm(&self) -> f64 {
        let elapsed = self.elapsed_minutes();
        if elapsed <= 0.0 {
            return 0.0;
        }
        self.chars as f64 / elapsed
    }

    pub fn wpm(&self) -> f64 {
        let elapsed = self.elapsed_minutes();
        if elapsed <= 0.0 {
            return 0.0;
        }
        self.words as f64 / elapsed
    }

    pub fn display_text(&self) -> String {
        format!("WPM: {:.0}  CPM: {:.0}", self.wpm(), self.cpm())
    }

    fn elapsed_minutes(&self) -> f64 {
        match self.started {
            Some(t) => t.elapsed().as_secs_f64() / 60.0,
            None => 0.0,
        }
    }
}

const WORD_END_KEYS: [u16; 2] = [57, 28]; // Space, Enter

const IGNORED_KEYS: [u16; 28] = [
    1, 29, 42, 54, 56, 58, 97, 100, 125, 126, // modifiers, caps, esc
    59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 87, 88, // F-keys
    103, 105, 106, 108, // arrows
    15, 14, // tab, backspace
];

fn is_word_end(code: u16) -> bool {
    WORD_END_KEYS.contains(&code)
}

fn is_typable(code: u16) -> bool {
    !IGNORED_KEYS.contains(&code) && code != 0
}
