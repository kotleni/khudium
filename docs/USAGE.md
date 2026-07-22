# Usage

## Launch
```
khudium [OPTIONS]
```

The overlay reads the current keyboard layout from `localectl` and renders it as a transparent overlay on screen.

## Options
| Flag | Default | Description |
|------|---------|-------------|
| `--scale <SCALE>` | `1.0` | Scale factor for overlay size |
| `--alpha <ALPHA>` | `0.5` | Global opacity (0.0 -- 1.0) |
| `--paddings <PX>` | `4` | Gap between keys in pixels |
| `--place <POS>` | `bottom-right` | Screen anchor position |
| `--fx-keys` | off | Show the function key row |
| `--split <NAME>` | off | Use a split keyboard layout |

## Placement
`--place` accepts any combination of `top`, `bottom`, `left`, `right`:

```
khudium --place top-left
khudium --place bottom-center
```

## Split keyboards
```
khudium --split lily58
khudium --split corne
khudium --split cheapino
```

Each split layout is a self-contained definition with left and right halves rendered as separate blocks.

## Examples
Large, transparent overlay anchored to top-left:

```
khudium --scale 1.5 --alpha 0.3 --place top-left
```

Corne layout, full size, function row visible:

```
khudium --split corne --scale 1.0 --fx-keys
```
