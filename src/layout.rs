pub struct Key {
    pub label: &'static str,
    pub code: u16,
    pub width: f32,
}

pub type Row = Vec<Key>;

pub struct KeyboardLayout {
    pub rows: Vec<Row>,
}

pub struct LayoutOptions {
    pub fx_keys: bool,
}

struct PunctKey {
    label: &'static str,
    code: u16,
}

/// Letter labels mapped to standard QWERTY physical positions (Q-P, A-L, Z-M).
struct AlphaRows {
    top: [&'static str; 10],
    home: [&'static str; 9],
    bottom: [&'static str; 7],
    top_punct: [PunctKey; 3],
    home_punct: [PunctKey; 2],
    bottom_punct: [PunctKey; 3],
}

struct NumberRow {
    labels: [&'static str; 13],
}

struct LayoutDef {
    number: NumberRow,
    alpha: AlphaRows,
}

const TOP_CODES: [u16; 10] = [16, 17, 18, 19, 20, 21, 22, 23, 24, 25];
const HOME_CODES: [u16; 9] = [30, 31, 32, 33, 34, 35, 36, 37, 38];
const BOT_CODES: [u16; 7] = [44, 45, 46, 47, 48, 49, 50];

const QWERTY: LayoutDef = LayoutDef {
    number: NumberRow {
        labels: ["`", "1", "2", "3", "4", "5", "6", "7", "8", "9", "0", "-", "="],
    },
    alpha: AlphaRows {
        top: ["Q", "W", "E", "R", "T", "Y", "U", "I", "O", "P"],
        home: ["A", "S", "D", "F", "G", "H", "J", "K", "L"],
        bottom: ["Z", "X", "C", "V", "B", "N", "M"],
        top_punct: [
            PunctKey { label: "[", code: 26 },
            PunctKey { label: "]", code: 27 },
            PunctKey { label: "\\", code: 43 },
        ],
        home_punct: [
            PunctKey { label: ";", code: 39 },
            PunctKey { label: "'", code: 40 },
        ],
        bottom_punct: [
            PunctKey { label: ",", code: 51 },
            PunctKey { label: ".", code: 52 },
            PunctKey { label: "/", code: 53 },
        ],
    },
};

const DVORAK: LayoutDef = LayoutDef {
    number: NumberRow {
        labels: ["`", "1", "2", "3", "4", "5", "6", "7", "8", "9", "0", "[", "]"],
    },
    alpha: AlphaRows {
        top: ["'", ",", ".", "P", "Y", "F", "G", "C", "R", "L"],
        home: ["A", "O", "E", "U", "I", "D", "H", "T", "N"],
        bottom: [";", "Q", "J", "K", "X", "B", "M"],
        top_punct: [
            PunctKey { label: "/", code: 26 },
            PunctKey { label: "=", code: 27 },
            PunctKey { label: "\\", code: 43 },
        ],
        home_punct: [
            PunctKey { label: "S", code: 39 },
            PunctKey { label: "-", code: 40 },
        ],
        bottom_punct: [
            PunctKey { label: "W", code: 51 },
            PunctKey { label: "V", code: 52 },
            PunctKey { label: "Z", code: 53 },
        ],
    },
};

const COLEMAK: LayoutDef = LayoutDef {
    number: NumberRow {
        labels: ["`", "1", "2", "3", "4", "5", "6", "7", "8", "9", "0", "-", "="],
    },
    alpha: AlphaRows {
        top: ["Q", "W", "F", "P", "G", "J", "L", "U", "Y", ";"],
        home: ["A", "R", "S", "T", "D", "H", "N", "E", "I"],
        bottom: ["Z", "X", "C", "V", "B", "K", "M"],
        top_punct: [
            PunctKey { label: "[", code: 26 },
            PunctKey { label: "]", code: 27 },
            PunctKey { label: "\\", code: 43 },
        ],
        home_punct: [
            PunctKey { label: "O", code: 39 },
            PunctKey { label: "'", code: 40 },
        ],
        bottom_punct: [
            PunctKey { label: ",", code: 51 },
            PunctKey { label: ".", code: 52 },
            PunctKey { label: "/", code: 53 },
        ],
    },
};

const CANARY: LayoutDef = LayoutDef {
    number: NumberRow {
        labels: ["`", "1", "2", "3", "4", "5", "6", "7", "8", "9", "0", "-", "="],
    },
    alpha: AlphaRows {
        top: ["w", "l", "y", "p", "k", "z", "x", "o", "u", ";"],
        home: ["c", "r", "s", "t", "b", "f", "n", "e", "i"],
        bottom: ["j", "v", "d", "g", "q", "m", "h"],
        top_punct: [
            PunctKey { label: "[", code: 26 },
            PunctKey { label: "]", code: 27 },
            PunctKey { label: "\\", code: 43 },
        ],
        home_punct: [
            PunctKey { label: "a", code: 39 },
            PunctKey { label: "'", code: 40 },
        ],
        bottom_punct: [
            PunctKey { label: "/", code: 51 },
            PunctKey { label: ",", code: 52 },
            PunctKey { label: ".", code: 53 },
        ],
    },
};

pub struct Layouts;

impl Layouts {
    pub fn get(name: &str, opts: LayoutOptions) -> KeyboardLayout {
        let def = match name.to_lowercase().as_str() {
            "qwerty" => &QWERTY,
            "dvorak" => &DVORAK,
            "colemak" => &COLEMAK,
            "canary" => &CANARY,
            _ => panic!("Unknown layout: {name}"),
        };
        build_layout(def, opts)
    }
}

fn build_layout(def: &LayoutDef, opts: LayoutOptions) -> KeyboardLayout {
    let mut rows = Vec::new();

    if opts.fx_keys {
        rows.push(f_row());
    }
    rows.push(number_row(&def.number));
    rows.push(top_alpha_row(&def.alpha));
    rows.push(home_alpha_row(&def.alpha));
    rows.push(bottom_alpha_row(&def.alpha));
    rows.push(bottom_row());
    rows.push(arrow_row());

    KeyboardLayout {
        rows,
    }
}

fn number_row(num: &NumberRow) -> Row {
    let codes = [41, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13];
    let mut row: Row = codes
        .iter()
        .zip(num.labels.iter())
        .map(|(&code, &label)| k(label, code, 1.))
        .collect();
    row.push(k("Bksp", 14, 2.));
    row
}

fn top_alpha_row(alpha: &AlphaRows) -> Row {
    let mut row = vec![k("Tab", 15, 1.5)];
    row.extend(
        TOP_CODES
            .iter()
            .zip(alpha.top.iter())
            .map(|(&code, &label)| k(label, code, 1.)),
    );
    row.extend(
        alpha
            .top_punct
            .iter()
            .map(|p| k(p.label, p.code, 1.)),
    );
    row
}

fn home_alpha_row(alpha: &AlphaRows) -> Row {
    let mut row = vec![k("Caps", 58, 1.75)];
    row.extend(
        HOME_CODES
            .iter()
            .zip(alpha.home.iter())
            .map(|(&code, &label)| k(label, code, 1.)),
    );
    row.extend(
        alpha
            .home_punct
            .iter()
            .map(|p| k(p.label, p.code, 1.)),
    );
    row.push(k("Enter", 28, 2.25));
    row
}

fn bottom_alpha_row(alpha: &AlphaRows) -> Row {
    let mut row = vec![k("Shift", 42, 2.25)];
    row.extend(
        BOT_CODES
            .iter()
            .zip(alpha.bottom.iter())
            .map(|(&code, &label)| k(label, code, 1.)),
    );
    row.extend(
        alpha
            .bottom_punct
            .iter()
            .map(|p| k(p.label, p.code, 1.)),
    );
    row.push(k("Shift", 54, 2.75));
    row
}

fn f_row() -> Row {
    vec![
        k("Esc", 1, 1.),
        gap(0.3),
        k("F1", 59, 1.),
        k("F2", 60, 1.),
        k("F3", 61, 1.),
        k("F4", 62, 1.),
        gap(0.3),
        k("F5", 63, 1.),
        k("F6", 64, 1.),
        k("F7", 65, 1.),
        k("F8", 66, 1.),
        gap(0.3),
        k("F9", 67, 1.),
        k("F10", 68, 1.),
        k("F11", 87, 1.),
        k("F12", 88, 1.),
    ]
}

fn bottom_row() -> Row {
    vec![
        k("Ctrl", 29, 1.25),
        k("Win", 125, 1.25),
        k("Alt", 56, 1.25),
        k("Space", 57, 7.5),
        k("Alt", 100, 1.25),
        k("Win", 126, 1.25),
        k("Ctrl", 97, 1.25),
    ]
}

fn arrow_row() -> Row {
    vec![
        gap(11.),
        k("←", 105, 1.),
        k("↓", 108, 1.),
        k("↑", 103, 1.),
        k("→", 106, 1.),
    ]
}

fn k(label: &'static str, code: u16, width: f32) -> Key {
    Key { label, code, width }
}

fn gap(width: f32) -> Key {
    Key {
        label: "",
        code: 0,
        width,
    }
}
