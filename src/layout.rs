#[derive(Clone)]
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
    pub split: Option<String>,
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
        if let Some(ref split_name) = opts.split {
            return get_split_layout(split_name);
        }
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

fn get_split_layout(name: &str) -> KeyboardLayout {
    match name.to_lowercase().as_str() {
        "lily58" => lily58(),
        "corne" => corne(),
        "cheapino" => cheapino(),
        _ => panic!("Unknown split keyboard: {name}. Supported: lily58, corne, cheapino"),
    }
}

fn build_split(left_rows: Vec<Vec<Key>>, right_rows: Vec<Vec<Key>>, thumb: Vec<Key>, gap_width: f32) -> KeyboardLayout {
    let g = gap(gap_width);
    let mut rows = Vec::new();
    for (left, right) in left_rows.iter().zip(right_rows.iter()) {
        let mut row = left.clone();
        row.push(g.clone());
        row.extend(right.iter().cloned());
        rows.push(row);
    }
    if !thumb.is_empty() {
        rows.push(thumb);
    }
    KeyboardLayout { rows }
}

fn lily58() -> KeyboardLayout {
    // Each side = 7u wide, every row matches
    let left = vec![
        vec![k("Esc", 1, 1.), k("`", 41, 1.), k("1", 2, 1.), k("2", 3, 1.), k("3", 4, 1.), k("4", 5, 1.), k("5", 6, 1.)],
        vec![k("Tab", 15, 2.), k("Q", 16, 1.), k("W", 17, 1.), k("E", 18, 1.), k("R", 19, 1.), k("T", 20, 1.)],
        vec![k("Caps", 58, 2.), k("A", 30, 1.), k("S", 31, 1.), k("D", 32, 1.), k("F", 33, 1.), k("G", 34, 1.)],
        vec![k("Shift", 42, 2.), k("Z", 44, 1.), k("X", 45, 1.), k("C", 46, 1.), k("V", 47, 1.), k("B", 48, 1.)],
    ];
    let right = vec![
        vec![k("6", 7, 1.), k("7", 8, 1.), k("8", 9, 1.), k("9", 10, 1.), k("0", 11, 1.), k("-", 12, 1.), k("=", 13, 1.), k("Bksp", 14, 1.)],
        vec![k("Y", 21, 1.), k("U", 22, 1.), k("I", 23, 1.), k("O", 24, 1.), k("P", 25, 1.), k("[", 26, 1.), k("]", 27, 1.), k("\\", 43, 1.)],
        vec![k("H", 35, 1.), k("J", 36, 1.), k("K", 37, 1.), k("L", 38, 1.), k(";", 39, 1.), k("'", 40, 1.), k("Enter", 28, 2.)],
        vec![k("N", 49, 1.), k("M", 50, 1.), k(",", 51, 1.), k(".", 52, 1.), k("/", 53, 1.), k("Shift", 54, 2.)],
    ];
    let thumb = vec![
        k("Ctrl", 29, 1.25), k("Win", 125, 1.25), k("Space", 57, 1.5),
        gap(1.5),
        k("Space", 57, 1.5), k("Win", 126, 1.25), k("Ctrl", 97, 1.25),
    ];
    build_split(left, right, thumb, 1.5)
}

fn corne() -> KeyboardLayout {
    // Each side = 6u wide, every row matches
    let left = vec![
        vec![k("Q", 16, 1.), k("W", 17, 1.), k("E", 18, 1.), k("R", 19, 1.), k("T", 20, 1.), k("Tab", 15, 1.)],
        vec![k("A", 30, 1.), k("S", 31, 1.), k("D", 32, 1.), k("F", 33, 1.), k("G", 34, 1.), k("Caps", 58, 1.)],
        vec![k("Z", 44, 1.), k("X", 45, 1.), k("C", 46, 1.), k("V", 47, 1.), k("B", 48, 1.), k("Shift", 42, 1.)],
    ];
    let right = vec![
        vec![k("Y", 21, 1.), k("U", 22, 1.), k("I", 23, 1.), k("O", 24, 1.), k("P", 25, 1.), k("\\", 43, 1.)],
        vec![k("H", 35, 1.), k("J", 36, 1.), k("K", 37, 1.), k("L", 38, 1.), k(";", 39, 1.), k("Enter", 28, 1.)],
        vec![k("N", 49, 1.), k("M", 50, 1.), k(",", 51, 1.), k(".", 52, 1.), k("/", 53, 1.), k("Shift", 54, 1.)],
    ];
    let thumb = vec![
        k("Ctrl", 29, 1.), k("Win", 125, 1.), k("Alt", 56, 1.),
        gap(1.5),
        k("Alt", 100, 1.), k("Win", 126, 1.), k("Ctrl", 97, 1.),
    ];
    build_split(left, right, thumb, 1.5)
}

fn cheapino() -> KeyboardLayout {
    // Each side = 5.5u wide: Esc/Bksp 1.5u + 5 alpha 1u = 6.5u left, 5 alpha 1u + Bksp 1.5u = 6.5u right
    let left = vec![
        vec![k("Esc", 1, 1.5), k("Q", 16, 1.), k("W", 17, 1.), k("E", 18, 1.), k("R", 19, 1.), k("T", 20, 1.)],
        vec![k("A", 30, 1.), k("S", 31, 1.), k("D", 32, 1.), k("F", 33, 1.), k("G", 34, 1.), k("Tab", 15, 1.)],
        vec![k("Shift", 42, 1.), k("Z", 44, 1.), k("X", 45, 1.), k("C", 46, 1.), k("V", 47, 1.), k("B", 48, 1.)],
    ];
    let right = vec![
        vec![k("Y", 21, 1.), k("U", 22, 1.), k("I", 23, 1.), k("O", 24, 1.), k("P", 25, 1.), k("Bksp", 14, 1.5)],
        vec![k("H", 35, 1.), k("J", 36, 1.), k("K", 37, 1.), k("L", 38, 1.), k(";", 39, 1.), k("\\", 43, 1.)],
        vec![k("N", 49, 1.), k("M", 50, 1.), k(",", 51, 1.), k(".", 52, 1.), k("/", 53, 1.), k("Shift", 54, 1.)],
    ];
    let thumb = vec![
        k("Ctrl", 29, 1.25), k("Space", 57, 1.5),
        gap(1.5),
        k("Space", 57, 1.5), k("Ctrl", 97, 1.25),
    ];
    build_split(left, right, thumb, 1.5)
}
