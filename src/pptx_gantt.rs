/// Convert a `GanttData` IR into a PowerPoint (.pptx) file using ppt-rs.
///
/// Layout (all measurements in inches unless noted):
///   Slide:       10 × 7.5 inches
///   Title area:  top 0.7 in  (managed by SlideContent title field)
///   Header row:  y = 0.75, h = 0.35  (date tick labels)
///   Task rows:   y = 1.10 + n * ROW_H, h = ROW_H
///   Label col:   x = 0,    w = LABEL_W
///   Timeline:    x = LABEL_W, w = TIMELINE_W

use anyhow::Result;
use ppt_rs::generator::shapes::{Shape, ShapeFill, ShapeLine, ShapeType};
use ppt_rs::generator::slide_content::SlideContent;
use ppt_rs::generator::builder::create_pptx_with_content;
use std::fs;
use std::path::Path;

use crate::gantt::{civil_from_days, format_day, GanttData, ResolvedTask, TaskStatus};

// ── Layout constants (inches) ─────────────────────────────────────────────────

const SLIDE_W_IN: f64 = 10.0;
const SLIDE_H_IN: f64 = 7.5;

const TITLE_H_IN: f64 = 0.75;   // reserved for slide title text
const HEADER_H_IN: f64 = 0.35;  // date-tick header row
const LABEL_W_IN: f64 = 2.5;    // left column (task labels)
const TIMELINE_X_IN: f64 = LABEL_W_IN;
const TIMELINE_W_IN: f64 = SLIDE_W_IN - LABEL_W_IN;
const ROW_H_IN: f64 = 0.38;     // height of each task row
const BAR_MARGIN_IN: f64 = 0.06; // vertical padding inside a task row
const MIN_BAR_W_IN: f64 = 0.05; // minimum visible bar width

const TICK_COUNT: usize = 5; // number of date ticks across the timeline

// ── Colour palettes ───────────────────────────────────────────────────────────

/// Section background colours (light, cycling).
const SECTION_BG: &[&str] = &[
    "D6E4F7", "FCE4D6", "E2EFDA", "FFF2CC",
    "DAE3F3", "F8CBAD", "C6E0B4", "FFE699",
];

/// Section header bar colours (medium, cycling – same hue as bg, darker).
const SECTION_HDR: &[&str] = &[
    "4472C4", "ED7D31", "70AD47", "FFC000",
    "5B9BD5", "E36C09", "548235", "BF8F00",
];

/// Task bar colours for each status.
fn bar_color(status: &TaskStatus, section_idx: usize) -> &'static str {
    match status {
        TaskStatus::Done => "A6A6A6",
        TaskStatus::Active => "4472C4",
        TaskStatus::Crit => "FF0000",
        TaskStatus::Milestone => "FFD700",
        TaskStatus::Default => SECTION_HDR[section_idx % SECTION_HDR.len()],
    }
}

// ── EMU conversion ────────────────────────────────────────────────────────────

#[inline]
fn emu(inches: f64) -> u32 {
    (inches * 914_400.0).round() as u32
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Build and save a PPTX file from Gantt data.
pub fn save_gantt_pptx(data: &GanttData, path: &Path) -> Result<()> {
    let slide = build_slide(data);
    let bytes = create_pptx_with_content(&data.title, vec![slide])?;
    fs::write(path, bytes)?;
    Ok(())
}

// ── Slide building ────────────────────────────────────────────────────────────

fn build_slide(data: &GanttData) -> SlideContent {
    let chart_days = (data.end_day - data.start_day).max(1) as f64;

    // Helpers: convert a day offset to an x position (inches) in the timeline.
    let day_to_x = |day: i32| -> f64 {
        TIMELINE_X_IN + (day - data.start_day) as f64 / chart_days * TIMELINE_W_IN
    };

    let mut slide = SlideContent::new(&data.title);

    // ── Chart background ──────────────────────────────────────────────────────
    slide = slide.add_shape(
        Shape::new(
            ShapeType::Rectangle,
            0,
            emu(TITLE_H_IN),
            emu(SLIDE_W_IN),
            emu(SLIDE_H_IN - TITLE_H_IN),
        )
        .with_fill(ShapeFill::new("F5F5F5")),
    );

    // ── Timeline header area (light grey band) ────────────────────────────────
    let header_y = TITLE_H_IN;
    slide = slide.add_shape(
        Shape::new(
            ShapeType::Rectangle,
            emu(TIMELINE_X_IN),
            emu(header_y),
            emu(TIMELINE_W_IN),
            emu(HEADER_H_IN),
        )
        .with_fill(ShapeFill::new("DDEEFF")),
    );

    // Date tick labels across the header
    slide = add_tick_labels(slide, data.start_day, data.end_day, header_y);

    // ── Group tasks by section ────────────────────────────────────────────────
    let task_groups = group_by_section(&data.tasks);
    let mut row_y = TITLE_H_IN + HEADER_H_IN;

    for (group_idx, (section_name, group_tasks)) in task_groups.iter().enumerate() {
        let sec_color_idx = group_idx % SECTION_BG.len();

        // Section header row (full width, coloured)
        if let Some(name) = section_name {
            slide = add_section_header(slide, name, row_y, sec_color_idx);
            row_y += ROW_H_IN;
        }

        // Task rows
        for task in group_tasks {
            // Row background (alternating tint)
            let bg_hex = SECTION_BG[sec_color_idx];
            slide = slide.add_shape(
                Shape::new(
                    ShapeType::Rectangle,
                    0,
                    emu(row_y),
                    emu(SLIDE_W_IN),
                    emu(ROW_H_IN),
                )
                .with_fill(ShapeFill::new(bg_hex))
                .with_fill(ShapeFill::new(bg_hex).with_transparency(70)),
            );

            // Task label (left column)
            slide = slide.add_shape(
                Shape::new(
                    ShapeType::Rectangle,
                    0,
                    emu(row_y),
                    emu(LABEL_W_IN),
                    emu(ROW_H_IN),
                )
                .with_fill(ShapeFill::new("FFFFFF").with_transparency(80))
                .with_text(&task.label),
            );

            // Timeline bar
            let bar_x = day_to_x(task.start_day);
            let bar_end_x = day_to_x(task.end_day).max(bar_x + MIN_BAR_W_IN);
            let bar_w = (bar_end_x - bar_x).max(MIN_BAR_W_IN);
            let bar_y = row_y + BAR_MARGIN_IN;
            let bar_h = ROW_H_IN - 2.0 * BAR_MARGIN_IN;

            let color = bar_color(&task.status, group_idx);

            if task.status == TaskStatus::Milestone {
                // Render milestone as a small diamond
                slide = slide.add_shape(
                    Shape::new(
                        ShapeType::Diamond,
                        emu(bar_x - ROW_H_IN * 0.4),
                        emu(bar_y),
                        emu(bar_h),
                        emu(bar_h),
                    )
                    .with_fill(ShapeFill::new(color))
                    .with_line(ShapeLine::new("000000", 9525)),
                );
            } else {
                let shape = Shape::new(
                    ShapeType::Rectangle,
                    emu(bar_x),
                    emu(bar_y),
                    emu(bar_w),
                    emu(bar_h),
                )
                .with_fill(ShapeFill::new(color))
                .with_line(ShapeLine::new("000000", 9525));

                // Only add label text if bar is wide enough
                let shape = if bar_w > 0.4 {
                    shape.with_text(&task.label)
                } else {
                    shape
                };

                slide = slide.add_shape(shape);
            }

            row_y += ROW_H_IN;

            // Stop if we're running off the slide
            if row_y + ROW_H_IN > SLIDE_H_IN {
                break;
            }
        }

        if row_y + ROW_H_IN > SLIDE_H_IN {
            break;
        }
    }

    // ── Vertical grid lines at each tick ─────────────────────────────────────
    slide = add_grid_lines(slide, data.start_day, data.end_day, TITLE_H_IN, row_y);

    slide
}

// ── Section header ────────────────────────────────────────────────────────────

fn add_section_header(mut slide: SlideContent, name: &str, y: f64, color_idx: usize) -> SlideContent {
    let hdr_color = SECTION_HDR[color_idx % SECTION_HDR.len()];

    // Full-width coloured background
    slide = slide.add_shape(
        Shape::new(
            ShapeType::Rectangle,
            0,
            emu(y),
            emu(SLIDE_W_IN),
            emu(ROW_H_IN),
        )
        .with_fill(ShapeFill::new(hdr_color))
        .with_text(name),
    );
    slide
}

// ── Tick labels ───────────────────────────────────────────────────────────────

fn add_tick_labels(mut slide: SlideContent, start: i32, end: i32, header_y: f64) -> SlideContent {
    let chart_days = (end - start).max(1) as f64;
    for i in 0..=TICK_COUNT {
        let frac = i as f64 / TICK_COUNT as f64;
        let day = start + (frac * chart_days).round() as i32;
        let x_in = TIMELINE_X_IN + frac * TIMELINE_W_IN;

        let label = short_date(day);
        slide = slide.add_shape(
            Shape::new(
                ShapeType::Rectangle,
                emu(x_in - 0.4),
                emu(header_y),
                emu(0.8),
                emu(HEADER_H_IN),
            )
            .with_fill(ShapeFill::new("DDEEFF"))
            .with_text(&label),
        );
    }
    slide
}

// ── Grid lines ────────────────────────────────────────────────────────────────

fn add_grid_lines(mut slide: SlideContent, start: i32, end: i32, top_y: f64, bottom_y: f64) -> SlideContent {
    let chart_days = (end - start).max(1) as f64;
    let line_h = bottom_y - top_y;

    for i in 0..=TICK_COUNT {
        let frac = i as f64 / TICK_COUNT as f64;
        let x_in = TIMELINE_X_IN + frac * TIMELINE_W_IN;

        slide = slide.add_shape(
            Shape::new(
                ShapeType::Rectangle,
                emu(x_in),
                emu(top_y),
                emu(0.01),      // hairline width
                emu(line_h),
            )
            .with_fill(ShapeFill::new("AAAAAA")),
        );
    }
    slide
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Format a day as a short date label, e.g. "Jan 1" or "2024-01-01".
fn short_date(day: i32) -> String {
    let (y, m, d) = civil_from_days(day);
    let month_abbr = ["Jan","Feb","Mar","Apr","May","Jun",
                      "Jul","Aug","Sep","Oct","Nov","Dec"];
    format!("{} {}\n{}", month_abbr[(m - 1) as usize], d, y)
}

/// Group tasks into (section_name, tasks) preserving order.
fn group_by_section<'a>(tasks: &'a [ResolvedTask]) -> Vec<(Option<String>, Vec<&'a ResolvedTask>)> {
    let mut groups: Vec<(Option<String>, Vec<&'a ResolvedTask>)> = Vec::new();

    for task in tasks {
        let matches = groups
            .last()
            .map(|(s, _)| s == &task.section)
            .unwrap_or(false);

        if matches {
            groups.last_mut().unwrap().1.push(task);
        } else {
            groups.push((task.section.clone(), vec![task]));
        }
    }

    groups
}
