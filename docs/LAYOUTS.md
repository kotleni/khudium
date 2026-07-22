# Adding layouts

## Standard layouts
Standard layouts are defined in `src/layout.rs` as `LayoutDef` constants. Each layout specifies label arrays for the number row and three alpha rows, plus punctuation keys.

```rust
const MY_LAYOUT: LayoutDef = LayoutDef {
    number: NumberRow {
        labels: ["`", "1", "2", "3", "4", "5", "6", "7", "8", "9", "0", "-", "="],
    },
    alpha: AlphaRows {
        top:    ["Q", "W", "E", "R", "T", "Y", "U", "I", "O", "P"],
        home:   ["A", "S", "D", "F", "G", "H", "J", "K", "L"],
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
```

Then register it in `Layouts::get()`:

```rust
"my_layout" => &MY_LAYOUT,
```

### Key codes
Key codes are Linux `input_event.code` values from `input-event-codes.h`. Common ones:

| Row | Codes |
|-----|-------|
| Number row | `41`, `2`--`13` |
| Top alpha (Q--P) | `16`--`25` |
| Home row (A--L) | `30`--`38` |
| Bottom row (Z--M) | `44`--`50` |
| Punct: `[` `]` `\` | `26`, `27`, `43` |
| Punct: `;` `'` | `39`, `40` |
| Punct: `,` `.` `/` | `51`, `52`, `53` |

## Split layouts
Split layouts are built with `build_split()`. Each side is a `Vec<Vec<Key>>` where every row must have the same total width (in key units).

```rust
fn my_split() -> KeyboardLayout {
    let left = vec![
        vec![k("Q", 16, 1.), k("W", 17, 1.), k("E", 18, 1.), k("R", 19, 1.), k("T", 20, 1.)],
        vec![k("A", 30, 1.), k("S", 31, 1.), k("D", 32, 1.), k("F", 33, 1.), k("G", 34, 1.)],
        vec![k("Z", 44, 1.), k("X", 45, 1.), k("C", 46, 1.), k("V", 47, 1.), k("B", 48, 1.)],
    ];
    let right = vec![
        vec![k("Y", 21, 1.), k("U", 22, 1.), k("I", 23, 1.), k("O", 24, 1.), k("P", 25, 1.)],
        vec![k("H", 35, 1.), k("J", 36, 1.), k("K", 37, 1.), k("L", 38, 1.), k(";", 39, 1.)],
        vec![k("N", 49, 1.), k("M", 50, 1.), k(",", 51, 1.), k(".", 52, 1.), k("/", 53, 1.)],
    ];
    let thumb = vec![
        k("Ctrl", 29, 1.), k("Space", 57, 1.),
        gap(1.5),
        k("Space", 57, 1.), k("Ctrl", 97, 1.),
    ];
    build_split(left, right, thumb, 1.5)
}
```

Register it in `get_split_layout()`:

```rust
"my_split" => my_split(),
```

### Rules
- Every row in `left` and `right` must sum to the same width.
- Use `gap(width)` for spacing between halves in the thumb row.
- Use wider keys (e.g. `k("Shift", 42, 2.)`) to pad rows to equal width.
- Thumb row is a single flat `Vec<Key>` with a `gap()` between left and right thumbs.
