# mermaid-to-ppt

Convert Mermaid diagram files (`.mmd`) to editable PowerPoint presentations (`.pptx`).

All output shapes are **native Office Open XML** — rectangles, diamonds, and text frames — fully editable in PowerPoint, Keynote, and LibreOffice Impress. No SVG, no images, no embedded renderers.

## Supported diagram types

| Type | Status |
|------|--------|
| Gantt chart | ✅ Implemented |
| Flowchart | Planned |
| Sequence diagram | Planned |
| Others | Planned |

## Installation

Requires [Rust](https://rustup.rs/) 1.70+.

```sh
git clone <repo>
cd mermaid-to-ppt
cargo build --release
# binary at target/release/mermaid-to-ppt
```

## Usage

```
mermaid-to-ppt <input.mmd> [-o output.pptx]
```

If `-o` is omitted the output file is placed next to the input with a `.pptx` extension.

```sh
# Basic
mermaid-to-ppt project.mmd

# Explicit output path
mermaid-to-ppt project.mmd -o slides/project-gantt.pptx
```

## Gantt charts

### Input format

Standard Mermaid gantt syntax:

```
gantt
    title Project Alpha — Q1 2025
    dateFormat YYYY-MM-DD

    section Planning
    Define scope         : done, scope, 2025-01-06, 5d
    Stakeholder review   : done, review, after scope, 3d
    Finalise requirements: done, req,    after review, 4d

    section Development
    Backend API          : active, crit, backend, 2025-01-20, 20d
    Frontend UI          : frontend, after backend, 15d
    Integration tests    : tests,   after frontend, 7d

    section Release
    User acceptance test : uat, 2025-03-03, 10d
    Go-live              : milestone, golive, after uat, 1d
```

### Supported syntax

**Date formats**

| Format | Example |
|--------|---------|
| `YYYY-MM-DD` | `2025-01-20` |
| `YYYY/MM/DD` | `2025/01/20` |
| `YYYY.MM.DD` | `2025.01.20` |

**Duration units**

| Unit | Meaning |
|------|---------|
| `d` | days |
| `w` | weeks (7 days) |
| `m` | months (30 days) |
| `y` | years (365 days) |
| `h` | hours (rounded up to 1 day minimum) |

**Task start**

- Explicit date: `: 2025-01-06, 5d`
- After another task: `: after taskId, 5d`
- Implicit (follows previous task in source order)

**Task status / styling**

| Keyword | Rendering |
|---------|-----------|
| *(none)* | Section colour |
| `done` | Gray |
| `active` | Blue |
| `crit` | Red |
| `milestone` | Gold diamond marker |

### Slide layout

```
┌──────────────────────────────────────────────┐
│  Title                                       │  0.75 in
├──────────┬───────────────────────────────────┤
│ Date hdr │  Jan 6   Jan 20  Feb 3  Feb 17   │  0.35 in
├──────────┼───────────────────────────────────┤
│ Planning │  (section header, full width)     │
│ task 1   │  ████████                         │  0.38 in / row
│ task 2   │      ████                         │
├──────────┼───────────────────────────────────┤
│ Develop  │  (section header)                 │
│ ...      │  ...                              │
└──────────┴───────────────────────────────────┘
  2.5 in        7.5 in
```

Slide size: **10 × 7.5 inches** (standard widescreen-compatible).

## Output format

The generated `.pptx` file contains only native OOXML shapes:

- `<a:prstGeom prst="rect">` — background bands, task bars, labels
- `<a:prstGeom prst="diamond">` — milestone markers
- `<p:sp>` text frames — title, date ticks, task labels

Every shape is individually selectable and editable in any PPTX-compatible application.

## Project structure

```
src/
  main.rs          CLI entry point, diagram-type dispatch
  gantt.rs         Mermaid gantt parser → GanttData IR, date/duration arithmetic
  pptx_gantt.rs    GanttData → PPTX slide (shapes, colours, layout)
vendor/
  ppt-rs/          Vendored fork of ppt-rs with upstream compilation fix
example.mmd        Sample gantt diagram (Q1 2025 project)
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| `mermaid-rs-renderer` | Mermaid syntax parser |
| `ppt-rs` (vendored) | OOXML / PPTX ZIP builder |
| `clap` | CLI argument parsing |
| `anyhow` | Error handling |

## Running tests

```sh
cargo test
```

The test suite covers date round-trips, date format variants, and duration unit parsing.
