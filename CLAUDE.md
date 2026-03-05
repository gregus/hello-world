# CLAUDE.md — mermaid-to-ppt

## Project overview

Rust CLI that converts Mermaid diagram files to PowerPoint (`.pptx`). Output uses **native OOXML shapes only** — no SVG, no raster images, no embedded renderers. Every shape is editable in PowerPoint.

Currently only Gantt charts are implemented. The architecture is designed for additional diagram types to be added as separate modules.

## Build & run

```sh
cargo build                        # debug
cargo build --release              # release → target/release/mermaid-to-ppt
cargo run -- example.mmd           # run against example
cargo run -- example.mmd -o out.pptx
cargo test                         # unit tests
```

## Repository layout

```
src/
  main.rs          CLI (clap), reads .mmd, dispatches by DiagramKind
  gantt.rs         Gantt IR (GanttData, ResolvedTask), date/duration parsing
  pptx_gantt.rs    Gantt → PPTX slide builder (shapes, layout, colours)
vendor/
  ppt-rs/          Vendored ppt-rs with one upstream fix (see below)
example.mmd        Sample Gantt diagram for manual testing
Cargo.toml         Dependencies; ppt-rs pointed at vendor/ppt-rs
```

## Architecture

### Data flow

```
.mmd file
  → mermaid_rs_renderer::parse_mermaid()  (upstream parser)
  → Graph { gantt_tasks, gantt_title, … }
  → gantt::GanttData::from_graph()        (our IR)
  → GanttData { title, tasks: Vec<ResolvedTask>, start_day, end_day }
  → pptx_gantt::save_gantt_pptx()         (PPTX builder)
  → .pptx file
```

### Key types (`gantt.rs`)

- **`GanttData`** — top-level IR; holds resolved tasks and the overall date range.
- **`ResolvedTask`** — a single task with absolute `start_day` / `end_day` (days since Unix epoch), `section`, `label`, and `TaskStatus`.
- **`TaskStatus`** — `Default | Done | Active | Crit | Milestone`.
- Date arithmetic uses epoch-based integer days (`days_from_civil` / `civil_from_days`); no `chrono` dependency in our code.

### PPTX generation (`pptx_gantt.rs`)

All measurements are in inches; converted to EMU (`× 914_400`) when calling `ppt-rs`.

Layout constants:

| Constant | Value | Purpose |
|----------|-------|---------|
| `SLIDE_W_IN` | 10.0 | Slide width |
| `SLIDE_H_IN` | 7.5 | Slide height |
| `TITLE_H_IN` | 0.75 | Reserved for title |
| `HEADER_H_IN` | 0.35 | Date-tick header row |
| `LABEL_W_IN` | 2.5 | Left label column |
| `ROW_H_IN` | 0.38 | Height per task row |
| `BAR_MARGIN_IN` | 0.06 | Vertical padding inside bar |
| `TICK_COUNT` | 5 | Number of date ticks |

Shape types used:

| Shape | OOXML preset | Used for |
|-------|-------------|----------|
| `ShapeType::Rectangle` | `rect` | Background, header, task bars, labels, grid lines |
| `ShapeType::Diamond` | `diamond` | Milestone markers |

Colour palettes cycle by section index (`SECTION_BG`, `SECTION_HDR`). Task bar colour is determined by `TaskStatus` via `bar_color()`.

## Vendored dependency: `ppt-rs`

`ppt-rs` is vendored at `vendor/ppt-rs` because the upstream crate has a compilation bug: a `Commands::Web2Ppt` match arm in `src/cli/parser.rs` is not guarded by `#[cfg(feature = "web2ppt")]`, causing a compile error when the feature is disabled.

**The fix applied** (`vendor/ppt-rs/src/cli/parser.rs`):
```rust
// Added #[cfg(feature = "web2ppt")] before the match arm at line ~394
#[cfg(feature = "web2ppt")]
Commands::Web2Ppt { … } => { … }
```

If `ppt-rs` is updated upstream to fix this, the vendored copy can be replaced with the git dependency:
```toml
ppt-rs = { git = "https://github.com/yingkitw/ppt-rs", default-features = false }
```

## Adding a new diagram type

1. Create `src/<type>.rs` — parse `Graph` into an IR struct.
2. Create `src/pptx_<type>.rs` — render IR to a `SlideContent` using `ppt-rs` shapes.
3. Add a `DiagramKind::<Type>` arm to the `match` in `main.rs`.

Keep parsing and rendering in separate modules (matching the `gantt` / `pptx_gantt` split).

## Output requirements

- **No SVG.** Never use `ppt-rs` image/media APIs to embed a rendered diagram. All visual elements must be native `p:sp` shapes with `a:prstGeom`.
- **No raster images.** Same reason — shapes must remain editable.
- Shapes should have explicit fill colours (`ShapeFill::new(hex)`) and, where appropriate, a thin border (`ShapeLine::new("000000", 9525)`).

## Testing

```sh
cargo test          # unit tests in gantt.rs (date round-trips, duration parsing)
cargo run -- example.mmd -o /tmp/test.pptx   # smoke test
```

To verify no SVG is present in the output:
```sh
python3 -c "
import zipfile
with zipfile.ZipFile('/tmp/test.pptx') as z:
    xml = z.read('ppt/slides/slide1.xml').decode()
    print('SVG elements:', xml.count('<svg'))
    print('p:sp shapes:', xml.count('<p:sp>'))
"
```
