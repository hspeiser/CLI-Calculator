**Calculator**

## Overview

A fast, unit-aware calculator with a minimal, keyboard- and mouse-friendly TUI, a reusable core engine, and optional GUI. It supports physical units, conversions, complex numbers, trigonometry (radians and degrees), implicit multiplication, user functions, and more.

## Highlights

- **Units and conversions**: SI and common imperial units (e.g., `m`, `s`, `kg`, `V`, `Ω`, `in`, `ft`, prefixes like `k`, `m`, `μ`). Convert via `to` (e.g., `10 m to in`).
- **Angles**: radians (`rad`) canonical; `deg`/`°` scale correctly; trig accepts radians, and `*_deg` variants accept degrees.
- **Complex numbers**: use `i`/`j` or `i()`; arithmetic and parallel operator work with complex quantities.
- **Implicit multiplication**: `10x`, `2(3+4)`, `x y`.
- **Parallel operator**: `//` for electrical parallel combination.
- **TUI**: split view (input left, output right), line numbers, minimal highlighting, mouse navigation, visible cursor, and `Ans` for previous result.

## Project layout

- `calc_core`: core library (lexer, parser, IR/VM, units, registry)
- `calc_cli`: terminal UI (TUI) built with `ratatui`
- `calc_gui`: GUI starter (optional)

## Prerequisites

- Rust (stable) and Cargo
- Optional: Node.js (for the bundled Jest setup)

## Build & run

- **CLI**
  - Run: `cargo run -p calc_cli`
  - Open a file: `cargo run -p calc_cli -- --file sample.calc`

- **GUI (optional)**
  - Run: `cargo run -p calc_gui`

## Using the CLI

- Left pane: type one expression per line. Right pane: live results line-by-line.
- Line numbers on both sides. Errors display in red when they are runtime errors.
- Minimal syntax highlighting for numbers, units, and built-in functions.
- Mouse: click to focus a pane; click in input to move the cursor.
- Keyboard:
  - Typing inserts at the cursor; Enter splits the line; Delete/Backspace edit normally.
  - Backspace at column 0 merges the current line into the previous line and moves the cursor to the end of the previous line.
  - Arrow keys move the cursor; Tab switches focus between panes; Esc or Ctrl+C exits.
- `Ans`: replaces occurrences with the previous displayed result within the current evaluation pass.

## Language features (quick tour)

```text
# Numbers and implicit multiplication
x = 10
10x            # 100
2(3 + 4)       # 14

# Units and conversions
d = 10 m
d to in        # inches

# Trig (radians) and degree variants
sin(pi()/2)    # 1
cos_deg(60)    # 0.5
sin(90 °)      # 1

# Complex numbers
y = 1/10 i
y = y * 10
10 + y         # complex result

# Parallel operator
r1 = 10kΩ
r2 = 15kΩ
r_eq = r1 // r2

# User function
Zc(f, C) = -1 * i() / (2*pi() * f * C)
Z = 100 Ω // Zc(1000 Hz, 100 nF)
```

Notes:
- Angles: `sin`, `cos`, `tan` expect radians. Use `sin_deg`, `cos_deg`, `tan_deg` for degrees.
- Complex: `i`, `j`, `i()` are supported. The engine seeds `i`, `j`, `pi`, and `π`.
- Quantities: addition/subtraction require compatible dimensions and the same canonical unit on both sides.

## Testing

- All Rust tests (workspace):
  - `make test`
  - or `cargo test --workspace --all-targets`

- Core library tests only:
  - `cargo test -p calc_core --lib --tests`

- JS tests (optional):
  - `npm run test`
  - Or everything: `npm run test:all` (runs Rust then Jest)

## Examples

- `sample.calc`, `conv.calc`, `complex.calc` in the repo are good starting points. Run the CLI with `--file <path>` to load them.

## Implementation notes

- The core lowers parsed expressions to a compact IR and evaluates them on a small stack VM.
- Units are tracked with a 7-base-dimension vector and canonicalized for display.
- The CLI expands `Ans` purely on the UI side before evaluation for each line.

## Limitations

- Unit addition/subtraction require identical canonical unit strings (no auto-rescaling across like-units during `+`/`-`). Convert explicitly before adding if needed.
- Diagnostics are minimal; VM type errors may display as strings (e.g., `<type-error:+>`). Runtime errors show as `error: ...` in the UI.

## License

MIT


