//! 2D Screen-space HUD and telemetry overlay graphics pipeline.

use bytemuck::{Pod, Zeroable};

/// 2D HUD Vertex in pixel screen-space coordinates.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct HudVertex {
    /// Screen-space position in pixels (X = right from 0, Y = down from 0).
    pub position: [f32; 2],
    /// RGBA vertex color.
    pub color: [f32; 4],
}

impl HudVertex {
    pub const fn new(position: [f32; 2], color: [f32; 4]) -> Self {
        Self { position, color }
    }
}

/// Dynamic 2D mesh container for HUD panels, gauges, text, and telemetry indicators.
#[derive(Debug, Clone, Default)]
pub struct HudMesh {
    pub vertices: Vec<HudVertex>,
    pub indices: Vec<u32>,
}

impl HudMesh {
    pub fn new() -> Self {
        Self {
            vertices: Vec::with_capacity(1024),
            indices: Vec::with_capacity(2048),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty() || self.indices.is_empty()
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
    }

    /// Adds a filled rectangle.
    pub fn add_rect(&mut self, x: f32, y: f32, w: f32, h: f32, color: [f32; 4]) {
        if w <= 0.0 || h <= 0.0 {
            return;
        }
        let base_idx = self.vertices.len() as u32;
        self.vertices.push(HudVertex::new([x, y], color));
        self.vertices.push(HudVertex::new([x + w, y], color));
        self.vertices.push(HudVertex::new([x + w, y + h], color));
        self.vertices.push(HudVertex::new([x, y + h], color));

        self.indices.extend_from_slice(&[
            base_idx,
            base_idx + 1,
            base_idx + 2,
            base_idx,
            base_idx + 2,
            base_idx + 3,
        ]);
    }

    /// Adds a hollow rectangular border.
    pub fn add_rect_outline(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        thickness: f32,
        color: [f32; 4],
    ) {
        // Top
        self.add_rect(x, y, w, thickness, color);
        // Bottom
        self.add_rect(x, y + h - thickness, w, thickness, color);
        // Left
        self.add_rect(x, y + thickness, thickness, h - 2.0 * thickness, color);
        // Right
        self.add_rect(
            x + w - thickness,
            y + thickness,
            thickness,
            h - 2.0 * thickness,
            color,
        );
    }

    /// Adds a telemetry progress / meter bar with background and fill.
    #[allow(clippy::too_many_arguments)]
    pub fn add_bar(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        progress: f32,
        fill_color: [f32; 4],
        bg_color: [f32; 4],
    ) {
        // Background
        self.add_rect(x, y, w, h, bg_color);
        // Fill
        let clamped = progress.clamp(0.0, 1.0);
        if clamped > 0.0 {
            self.add_rect(x, y, w * clamped, h, fill_color);
        }
        // Border
        self.add_rect_outline(x, y, w, h, 1.0, [1.0, 1.0, 1.0, 0.4]);
    }

    /// Adds a line segment between two points with given thickness.
    pub fn add_line(
        &mut self,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        thickness: f32,
        color: [f32; 4],
    ) {
        let dx = x2 - x1;
        let dy = y2 - y1;
        let len = (dx * dx + dy * dy).sqrt();
        if len < 1e-4 {
            return;
        }
        let nx = (-dy / len) * (thickness * 0.5);
        let ny = (dx / len) * (thickness * 0.5);

        let base_idx = self.vertices.len() as u32;
        self.vertices
            .push(HudVertex::new([x1 + nx, y1 + ny], color));
        self.vertices
            .push(HudVertex::new([x2 + nx, y2 + ny], color));
        self.vertices
            .push(HudVertex::new([x2 - nx, y2 - ny], color));
        self.vertices
            .push(HudVertex::new([x1 - nx, y1 - ny], color));

        self.indices.extend_from_slice(&[
            base_idx,
            base_idx + 1,
            base_idx + 2,
            base_idx,
            base_idx + 2,
            base_idx + 3,
        ]);
    }

    /// Adds a circular dial outline (e.g. for compass or speedometer).
    pub fn add_circle_outline(
        &mut self,
        cx: f32,
        cy: f32,
        radius: f32,
        thickness: f32,
        segments: usize,
        color: [f32; 4],
    ) {
        let n = segments.max(12);
        for i in 0..n {
            let theta1 = (i as f32 / n as f32) * std::f32::consts::TAU;
            let theta2 = ((i + 1) as f32 / n as f32) * std::f32::consts::TAU;
            let x1 = cx + theta1.cos() * radius;
            let y1 = cy + theta1.sin() * radius;
            let x2 = cx + theta2.cos() * radius;
            let y2 = cy + theta2.sin() * radius;
            self.add_line(x1, y1, x2, y2, thickness, color);
        }
    }

    /// Renders a single ASCII character using built-in 5x7 bitmap font.
    pub fn add_char(&mut self, x: f32, y: f32, scale: f32, ch: char, color: [f32; 4]) {
        let glyph = get_glyph(ch);
        for (col, &bits) in glyph.iter().enumerate() {
            for row in 0..7 {
                if (bits & (1 << row)) != 0 {
                    self.add_rect(
                        x + (col as f32) * scale,
                        y + (row as f32) * scale,
                        scale,
                        scale,
                        color,
                    );
                }
            }
        }
    }

    /// Renders a multi-line ASCII string.
    pub fn add_text(
        &mut self,
        start_x: f32,
        start_y: f32,
        scale: f32,
        text: &str,
        color: [f32; 4],
    ) {
        let mut cur_x = start_x;
        let mut cur_y = start_y;
        let char_w = 6.0 * scale;
        let line_h = 9.0 * scale;

        for ch in text.chars() {
            if ch == '\n' {
                cur_x = start_x;
                cur_y += line_h;
            } else {
                self.add_char(cur_x, cur_y, scale, ch, color);
                cur_x += char_w;
            }
        }
    }

    /// Convenience: adds a stylish semi-transparent panel with a bright accent top border.
    pub fn add_panel(&mut self, x: f32, y: f32, w: f32, h: f32, accent_color: [f32; 4]) {
        // Dark glass background
        self.add_rect(x, y, w, h, [0.05, 0.07, 0.12, 0.85]);
        // Soft border
        self.add_rect_outline(x, y, w, h, 1.0, [0.25, 0.35, 0.50, 0.40]);
        // Accent top bar
        self.add_rect(x, y, w, 2.5, accent_color);
    }
}

/// 5x7 bitmap font table for printable ASCII (32 ' ' through 126 '~').
/// Each character is represented by 5 column bytes; bit 0 is top row, bit 6 is bottom row.
fn get_glyph(ch: char) -> [u8; 5] {
    let ascii = ch as usize;
    if !(32..=126).contains(&ascii) {
        return [0x00, 0x00, 0x00, 0x00, 0x00];
    }
    match ch {
        ' ' => [0x00, 0x00, 0x00, 0x00, 0x00],
        '!' => [0x00, 0x00, 0x5f, 0x00, 0x00],
        '"' => [0x00, 0x07, 0x00, 0x07, 0x00],
        '#' => [0x14, 0x7f, 0x14, 0x7f, 0x14],
        '$' => [0x24, 0x2a, 0x7f, 0x2a, 0x12],
        '%' => [0x23, 0x13, 0x08, 0x64, 0x62],
        '&' => [0x36, 0x49, 0x55, 0x22, 0x50],
        '\'' => [0x00, 0x05, 0x03, 0x00, 0x00],
        '(' => [0x00, 0x1c, 0x22, 0x41, 0x00],
        ')' => [0x00, 0x41, 0x22, 0x1c, 0x00],
        '*' => [0x14, 0x08, 0x3e, 0x08, 0x14],
        '+' => [0x08, 0x08, 0x3e, 0x08, 0x08],
        ',' => [0x00, 0x50, 0x30, 0x00, 0x00],
        '-' => [0x08, 0x08, 0x08, 0x08, 0x08],
        '.' => [0x00, 0x60, 0x60, 0x00, 0x00],
        '/' => [0x20, 0x10, 0x08, 0x04, 0x02],
        '0' => [0x3e, 0x51, 0x49, 0x45, 0x3e],
        '1' => [0x00, 0x42, 0x7f, 0x40, 0x00],
        '2' => [0x42, 0x61, 0x51, 0x49, 0x46],
        '3' => [0x21, 0x41, 0x45, 0x4b, 0x31],
        '4' => [0x18, 0x14, 0x12, 0x7f, 0x10],
        '5' => [0x27, 0x45, 0x45, 0x45, 0x39],
        '6' => [0x3c, 0x4a, 0x49, 0x49, 0x30],
        '7' => [0x01, 0x71, 0x09, 0x05, 0x03],
        '8' => [0x36, 0x49, 0x49, 0x49, 0x36],
        '9' => [0x06, 0x49, 0x49, 0x29, 0x1e],
        ':' => [0x00, 0x36, 0x36, 0x00, 0x00],
        ';' => [0x00, 0x56, 0x36, 0x00, 0x00],
        '<' => [0x08, 0x14, 0x22, 0x41, 0x00],
        '=' => [0x14, 0x14, 0x14, 0x14, 0x14],
        '>' => [0x00, 0x41, 0x22, 0x14, 0x08],
        '?' => [0x02, 0x01, 0x51, 0x09, 0x06],
        '@' => [0x32, 0x49, 0x79, 0x41, 0x3e],
        'A' => [0x7e, 0x11, 0x11, 0x11, 0x7e],
        'B' => [0x7f, 0x49, 0x49, 0x49, 0x36],
        'C' => [0x3e, 0x41, 0x41, 0x41, 0x22],
        'D' => [0x7f, 0x41, 0x41, 0x22, 0x1c],
        'E' => [0x7f, 0x49, 0x49, 0x49, 0x41],
        'F' => [0x7f, 0x09, 0x09, 0x09, 0x01],
        'G' => [0x3e, 0x41, 0x49, 0x49, 0x7a],
        'H' => [0x7f, 0x08, 0x08, 0x08, 0x7f],
        'I' => [0x00, 0x41, 0x7f, 0x41, 0x00],
        'J' => [0x20, 0x40, 0x41, 0x3f, 0x01],
        'K' => [0x7f, 0x08, 0x14, 0x22, 0x41],
        'L' => [0x7f, 0x40, 0x40, 0x40, 0x40],
        'M' => [0x7f, 0x02, 0x0c, 0x02, 0x7f],
        'N' => [0x7f, 0x04, 0x08, 0x10, 0x7f],
        'O' => [0x3e, 0x41, 0x41, 0x41, 0x3e],
        'P' => [0x7f, 0x09, 0x09, 0x09, 0x06],
        'Q' => [0x3e, 0x41, 0x51, 0x21, 0x5e],
        'R' => [0x7f, 0x09, 0x19, 0x29, 0x46],
        'S' => [0x46, 0x49, 0x49, 0x49, 0x31],
        'T' => [0x01, 0x01, 0x7f, 0x01, 0x01],
        'U' => [0x3f, 0x40, 0x40, 0x40, 0x3f],
        'V' => [0x1f, 0x20, 0x40, 0x20, 0x1f],
        'W' => [0x7f, 0x20, 0x18, 0x20, 0x7f],
        'X' => [0x63, 0x14, 0x08, 0x14, 0x63],
        'Y' => [0x07, 0x08, 0x70, 0x08, 0x07],
        'Z' => [0x61, 0x51, 0x49, 0x45, 0x43],
        '[' => [0x00, 0x7f, 0x41, 0x41, 0x00],
        '\\' => [0x02, 0x04, 0x08, 0x10, 0x20],
        ']' => [0x00, 0x41, 0x41, 0x7f, 0x00],
        '^' => [0x04, 0x02, 0x01, 0x02, 0x04],
        '_' => [0x40, 0x40, 0x40, 0x40, 0x40],
        '`' => [0x00, 0x01, 0x02, 0x04, 0x00],
        'a' => [0x20, 0x54, 0x54, 0x54, 0x78],
        'b' => [0x7f, 0x48, 0x44, 0x44, 0x38],
        'c' => [0x38, 0x44, 0x44, 0x44, 0x20],
        'd' => [0x38, 0x44, 0x44, 0x48, 0x7f],
        'e' => [0x38, 0x54, 0x54, 0x54, 0x18],
        'f' => [0x08, 0x7e, 0x09, 0x01, 0x02],
        'g' => [0x0c, 0x52, 0x52, 0x52, 0x3e],
        'h' => [0x7f, 0x08, 0x04, 0x04, 0x78],
        'i' => [0x00, 0x44, 0x7d, 0x40, 0x00],
        'j' => [0x20, 0x40, 0x44, 0x3d, 0x00],
        'k' => [0x7f, 0x10, 0x28, 0x44, 0x00],
        'l' => [0x00, 0x41, 0x7f, 0x40, 0x00],
        'm' => [0x7c, 0x04, 0x18, 0x04, 0x78],
        'n' => [0x7c, 0x08, 0x04, 0x04, 0x78],
        'o' => [0x38, 0x44, 0x44, 0x44, 0x38],
        'p' => [0x7c, 0x14, 0x14, 0x14, 0x08],
        'q' => [0x08, 0x14, 0x14, 0x18, 0x7c],
        'r' => [0x7c, 0x08, 0x04, 0x04, 0x08],
        's' => [0x48, 0x54, 0x54, 0x54, 0x20],
        't' => [0x04, 0x3f, 0x44, 0x40, 0x20],
        'u' => [0x3c, 0x40, 0x40, 0x20, 0x7c],
        'v' => [0x1c, 0x20, 0x40, 0x20, 0x1c],
        'w' => [0x3c, 0x40, 0x30, 0x40, 0x3c],
        'x' => [0x44, 0x28, 0x10, 0x28, 0x44],
        'y' => [0x0c, 0x50, 0x50, 0x50, 0x3c],
        'z' => [0x44, 0x64, 0x54, 0x4c, 0x44],
        '{' => [0x00, 0x08, 0x36, 0x41, 0x00],
        '|' => [0x00, 0x00, 0x7f, 0x00, 0x00],
        '}' => [0x00, 0x41, 0x36, 0x08, 0x00],
        '~' => [0x08, 0x04, 0x08, 0x10, 0x08],
        _ => [0x7f, 0x7f, 0x7f, 0x7f, 0x7f],
    }
}
