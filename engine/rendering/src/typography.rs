//! High-definition, proportional antialiased vector typography engine for modern HUD and UI.
//!
//! Replaces tiny, blocky 5x7 bitmap fonts with smooth, legible, beautifully proportioned
//! vector-stroked glyphs with anti-aliased edge feathering and drop shadows.

use crate::hud::{HudMesh, HudVertex};

/// Font size presets in metric screen pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FontSize {
    /// Small label / badge / telemetry caption (11px).
    Caption,
    /// Standard UI body text / list item (14px).
    Body,
    /// Card title / prominent parameter (18px).
    Subhead,
    /// Modal header / section title (24px).
    Title,
    /// Prominent alert / milestone banner (32px).
    Heading,
    /// Speedometer digits / countdown hero display (48px).
    Hero,
    /// Custom pixel height.
    Custom(f32),
}

impl FontSize {
    pub fn pixels(&self) -> f32 {
        match *self {
            Self::Caption => 11.0,
            Self::Body => 14.0,
            Self::Subhead => 18.0,
            Self::Title => 24.0,
            Self::Heading => 32.0,
            Self::Hero => 48.0,
            Self::Custom(px) => px.max(8.0),
        }
    }
}

/// Font weight specifying stroke thickness and visual weight.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontWeight {
    Regular,
    Bold,
    Heavy,
}

impl FontWeight {
    pub fn stroke_ratio(&self) -> f32 {
        match self {
            Self::Regular => 0.09,
            Self::Bold => 0.14,
            Self::Heavy => 0.19,
        }
    }
}

/// A 2D stroke segment within a normalized 12x16 glyph coordinate box.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlyphSegment {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}

impl GlyphSegment {
    pub const fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Self { x1, y1, x2, y2 }
    }
}

const EMPTY_SEG: GlyphSegment = GlyphSegment::new(0.0, 0.0, 0.0, 0.0);

/// Normalized vector glyph definition for proportional font rendering.
#[derive(Debug, Clone, Copy)]
pub struct VectorGlyph {
    /// Horizontal proportional character advance width in normalized units (0..16).
    pub advance: f32,
    /// Active number of stroke segments.
    pub count: u8,
    /// Multi-segment strokes composing this character.
    pub segments: [GlyphSegment; 12],
}

impl VectorGlyph {
    #[inline]
    pub fn segments(&self) -> &[GlyphSegment] {
        &self.segments[0..(self.count as usize).min(12)]
    }
}

macro_rules! glyph {
    ($adv:expr, [ $( ($x1:expr, $y1:expr, $x2:expr, $y2:expr) ),* $(,)? ]) => {{
        #[allow(unused_mut)]
        let mut segs = [EMPTY_SEG; 12];
        #[allow(unused_mut)]
        let mut i = 0usize;
        $(
            if i < 12 {
                segs[i] = GlyphSegment::new($x1, $y1, $x2, $y2);
                i += 1;
            }
        )*
        VectorGlyph {
            advance: $adv,
            count: i as u8,
            segments: segs,
        }
    }};
}

/// Retrieves the proportional vector glyph definition for an ASCII character.
pub fn get_vector_glyph(ch: char) -> VectorGlyph {
    match ch {
        ' ' => glyph!(6.0, []),
        '!' => glyph!(5.0, [(2.5, 1.0, 2.5, 10.5), (2.5, 13.5, 2.5, 15.0)]),
        '"' => glyph!(8.0, [(2.5, 1.0, 2.5, 5.5), (5.5, 1.0, 5.5, 5.5)]),
        '#' => glyph!(
            12.0,
            [
                (4.0, 1.0, 3.0, 15.0),
                (8.0, 1.0, 7.0, 15.0),
                (1.0, 5.0, 11.0, 5.0),
                (1.0, 10.0, 11.0, 10.0)
            ]
        ),
        '$' => glyph!(
            11.0,
            [
                (5.5, 0.0, 5.5, 16.0),
                (9.0, 3.0, 3.0, 3.0),
                (3.0, 3.0, 2.0, 6.0),
                (2.0, 6.0, 9.0, 9.0),
                (9.0, 9.0, 8.0, 13.0),
                (8.0, 13.0, 2.0, 13.0)
            ]
        ),
        '%' => glyph!(
            13.0,
            [
                (10.0, 2.0, 2.0, 14.0),
                (3.0, 3.0, 5.0, 3.0),
                (5.0, 3.0, 5.0, 5.0),
                (5.0, 5.0, 3.0, 5.0),
                (3.0, 5.0, 3.0, 3.0),
                (7.0, 11.0, 9.0, 11.0),
                (9.0, 11.0, 9.0, 13.0),
                (9.0, 13.0, 7.0, 13.0),
                (7.0, 13.0, 7.0, 11.0)
            ]
        ),
        '&' => glyph!(
            12.0,
            [
                (10.0, 14.0, 3.0, 7.0),
                (3.0, 7.0, 5.0, 2.0),
                (5.0, 2.0, 7.0, 4.0),
                (7.0, 4.0, 2.0, 12.0),
                (2.0, 12.0, 6.0, 15.0),
                (6.0, 15.0, 11.0, 11.0)
            ]
        ),
        '\'' => glyph!(4.5, [(2.0, 1.0, 2.0, 5.0)]),
        '(' => glyph!(
            6.0,
            [
                (4.5, 1.0, 2.0, 5.0),
                (2.0, 5.0, 2.0, 11.0),
                (2.0, 11.0, 4.5, 15.0)
            ]
        ),
        ')' => glyph!(
            6.0,
            [
                (1.5, 1.0, 4.0, 5.0),
                (4.0, 5.0, 4.0, 11.0),
                (4.0, 11.0, 1.5, 15.0)
            ]
        ),
        '*' => glyph!(
            8.0,
            [
                (4.0, 2.0, 4.0, 10.0),
                (1.0, 4.0, 7.0, 8.0),
                (1.0, 8.0, 7.0, 4.0)
            ]
        ),
        '+' => glyph!(10.0, [(5.0, 3.0, 5.0, 13.0), (1.0, 8.0, 9.0, 8.0)]),
        ',' => glyph!(5.0, [(2.5, 13.0, 1.5, 16.5)]),
        '-' => glyph!(8.0, [(1.5, 8.0, 6.5, 8.0)]),
        '.' => glyph!(5.0, [(2.5, 13.5, 2.5, 15.0)]),
        '/' => glyph!(9.0, [(7.5, 1.0, 1.5, 15.0)]),

        // --- DIGITS (Clean modern tabular numbers with curves) ---
        '0' => glyph!(
            10.5,
            [
                (3.0, 1.0, 7.5, 1.0),
                (7.5, 1.0, 9.5, 4.0),
                (9.5, 4.0, 9.5, 12.0),
                (9.5, 12.0, 7.5, 15.0),
                (7.5, 15.0, 3.0, 15.0),
                (3.0, 15.0, 1.0, 12.0),
                (1.0, 12.0, 1.0, 4.0),
                (1.0, 4.0, 3.0, 1.0),
                (8.0, 3.0, 2.5, 13.0)
            ]
        ),
        '1' => glyph!(
            9.0,
            [
                (2.0, 4.0, 5.5, 1.0),
                (5.5, 1.0, 5.5, 15.0),
                (2.0, 15.0, 9.0, 15.0)
            ]
        ),
        '2' => glyph!(
            10.5,
            [
                (2.0, 4.0, 3.5, 1.0),
                (3.5, 1.0, 7.5, 1.0),
                (7.5, 1.0, 9.5, 3.5),
                (9.5, 3.5, 9.5, 6.0),
                (9.5, 6.0, 1.5, 15.0),
                (1.5, 15.0, 9.5, 15.0)
            ]
        ),
        '3' => glyph!(
            10.5,
            [
                (1.5, 1.0, 9.5, 1.0),
                (9.5, 1.0, 5.0, 7.5),
                (5.0, 7.5, 8.5, 8.5),
                (8.5, 8.5, 9.5, 11.5),
                (9.5, 11.5, 7.5, 15.0),
                (7.5, 15.0, 2.5, 15.0),
                (2.5, 15.0, 1.5, 13.5)
            ]
        ),
        '4' => glyph!(
            10.5,
            [
                (7.5, 1.0, 1.5, 10.0),
                (1.5, 10.0, 9.5, 10.0),
                (7.5, 1.0, 7.5, 15.0)
            ]
        ),
        '5' => glyph!(
            10.5,
            [
                (9.0, 1.0, 2.0, 1.0),
                (2.0, 1.0, 1.5, 7.0),
                (1.5, 7.0, 7.5, 7.0),
                (7.5, 7.0, 9.5, 9.5),
                (9.5, 9.5, 9.5, 12.5),
                (9.5, 12.5, 7.0, 15.0),
                (7.0, 15.0, 1.5, 15.0)
            ]
        ),
        '6' => glyph!(
            10.5,
            [
                (8.0, 1.5, 4.0, 1.5),
                (4.0, 1.5, 1.5, 5.5),
                (1.5, 5.5, 1.5, 12.5),
                (1.5, 12.5, 4.0, 15.0),
                (4.0, 15.0, 7.5, 15.0),
                (7.5, 15.0, 9.5, 12.5),
                (9.5, 12.5, 9.5, 9.5),
                (9.5, 9.5, 7.5, 7.5),
                (7.5, 7.5, 1.5, 7.5)
            ]
        ),
        '7' => glyph!(
            10.5,
            [
                (1.5, 1.0, 9.5, 1.0),
                (9.5, 1.0, 4.0, 15.0),
                (3.0, 7.5, 7.0, 7.5)
            ]
        ),
        '8' => glyph!(
            10.5,
            [
                (3.5, 1.0, 7.0, 1.0),
                (7.0, 1.0, 9.0, 3.5),
                (9.0, 3.5, 6.5, 7.5),
                (6.5, 7.5, 9.5, 11.5),
                (9.5, 11.5, 7.5, 15.0),
                (7.5, 15.0, 3.0, 15.0),
                (3.0, 15.0, 1.0, 11.5),
                (1.0, 11.5, 4.0, 7.5),
                (4.0, 7.5, 1.5, 3.5),
                (1.5, 3.5, 3.5, 1.0)
            ]
        ),
        '9' => glyph!(
            10.5,
            [
                (9.0, 8.5, 3.5, 8.5),
                (3.5, 8.5, 1.5, 6.5),
                (1.5, 6.5, 1.5, 3.5),
                (1.5, 3.5, 3.5, 1.0),
                (3.5, 1.0, 7.0, 1.0),
                (7.0, 1.0, 9.5, 3.5),
                (9.5, 3.5, 9.5, 12.0),
                (9.5, 12.0, 6.5, 15.0),
                (6.5, 15.0, 2.5, 14.5)
            ]
        ),
        ':' => glyph!(5.0, [(2.5, 5.0, 2.5, 6.5), (2.5, 11.5, 2.5, 13.0)]),
        ';' => glyph!(5.0, [(2.5, 5.0, 2.5, 6.5), (2.5, 12.0, 1.5, 15.5)]),
        '<' => glyph!(9.0, [(7.5, 3.0, 1.5, 8.0), (1.5, 8.0, 7.5, 13.0)]),
        '=' => glyph!(10.0, [(1.5, 5.5, 8.5, 5.5), (1.5, 10.5, 8.5, 10.5)]),
        '>' => glyph!(9.0, [(1.5, 3.0, 7.5, 8.0), (7.5, 8.0, 1.5, 13.0)]),
        '?' => glyph!(
            9.0,
            [
                (2.0, 3.5, 4.0, 1.0),
                (4.0, 1.0, 6.5, 1.0),
                (6.5, 1.0, 8.0, 3.0),
                (8.0, 3.0, 5.5, 6.5),
                (5.5, 6.5, 5.5, 9.5),
                (5.5, 13.0, 5.5, 14.5)
            ]
        ),
        '@' => glyph!(
            13.0,
            [
                (11.0, 12.0, 11.0, 5.0),
                (11.0, 5.0, 7.0, 1.5),
                (7.0, 1.5, 3.0, 3.5),
                (3.0, 3.5, 1.5, 8.5),
                (1.5, 8.5, 3.5, 14.0),
                (3.5, 14.0, 8.5, 14.0),
                (8.5, 14.0, 9.5, 9.0),
                (9.5, 9.0, 5.5, 9.0),
                (5.5, 9.0, 5.5, 6.5),
                (5.5, 6.5, 8.5, 6.5),
                (8.5, 6.5, 8.5, 11.0)
            ]
        ),

        // --- UPPERCASE ALPHABET ---
        'A' => glyph!(
            11.0,
            [
                (5.5, 1.0, 1.0, 15.0),
                (5.5, 1.0, 10.0, 15.0),
                (2.8, 10.0, 8.2, 10.0)
            ]
        ),
        'B' => glyph!(
            11.0,
            [
                (2.0, 1.0, 2.0, 15.0),
                (2.0, 1.0, 7.5, 1.0),
                (7.5, 1.0, 9.5, 3.5),
                (9.5, 3.5, 7.5, 7.5),
                (7.5, 7.5, 2.0, 7.5),
                (7.5, 7.5, 10.0, 10.5),
                (10.0, 10.5, 8.0, 15.0),
                (8.0, 15.0, 2.0, 15.0)
            ]
        ),
        'C' => glyph!(
            11.5,
            [
                (9.5, 4.0, 7.0, 1.0),
                (7.0, 1.0, 4.0, 1.0),
                (4.0, 1.0, 1.5, 4.5),
                (1.5, 4.5, 1.5, 11.5),
                (1.5, 11.5, 4.0, 15.0),
                (4.0, 15.0, 7.5, 15.0),
                (7.5, 15.0, 10.0, 12.0)
            ]
        ),
        'D' => glyph!(
            11.5,
            [
                (2.0, 1.0, 2.0, 15.0),
                (2.0, 1.0, 6.5, 1.0),
                (6.5, 1.0, 10.0, 5.0),
                (10.0, 5.0, 10.0, 11.0),
                (10.0, 11.0, 6.5, 15.0),
                (6.5, 15.0, 2.0, 15.0)
            ]
        ),
        'E' => glyph!(
            10.0,
            [
                (2.0, 1.0, 2.0, 15.0),
                (2.0, 1.0, 9.0, 1.0),
                (2.0, 8.0, 7.5, 8.0),
                (2.0, 15.0, 9.0, 15.0)
            ]
        ),
        'F' => glyph!(
            9.5,
            [
                (2.0, 1.0, 2.0, 15.0),
                (2.0, 1.0, 9.0, 1.0),
                (2.0, 7.5, 7.5, 7.5)
            ]
        ),
        'G' => glyph!(
            11.5,
            [
                (9.5, 4.0, 7.0, 1.0),
                (7.0, 1.0, 4.0, 1.0),
                (4.0, 1.0, 1.5, 4.5),
                (1.5, 4.5, 1.5, 11.5),
                (1.5, 11.5, 4.0, 15.0),
                (4.0, 15.0, 8.0, 15.0),
                (8.0, 15.0, 10.0, 12.0),
                (10.0, 12.0, 10.0, 8.5),
                (10.0, 8.5, 6.0, 8.5)
            ]
        ),
        'H' => glyph!(
            11.5,
            [
                (2.0, 1.0, 2.0, 15.0),
                (9.5, 1.0, 9.5, 15.0),
                (2.0, 8.0, 9.5, 8.0)
            ]
        ),
        'I' => glyph!(
            6.0,
            [
                (3.0, 1.0, 3.0, 15.0),
                (1.0, 1.0, 5.0, 1.0),
                (1.0, 15.0, 5.0, 15.0)
            ]
        ),
        'J' => glyph!(
            9.0,
            [
                (2.5, 1.0, 7.5, 1.0),
                (7.0, 1.0, 7.0, 12.0),
                (7.0, 12.0, 5.0, 15.0),
                (5.0, 15.0, 2.0, 15.0),
                (2.0, 15.0, 1.0, 12.5)
            ]
        ),
        'K' => glyph!(
            11.0,
            [
                (2.0, 1.0, 2.0, 15.0),
                (9.5, 1.0, 2.5, 8.0),
                (4.5, 6.5, 10.0, 15.0)
            ]
        ),
        'L' => glyph!(9.5, [(2.0, 1.0, 2.0, 15.0), (2.0, 15.0, 8.5, 15.0)]),
        'M' => glyph!(
            13.5,
            [
                (1.5, 15.0, 1.5, 1.0),
                (1.5, 1.0, 6.75, 10.5),
                (6.75, 10.5, 12.0, 1.0),
                (12.0, 1.0, 12.0, 15.0)
            ]
        ),
        'N' => glyph!(
            11.5,
            [
                (2.0, 15.0, 2.0, 1.0),
                (2.0, 1.0, 9.5, 15.0),
                (9.5, 15.0, 9.5, 1.0)
            ]
        ),
        'O' => glyph!(
            11.5,
            [
                (4.0, 1.0, 7.5, 1.0),
                (7.5, 1.0, 10.0, 4.5),
                (10.0, 4.5, 10.0, 11.5),
                (10.0, 11.5, 7.5, 15.0),
                (7.5, 15.0, 4.0, 15.0),
                (4.0, 15.0, 1.5, 11.5),
                (1.5, 11.5, 1.5, 4.5),
                (1.5, 4.5, 4.0, 1.0)
            ]
        ),
        'P' => glyph!(
            10.5,
            [
                (2.0, 1.0, 2.0, 15.0),
                (2.0, 1.0, 7.0, 1.0),
                (7.0, 1.0, 9.5, 3.5),
                (9.5, 3.5, 9.5, 6.5),
                (9.5, 6.5, 7.0, 9.0),
                (7.0, 9.0, 2.0, 9.0)
            ]
        ),
        'Q' => glyph!(
            11.5,
            [
                (4.0, 1.0, 7.5, 1.0),
                (7.5, 1.0, 10.0, 4.5),
                (10.0, 4.5, 10.0, 11.5),
                (10.0, 11.5, 7.5, 15.0),
                (7.5, 15.0, 4.0, 15.0),
                (4.0, 15.0, 1.5, 11.5),
                (1.5, 11.5, 1.5, 4.5),
                (1.5, 4.5, 4.0, 1.0),
                (7.0, 11.5, 11.0, 16.5)
            ]
        ),
        'R' => glyph!(
            11.0,
            [
                (2.0, 1.0, 2.0, 15.0),
                (2.0, 1.0, 7.0, 1.0),
                (7.0, 1.0, 9.5, 3.5),
                (9.5, 3.5, 7.0, 7.5),
                (7.0, 7.5, 2.0, 7.5),
                (5.5, 7.5, 10.0, 15.0)
            ]
        ),
        'S' => glyph!(
            10.5,
            [
                (9.0, 3.5, 6.5, 1.0),
                (6.5, 1.0, 3.5, 1.0),
                (3.5, 1.0, 1.5, 3.5),
                (1.5, 3.5, 2.0, 6.5),
                (2.0, 6.5, 7.5, 8.5),
                (7.5, 8.5, 9.5, 11.0),
                (9.5, 11.0, 8.0, 15.0),
                (8.0, 15.0, 4.0, 15.0),
                (4.0, 15.0, 1.5, 12.5)
            ]
        ),
        'T' => glyph!(10.5, [(1.0, 1.0, 9.5, 1.0), (5.25, 1.0, 5.25, 15.0)]),
        'U' => glyph!(
            11.5,
            [
                (2.0, 1.0, 2.0, 11.5),
                (2.0, 11.5, 4.5, 15.0),
                (4.5, 15.0, 7.0, 15.0),
                (7.0, 15.0, 9.5, 11.5),
                (9.5, 11.5, 9.5, 1.0)
            ]
        ),
        'V' => glyph!(11.5, [(1.5, 1.0, 5.75, 15.0), (5.75, 15.0, 10.0, 1.0)]),
        'W' => glyph!(
            14.5,
            [
                (1.5, 1.0, 4.0, 15.0),
                (4.0, 15.0, 7.25, 6.0),
                (7.25, 6.0, 10.5, 15.0),
                (10.5, 15.0, 13.0, 1.0)
            ]
        ),
        'X' => glyph!(11.0, [(1.5, 1.0, 9.5, 15.0), (9.5, 1.0, 1.5, 15.0)]),
        'Y' => glyph!(
            11.0,
            [
                (1.5, 1.0, 5.5, 7.5),
                (9.5, 1.0, 5.5, 7.5),
                (5.5, 7.5, 5.5, 15.0)
            ]
        ),
        'Z' => glyph!(
            10.5,
            [
                (1.5, 1.0, 9.0, 1.0),
                (9.0, 1.0, 1.5, 15.0),
                (1.5, 15.0, 9.0, 15.0)
            ]
        ),

        // --- LOWERCASE ALPHABET ---
        'a' => glyph!(
            9.5,
            [
                (2.5, 6.0, 7.5, 6.0),
                (7.5, 6.0, 7.5, 15.0),
                (7.5, 15.0, 2.5, 15.0),
                (2.5, 15.0, 1.5, 12.0),
                (1.5, 12.0, 1.5, 9.0),
                (1.5, 9.0, 7.5, 9.0)
            ]
        ),
        'b' => glyph!(
            10.0,
            [
                (2.0, 1.0, 2.0, 15.0),
                (2.0, 6.0, 6.5, 6.0),
                (6.5, 6.0, 8.5, 8.5),
                (8.5, 8.5, 8.5, 12.5),
                (8.5, 12.5, 6.5, 15.0),
                (6.5, 15.0, 2.0, 15.0)
            ]
        ),
        'c' => glyph!(
            9.0,
            [
                (8.0, 8.0, 6.0, 6.0),
                (6.0, 6.0, 3.5, 6.0),
                (3.5, 6.0, 1.5, 8.5),
                (1.5, 8.5, 1.5, 12.5),
                (1.5, 12.5, 3.5, 15.0),
                (3.5, 15.0, 6.5, 15.0),
                (6.5, 15.0, 8.5, 13.0)
            ]
        ),
        'd' => glyph!(
            10.0,
            [
                (8.0, 1.0, 8.0, 15.0),
                (8.0, 6.0, 3.5, 6.0),
                (3.5, 6.0, 1.5, 8.5),
                (1.5, 8.5, 1.5, 12.5),
                (1.5, 12.5, 3.5, 15.0),
                (3.5, 15.0, 8.0, 15.0)
            ]
        ),
        'e' => glyph!(
            9.5,
            [
                (1.5, 10.5, 8.0, 10.5),
                (8.0, 10.5, 8.0, 8.0),
                (8.0, 8.0, 5.5, 6.0),
                (5.5, 6.0, 3.5, 6.0),
                (3.5, 6.0, 1.5, 8.5),
                (1.5, 8.5, 1.5, 12.5),
                (1.5, 12.5, 4.0, 15.0),
                (4.0, 15.0, 8.0, 15.0)
            ]
        ),
        'f' => glyph!(
            6.5,
            [
                (6.0, 2.0, 4.0, 1.0),
                (4.0, 1.0, 2.5, 3.0),
                (2.5, 3.0, 2.5, 15.0),
                (1.0, 6.0, 5.5, 6.0)
            ]
        ),
        'g' => glyph!(
            9.5,
            [
                (7.5, 6.0, 3.5, 6.0),
                (3.5, 6.0, 1.5, 8.5),
                (1.5, 8.5, 3.5, 11.0),
                (3.5, 11.0, 7.5, 11.0),
                (7.5, 6.0, 7.5, 16.5),
                (7.5, 16.5, 4.0, 18.0),
                (4.0, 18.0, 1.5, 16.5)
            ]
        ),
        'h' => glyph!(
            10.0,
            [
                (2.0, 1.0, 2.0, 15.0),
                (2.0, 8.0, 6.0, 6.0),
                (6.0, 6.0, 8.0, 8.0),
                (8.0, 8.0, 8.0, 15.0)
            ]
        ),
        'i' => glyph!(4.5, [(2.2, 3.0, 2.2, 4.2), (2.2, 6.0, 2.2, 15.0)]),
        'j' => glyph!(
            5.5,
            [
                (3.5, 3.0, 3.5, 4.2),
                (3.5, 6.0, 3.5, 16.0),
                (3.5, 16.0, 1.5, 17.5)
            ]
        ),
        'k' => glyph!(
            9.5,
            [
                (2.0, 1.0, 2.0, 15.0),
                (8.0, 6.0, 2.0, 11.0),
                (4.5, 9.0, 8.5, 15.0)
            ]
        ),
        'l' => glyph!(4.5, [(2.2, 1.0, 2.2, 15.0)]),
        'm' => glyph!(
            14.5,
            [
                (1.5, 6.0, 1.5, 15.0),
                (1.5, 8.0, 5.0, 6.0),
                (5.0, 6.0, 7.0, 8.0),
                (7.0, 8.0, 7.0, 15.0),
                (7.0, 8.0, 10.5, 6.0),
                (10.5, 6.0, 12.5, 8.0),
                (12.5, 8.0, 12.5, 15.0)
            ]
        ),
        'n' => glyph!(
            10.0,
            [
                (2.0, 6.0, 2.0, 15.0),
                (2.0, 8.0, 6.0, 6.0),
                (6.0, 6.0, 8.0, 8.0),
                (8.0, 8.0, 8.0, 15.0)
            ]
        ),
        'o' => glyph!(
            9.5,
            [
                (3.5, 6.0, 6.5, 6.0),
                (6.5, 6.0, 8.5, 8.5),
                (8.5, 8.5, 8.5, 12.5),
                (8.5, 12.5, 6.5, 15.0),
                (6.5, 15.0, 3.5, 15.0),
                (3.5, 15.0, 1.5, 12.5),
                (1.5, 12.5, 1.5, 8.5),
                (1.5, 8.5, 3.5, 6.0)
            ]
        ),
        'p' => glyph!(
            10.0,
            [
                (2.0, 6.0, 2.0, 18.0),
                (2.0, 6.0, 6.5, 6.0),
                (6.5, 6.0, 8.5, 8.5),
                (8.5, 8.5, 8.5, 12.0),
                (8.5, 12.0, 6.5, 14.5),
                (6.5, 14.5, 2.0, 14.5)
            ]
        ),
        'q' => glyph!(
            10.0,
            [
                (8.0, 6.0, 8.0, 18.0),
                (8.0, 6.0, 3.5, 6.0),
                (3.5, 6.0, 1.5, 8.5),
                (1.5, 8.5, 1.5, 12.0),
                (1.5, 12.0, 3.5, 14.5),
                (3.5, 14.5, 8.0, 14.5)
            ]
        ),
        'r' => glyph!(
            7.0,
            [
                (2.0, 6.0, 2.0, 15.0),
                (2.0, 8.0, 4.5, 6.0),
                (4.5, 6.0, 6.5, 6.5)
            ]
        ),
        's' => glyph!(
            8.5,
            [
                (7.5, 8.0, 5.5, 6.0),
                (5.5, 6.0, 3.5, 6.0),
                (3.5, 6.0, 1.5, 8.0),
                (1.5, 8.0, 6.5, 10.5),
                (6.5, 10.5, 7.5, 12.5),
                (7.5, 12.5, 5.5, 15.0),
                (5.5, 15.0, 2.0, 15.0)
            ]
        ),
        't' => glyph!(
            6.5,
            [
                (2.5, 2.0, 2.5, 13.5),
                (2.5, 13.5, 4.5, 15.0),
                (1.0, 6.0, 5.5, 6.0)
            ]
        ),
        'u' => glyph!(
            10.0,
            [
                (2.0, 6.0, 2.0, 13.0),
                (2.0, 13.0, 4.5, 15.0),
                (4.5, 15.0, 8.0, 15.0),
                (8.0, 6.0, 8.0, 15.0)
            ]
        ),
        'v' => glyph!(9.5, [(1.5, 6.0, 4.75, 15.0), (4.75, 15.0, 8.0, 6.0)]),
        'w' => glyph!(
            13.0,
            [
                (1.5, 6.0, 3.5, 15.0),
                (3.5, 15.0, 6.5, 9.0),
                (6.5, 9.0, 9.5, 15.0),
                (9.5, 15.0, 11.5, 6.0)
            ]
        ),
        'x' => glyph!(9.0, [(1.5, 6.0, 7.5, 15.0), (7.5, 6.0, 1.5, 15.0)]),
        'y' => glyph!(9.5, [(1.5, 6.0, 4.75, 15.0), (8.0, 6.0, 2.5, 18.0)]),
        'z' => glyph!(
            8.5,
            [
                (1.5, 6.0, 7.5, 6.0),
                (7.5, 6.0, 1.5, 15.0),
                (1.5, 15.0, 7.5, 15.0)
            ]
        ),
        '[' => glyph!(
            6.0,
            [
                (4.0, 1.0, 1.5, 1.0),
                (1.5, 1.0, 1.5, 15.0),
                (1.5, 15.0, 4.0, 15.0)
            ]
        ),
        ']' => glyph!(
            6.0,
            [
                (1.5, 1.0, 4.0, 1.0),
                (4.0, 1.0, 4.0, 15.0),
                (4.0, 15.0, 1.5, 15.0)
            ]
        ),
        _ => glyph!(
            8.0,
            [
                (1.0, 1.0, 7.0, 1.0),
                (7.0, 1.0, 7.0, 15.0),
                (7.0, 15.0, 1.0, 15.0),
                (1.0, 15.0, 1.0, 1.0)
            ]
        ),
    }
}

/// Renders proportional anti-aliased vector typography directly into a `HudMesh`.
#[allow(clippy::too_many_arguments)]
pub fn render_proportional_text(
    hud: &mut HudMesh,
    start_x: f32,
    start_y: f32,
    text: &str,
    size_px: f32,
    color: [f32; 4],
    weight: FontWeight,
    drop_shadow: bool,
) {
    let scale = size_px / 16.0;
    let stroke_w = (size_px * weight.stroke_ratio()).max(1.0);
    let feather = (stroke_w * 0.45).clamp(0.5, 1.2);

    if drop_shadow {
        // Subtle dark translucent drop shadow for contrast
        let shadow_offset = (size_px * 0.08).max(1.0);
        let shadow_color = [0.0, 0.0, 0.0, color[3] * 0.75];
        render_text_passes(
            hud,
            start_x + shadow_offset,
            start_y + shadow_offset,
            text,
            scale,
            stroke_w,
            feather,
            shadow_color,
        );
    }

    render_text_passes(hud, start_x, start_y, text, scale, stroke_w, feather, color);
}

#[allow(clippy::too_many_arguments)]
fn render_text_passes(
    hud: &mut HudMesh,
    start_x: f32,
    start_y: f32,
    text: &str,
    scale: f32,
    stroke_w: f32,
    feather: f32,
    color: [f32; 4],
) {
    let mut cur_x = start_x;
    let mut cur_y = start_y;
    let line_height = 20.0 * scale;

    for ch in text.chars() {
        if ch == '\n' {
            cur_x = start_x;
            cur_y += line_height;
            continue;
        }

        let glyph = get_vector_glyph(ch);
        let half_w = stroke_w * 0.5;

        for seg in glyph.segments() {
            let p1x = cur_x + seg.x1 * scale;
            let p1y = cur_y + seg.y1 * scale;
            let p2x = cur_x + seg.x2 * scale;
            let p2y = cur_y + seg.y2 * scale;

            let dx = p2x - p1x;
            let dy = p2y - p1y;
            let len = (dx * dx + dy * dy).sqrt();

            if len < 1e-4 {
                // Dot / point stroke: small square
                hud.add_rect(p1x - half_w, p1y - half_w, stroke_w, stroke_w, color);
                continue;
            }

            // Normal perpendicular vector
            let nx = (-dy / len) * half_w;
            let ny = (dx / len) * half_w;

            // Solid inner core
            let base_idx = hud.vertices.len() as u32;
            hud.vertices
                .push(HudVertex::new([p1x + nx, p1y + ny], color));
            hud.vertices
                .push(HudVertex::new([p2x + nx, p2y + ny], color));
            hud.vertices
                .push(HudVertex::new([p2x - nx, p2y - ny], color));
            hud.vertices
                .push(HudVertex::new([p1x - nx, p1y - ny], color));

            hud.indices.push(base_idx);
            hud.indices.push(base_idx + 1);
            hud.indices.push(base_idx + 2);
            hud.indices.push(base_idx);
            hud.indices.push(base_idx + 2);
            hud.indices.push(base_idx + 3);

            // Antialiased outer feathering fringes (alpha fades to 0)
            let fnx = (-dy / len) * (half_w + feather);
            let fny = (dx / len) * (half_w + feather);
            let edge_color = [color[0], color[1], color[2], 0.0];

            // Top feather quad
            let top_idx = hud.vertices.len() as u32;
            hud.vertices
                .push(HudVertex::new([p1x + fnx, p1y + fny], edge_color));
            hud.vertices
                .push(HudVertex::new([p2x + fnx, p2y + fny], edge_color));
            hud.vertices
                .push(HudVertex::new([p2x + nx, p2y + ny], color));
            hud.vertices
                .push(HudVertex::new([p1x + nx, p1y + ny], color));

            hud.indices.push(top_idx);
            hud.indices.push(top_idx + 1);
            hud.indices.push(top_idx + 2);
            hud.indices.push(top_idx);
            hud.indices.push(top_idx + 2);
            hud.indices.push(top_idx + 3);

            // Bottom feather quad
            let bot_idx = hud.vertices.len() as u32;
            hud.vertices
                .push(HudVertex::new([p1x - nx, p1y - ny], color));
            hud.vertices
                .push(HudVertex::new([p2x - nx, p2y - ny], color));
            hud.vertices
                .push(HudVertex::new([p2x - fnx, p2y - fny], edge_color));
            hud.vertices
                .push(HudVertex::new([p1x - fnx, p1y - fny], edge_color));

            hud.indices.push(bot_idx);
            hud.indices.push(bot_idx + 1);
            hud.indices.push(bot_idx + 2);
            hud.indices.push(bot_idx);
            hud.indices.push(bot_idx + 2);
            hud.indices.push(bot_idx + 3);
        }

        cur_x += (glyph.advance + 1.2) * scale;
    }
}

/// Evaluates bounding box dimensions (width, height) of proportional text in pixels.
pub fn measure_proportional_text(text: &str, size_px: f32) -> (f32, f32) {
    let scale = size_px / 16.0;
    let line_height = 20.0 * scale;
    let mut max_w = 0.0f32;
    let mut cur_w = 0.0f32;
    let mut lines = 1usize;

    for ch in text.chars() {
        if ch == '\n' {
            max_w = max_w.max(cur_w);
            cur_w = 0.0;
            lines += 1;
        } else {
            let glyph = get_vector_glyph(ch);
            cur_w += (glyph.advance + 1.2) * scale;
        }
    }
    max_w = max_w.max(cur_w);
    let total_h = (lines as f32) * line_height;
    (max_w, total_h)
}
