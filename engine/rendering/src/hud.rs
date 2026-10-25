//! 2D Screen-space HUD and telemetry overlay graphics pipeline.

use crate::typography::{FontSize, FontWeight};
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

/// 2D Screen-space bounding rectangle for responsive layout and mouse hit-testing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl UiRect {
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    /// Evaluates if screen-space physical coordinates (px, py) fall within this rectangle.
    #[inline]
    pub fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && px <= (self.x + self.w) && py >= self.y && py <= (self.y + self.h)
    }

    /// Returns the exact center point (cx, cy) of this rectangle.
    #[inline]
    pub fn center(&self) -> (f32, f32) {
        (self.x + self.w * 0.5, self.y + self.h * 0.5)
    }
}

/// Visual feedback state for interactive UI buttons and list items.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiButtonVisualState {
    Normal,
    Focused,
    Hovered,
    Pressed,
    Disabled,
}

impl UiButtonVisualState {
    pub fn resolve(focused: bool, hovered: bool, pressed: bool, enabled: bool) -> Self {
        if !enabled {
            Self::Disabled
        } else if pressed {
            Self::Pressed
        } else if hovered {
            Self::Hovered
        } else if focused {
            Self::Focused
        } else {
            Self::Normal
        }
    }
}

/// Responsive layout definitions and bounding boxes for the Start / Home screen.
#[derive(Debug, Clone, PartialEq)]
pub struct StartScreenLayout {
    pub menu_card: UiRect,
    pub option_rects: Vec<UiRect>,
    pub quick_roam_btn: UiRect,
    pub close_btn: UiRect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartScreenHit {
    Option(usize),
    QuickRoam,
    Close,
}

impl StartScreenLayout {
    pub fn compute(screen_w: f32, screen_h: f32, options_count: usize) -> Self {
        let menu_x = 60.0;
        let menu_y = screen_h * 0.22;
        let menu_w = (screen_w * 0.45).clamp(380.0, 520.0);
        let item_h = 44.0;
        let spacing = 8.0;

        let mut option_rects = Vec::with_capacity(options_count);
        for i in 0..options_count {
            let iy = menu_y + (i as f32) * (item_h + spacing);
            option_rects.push(UiRect::new(menu_x, iy, menu_w, item_h));
        }

        let total_h = (item_h + spacing) * options_count as f32 - spacing;
        let menu_card = UiRect::new(menu_x - 16.0, menu_y - 16.0, menu_w + 32.0, total_h + 32.0);

        // Quick roam floating button
        let quick_roam_btn = UiRect::new(menu_x, screen_h * 0.08 + 58.0, 320.0, 30.0);
        let close_btn = UiRect::new(screen_w - 44.0, 16.0, 28.0, 28.0);

        Self {
            menu_card,
            option_rects,
            quick_roam_btn,
            close_btn,
        }
    }

    pub fn hit_test(&self, px: f32, py: f32) -> Option<StartScreenHit> {
        if self.close_btn.contains(px, py) {
            return Some(StartScreenHit::Close);
        }
        if self.quick_roam_btn.contains(px, py) {
            return Some(StartScreenHit::QuickRoam);
        }
        for (i, rect) in self.option_rects.iter().enumerate() {
            if rect.contains(px, py) {
                return Some(StartScreenHit::Option(i));
            }
        }
        None
    }
}

/// Responsive layout definitions and bounding boxes for the Simulation Pause Menu.
#[derive(Debug, Clone, PartialEq)]
pub struct PauseMenuLayout {
    pub modal_rect: UiRect,
    pub option_rects: Vec<UiRect>,
    pub resume_btn: UiRect,
    pub close_btn: UiRect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PauseMenuHit {
    Option(usize),
    Resume,
    Close,
}

impl PauseMenuLayout {
    pub fn compute(screen_w: f32, screen_h: f32, options_count: usize) -> Self {
        let modal_w = (screen_w - 40.0).min(520.0);
        let modal_h = (screen_h - 40.0).min(500.0);
        let mx = (screen_w - modal_w) * 0.5;
        let my = (screen_h - modal_h) * 0.5;
        let list_y = my + 72.0;
        let item_h = 34.0;
        let spacing = 6.0;

        let mut option_rects = Vec::with_capacity(options_count);
        for i in 0..options_count {
            let iy = list_y + (i as f32) * (item_h + spacing);
            option_rects.push(UiRect::new(mx + 16.0, iy, modal_w - 32.0, item_h));
        }

        let resume_btn = UiRect::new(mx + modal_w - 140.0, my + 8.0, 100.0, 26.0);
        let close_btn = UiRect::new(mx + modal_w - 34.0, my + 9.0, 24.0, 24.0);

        Self {
            modal_rect: UiRect::new(mx, my, modal_w, modal_h),
            option_rects,
            resume_btn,
            close_btn,
        }
    }

    pub fn hit_test(&self, px: f32, py: f32) -> Option<PauseMenuHit> {
        for (i, rect) in self.option_rects.iter().enumerate() {
            if rect.contains(px, py) {
                return Some(PauseMenuHit::Option(i));
            }
        }
        if self.resume_btn.contains(px, py) {
            return Some(PauseMenuHit::Resume);
        }
        if self.close_btn.contains(px, py) {
            return Some(PauseMenuHit::Close);
        }
        None
    }
}

/// Responsive layout definitions and bounding boxes for the Job & Contract Browser modal.
#[derive(Debug, Clone, PartialEq)]
pub struct JobBrowserLayout {
    pub modal_rect: UiRect,
    pub item_rects: Vec<UiRect>,
    pub start_idx: usize,
    pub visible_count: usize,
    pub accept_btn: UiRect,
    pub back_btn: UiRect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobBrowserHit {
    JobItem(usize),
    AcceptContract,
    Back,
}

impl JobBrowserLayout {
    pub fn compute(screen_w: f32, screen_h: f32, selected_idx: usize, total_jobs: usize) -> Self {
        let modal_w = (screen_w - 40.0).min(820.0);
        let modal_h = (screen_h - 40.0).min(540.0);
        let mx = (screen_w - modal_w) * 0.5;
        let my = (screen_h - modal_h) * 0.5;
        let header_y = my + 50.0;
        let list_y = header_y + 28.0;
        let item_h = 38.0;
        let max_visible = 8;
        let start_idx = if selected_idx >= max_visible / 2 {
            (selected_idx + 1)
                .saturating_sub(max_visible)
                .min(total_jobs.saturating_sub(max_visible))
        } else {
            0
        };
        let visible_count = total_jobs.saturating_sub(start_idx).min(max_visible);

        let mut item_rects = Vec::with_capacity(visible_count);
        for i in 0..visible_count {
            let iy = list_y + i as f32 * (item_h + 3.0);
            item_rects.push(UiRect::new(mx + 12.0, iy, modal_w - 24.0, item_h));
        }

        let footer_y = my + modal_h - 36.0;
        let accept_btn = UiRect::new(mx + modal_w - 240.0, footer_y + 4.0, 140.0, 28.0);
        let back_btn = UiRect::new(mx + modal_w - 90.0, footer_y + 4.0, 80.0, 28.0);

        Self {
            modal_rect: UiRect::new(mx, my, modal_w, modal_h),
            item_rects,
            start_idx,
            visible_count,
            accept_btn,
            back_btn,
        }
    }

    pub fn hit_test(&self, px: f32, py: f32) -> Option<JobBrowserHit> {
        for (i, rect) in self.item_rects.iter().enumerate() {
            if rect.contains(px, py) {
                return Some(JobBrowserHit::JobItem(self.start_idx + i));
            }
        }
        if self.accept_btn.contains(px, py) {
            return Some(JobBrowserHit::AcceptContract);
        }
        if self.back_btn.contains(px, py) {
            return Some(JobBrowserHit::Back);
        }
        None
    }
}

/// Responsive layout definitions and bounding boxes for the Vehicle Garage modal.
/// Curated palette of 7 gloss automotive paint colors for garage vehicle customization.
pub const GARAGE_PAINT_PALETTE: [(&str, [f32; 4]); 7] = [
    ("Obsidian Black", [0.10, 0.10, 0.12, 1.0]),
    ("Alpine White", [0.95, 0.95, 0.98, 1.0]),
    ("Guards Red", [0.88, 0.12, 0.12, 1.0]),
    ("Racing Blue", [0.12, 0.40, 0.88, 1.0]),
    ("Solar Gold", [0.94, 0.76, 0.16, 1.0]),
    ("Emerald Green", [0.08, 0.62, 0.32, 1.0]),
    ("Titanium Silver", [0.65, 0.68, 0.72, 1.0]),
];

/// Responsive layout definitions and bounding boxes for the Vehicle Garage modal.
#[derive(Debug, Clone, PartialEq)]
pub struct GarageLayout {
    pub modal_rect: UiRect,
    pub item_rects: Vec<UiRect>,
    pub color_rects: Vec<UiRect>,
    pub select_btn: UiRect,
    pub back_btn: UiRect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GarageHit {
    VehicleItem(usize),
    ColorOption(usize),
    SelectAndSpawn,
    Back,
}

impl GarageLayout {
    pub fn compute(screen_w: f32, screen_h: f32, vehicle_count: usize) -> Self {
        let modal_w = (screen_w - 40.0).min(780.0);
        let modal_h = (screen_h - 40.0).min(520.0);
        let mx = (screen_w - modal_w) * 0.5;
        let my = (screen_h - modal_h) * 0.5;
        let header_y = my + 50.0;
        let list_y = header_y + 28.0;
        let item_h = 36.0;

        let mut item_rects = Vec::with_capacity(vehicle_count);
        for i in 0..vehicle_count {
            let iy = list_y + i as f32 * (item_h + 3.0);
            item_rects.push(UiRect::new(mx + 12.0, iy, modal_w - 24.0, item_h));
        }

        let footer_y = my + modal_h - 36.0;
        let select_btn = UiRect::new(mx + modal_w - 240.0, footer_y + 4.0, 140.0, 28.0);
        let back_btn = UiRect::new(mx + modal_w - 90.0, footer_y + 4.0, 80.0, 28.0);

        // 7 Color swatch chips in bottom bar to the left of action buttons
        let chip_size = 20.0;
        let chip_spacing = 6.0;
        let colors_start_x = mx + 20.0;
        let mut color_rects = Vec::with_capacity(GARAGE_PAINT_PALETTE.len());
        for c in 0..GARAGE_PAINT_PALETTE.len() {
            let cx = colors_start_x + (c as f32) * (chip_size + chip_spacing);
            color_rects.push(UiRect::new(cx, footer_y + 8.0, chip_size, chip_size));
        }

        Self {
            modal_rect: UiRect::new(mx, my, modal_w, modal_h),
            item_rects,
            color_rects,
            select_btn,
            back_btn,
        }
    }

    pub fn hit_test(&self, px: f32, py: f32) -> Option<GarageHit> {
        for (i, rect) in self.item_rects.iter().enumerate() {
            if rect.contains(px, py) {
                return Some(GarageHit::VehicleItem(i));
            }
        }
        for (i, rect) in self.color_rects.iter().enumerate() {
            if rect.contains(px, py) {
                return Some(GarageHit::ColorOption(i));
            }
        }
        if self.select_btn.contains(px, py) {
            return Some(GarageHit::SelectAndSpawn);
        }
        if self.back_btn.contains(px, py) {
            return Some(GarageHit::Back);
        }
        None
    }
}

/// Responsive layout definitions and bounding boxes for the Settings Preferences modal.
#[derive(Debug, Clone, PartialEq)]
pub struct SettingsLayout {
    pub modal_rect: UiRect,
    pub item_rects: Vec<UiRect>,
    pub back_btn: UiRect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsHit {
    SettingItem(usize),
    Back,
}

impl SettingsLayout {
    pub fn compute(screen_w: f32, screen_h: f32, settings_count: usize) -> Self {
        let modal_w = (screen_w - 40.0).min(680.0);
        let modal_h = (screen_h - 40.0).min(460.0);
        let mx = (screen_w - modal_w) * 0.5;
        let my = (screen_h - modal_h) * 0.5;
        let list_y = my + 56.0;
        let item_h = 42.0;

        let mut item_rects = Vec::with_capacity(settings_count);
        for i in 0..settings_count {
            let iy = list_y + i as f32 * (item_h + 4.0);
            item_rects.push(UiRect::new(mx + 16.0, iy, modal_w - 32.0, item_h));
        }

        let footer_y = my + modal_h - 36.0;
        let back_btn = UiRect::new(mx + modal_w - 90.0, footer_y + 4.0, 80.0, 28.0);

        Self {
            modal_rect: UiRect::new(mx, my, modal_w, modal_h),
            item_rects,
            back_btn,
        }
    }

    pub fn hit_test(&self, px: f32, py: f32) -> Option<SettingsHit> {
        for (i, rect) in self.item_rects.iter().enumerate() {
            if rect.contains(px, py) {
                return Some(SettingsHit::SettingItem(i));
            }
        }
        if self.back_btn.contains(px, py) {
            return Some(SettingsHit::Back);
        }
        None
    }
}

/// Responsive layout definitions and bounding boxes for the 38 US Cities Catalog modal.
#[derive(Debug, Clone, PartialEq)]
pub struct CatalogLayout {
    pub modal_rect: UiRect,
    pub item_rects: Vec<UiRect>,
    pub start_idx: usize,
    pub visible_count: usize,
    pub warp_btn: UiRect,
    pub back_btn: UiRect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CatalogHit {
    CityItem(usize),
    WarpCity,
    Back,
}

impl CatalogLayout {
    pub fn compute(screen_w: f32, screen_h: f32, selected_idx: usize, total_cities: usize) -> Self {
        let modal_w = (screen_w - 40.0).min(780.0);
        let modal_h = (screen_h - 40.0).min(520.0);
        let mx = (screen_w - modal_w) * 0.5;
        let my = (screen_h - modal_h) * 0.5;
        let header_y = my + 52.0;
        let list_y = header_y + 28.0;
        let item_h = 36.0;
        let max_visible = 10;
        let start_idx = if selected_idx >= max_visible / 2 {
            (selected_idx + 1)
                .saturating_sub(max_visible)
                .min(total_cities.saturating_sub(max_visible))
        } else {
            0
        };
        let visible_count = total_cities.saturating_sub(start_idx).min(max_visible);

        let mut item_rects = Vec::with_capacity(visible_count);
        for i in 0..visible_count {
            let iy = list_y + i as f32 * (item_h + 2.0);
            item_rects.push(UiRect::new(mx + 10.0, iy, modal_w - 20.0, item_h));
        }

        let footer_y = my + modal_h - 36.0;
        let warp_btn = UiRect::new(mx + modal_w - 240.0, footer_y + 4.0, 140.0, 28.0);
        let back_btn = UiRect::new(mx + modal_w - 90.0, footer_y + 4.0, 80.0, 28.0);

        Self {
            modal_rect: UiRect::new(mx, my, modal_w, modal_h),
            item_rects,
            start_idx,
            visible_count,
            warp_btn,
            back_btn,
        }
    }

    pub fn hit_test(&self, px: f32, py: f32) -> Option<CatalogHit> {
        for (i, rect) in self.item_rects.iter().enumerate() {
            if rect.contains(px, py) {
                return Some(CatalogHit::CityItem(self.start_idx + i));
            }
        }
        if self.warp_btn.contains(px, py) {
            return Some(CatalogHit::WarpCity);
        }
        if self.back_btn.contains(px, py) {
            return Some(CatalogHit::Back);
        }
        None
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

    /// Renders a multi-line string using proportional, antialiased vector typography.
    pub fn add_text(
        &mut self,
        start_x: f32,
        start_y: f32,
        scale: f32,
        text: &str,
        color: [f32; 4],
    ) {
        let size_px = (scale * 13.5).max(9.0);
        let weight = if scale >= 1.5 {
            crate::typography::FontWeight::Bold
        } else {
            crate::typography::FontWeight::Regular
        };
        crate::typography::render_proportional_text(
            self, start_x, start_y, text, size_px, color, weight, true,
        );
    }

    /// Renders proportional anti-aliased vector typography with explicit font size, weight, and shadow.
    #[allow(clippy::too_many_arguments)]
    pub fn add_text_proportional(
        &mut self,
        start_x: f32,
        start_y: f32,
        text: &str,
        size_px: f32,
        color: [f32; 4],
        weight: crate::typography::FontWeight,
        drop_shadow: bool,
    ) {
        crate::typography::render_proportional_text(
            self,
            start_x,
            start_y,
            text,
            size_px,
            color,
            weight,
            drop_shadow,
        );
    }

    /// Evaluates text dimensions (width, height) in screen pixels.
    pub fn measure_text_proportional(&self, text: &str, size_px: f32) -> (f32, f32) {
        crate::typography::measure_proportional_text(text, size_px)
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

    /// Adds a turn-by-turn navigation guidance overlay banner to the top-center of the screen.
    #[allow(clippy::too_many_arguments)]
    pub fn add_navigation_guidance(
        &mut self,
        screen_w: f32,
        _screen_h: f32,
        maneuver_icon: &str,
        distance_meters: f64,
        instruction_text: &str,
        street_name: Option<&str>,
        speed_limit_kph: u32,
        is_wrong_way: bool,
    ) {
        let banner_w = 460.0;
        let banner_h = 66.0;
        let x = (screen_w - banner_w) * 0.5;
        let y = 16.0;

        let accent_color = if is_wrong_way {
            [1.0, 0.1, 0.1, 1.0] // Flashing red
        } else {
            [0.1, 0.85, 0.45, 1.0] // Clean Emerald Green
        };

        self.add_panel(x, y, banner_w, banner_h, accent_color);

        if is_wrong_way {
            self.add_rect(
                x + 8.0,
                y + 8.0,
                banner_w - 16.0,
                banner_h - 16.0,
                [0.85, 0.05, 0.05, 0.95],
            );
            self.add_text(
                x + 40.0,
                y + 24.0,
                2.2,
                "! WRONG WAY - TURN AROUND !",
                [1.0, 1.0, 1.0, 1.0],
            );
            return;
        }

        // Maneuver icon / glyph
        self.add_text(x + 16.0, y + 20.0, 2.8, maneuver_icon, [0.2, 0.9, 0.5, 1.0]);

        // Distance text
        let dist_str = if distance_meters >= 1000.0 {
            format!("In {:.1} km", distance_meters / 1000.0)
        } else {
            format!("In {:.0} m", distance_meters)
        };
        self.add_text(x + 65.0, y + 14.0, 1.5, &dist_str, [1.0, 0.85, 0.2, 1.0]);

        // Street name / instruction
        let display_street = street_name.unwrap_or(instruction_text);
        self.add_text(
            x + 65.0,
            y + 36.0,
            1.8,
            display_street,
            [1.0, 1.0, 1.0, 1.0],
        );

        // Speed limit badge (Right side)
        let badge_x = x + banner_w - 65.0;
        let badge_y = y + 10.0;
        self.add_rect(badge_x, badge_y, 48.0, 48.0, [0.95, 0.95, 0.95, 1.0]);
        self.add_rect_outline(badge_x, badge_y, 48.0, 48.0, 2.0, [0.85, 0.1, 0.1, 1.0]);
        self.add_text(
            badge_x + 8.0,
            badge_y + 6.0,
            1.0,
            "LIMIT",
            [0.1, 0.1, 0.1, 1.0],
        );
        let limit_str = format!("{}", speed_limit_kph);
        self.add_text(
            badge_x + 12.0,
            badge_y + 22.0,
            2.0,
            &limit_str,
            [0.1, 0.1, 0.1, 1.0],
        );
    }

    /// Adds an interactive button with selectable / hover state and optional shortcut key badge.
    #[allow(clippy::too_many_arguments)]
    pub fn add_button(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        label: &str,
        shortcut: Option<&str>,
        is_selected: bool,
    ) {
        let (bg, border, text_col, accent) = if is_selected {
            (
                [0.15, 0.28, 0.45, 0.95],
                [0.3, 0.75, 1.0, 1.0],
                [1.0, 1.0, 1.0, 1.0],
                [0.0, 0.8, 1.0, 1.0],
            )
        } else {
            (
                [0.08, 0.10, 0.15, 0.85],
                [0.25, 0.30, 0.40, 0.6],
                [0.80, 0.85, 0.90, 1.0],
                [0.2, 0.3, 0.4, 0.0],
            )
        };

        // Background & border
        self.add_rect(x, y, w, h, bg);
        self.add_rect_outline(x, y, w, h, if is_selected { 2.0 } else { 1.0 }, border);

        // Accent indicator on left if selected
        if is_selected {
            self.add_rect(x, y, 4.0, h, accent);
        }

        // Label text (vertically centered)
        let text_y = y + (h - 14.0) * 0.5;
        self.add_text(x + 16.0, text_y, 1.5, label, text_col);

        // Shortcut badge on right if present
        if let Some(sc) = shortcut {
            let sc_w = (sc.len() as f32 * 6.0 * 1.1) + 12.0;
            let sc_x = x + w - sc_w - 12.0;
            let sc_y = y + (h - 18.0) * 0.5;
            self.add_rect(sc_x, sc_y, sc_w, 18.0, [0.04, 0.06, 0.08, 0.9]);
            self.add_rect_outline(sc_x, sc_y, sc_w, 18.0, 1.0, [0.35, 0.45, 0.60, 0.6]);
            self.add_text(sc_x + 6.0, sc_y + 3.0, 1.1, sc, [0.4, 0.9, 1.0, 1.0]);
        }
    }

    /// Adds a centered modal dialog window with responsive translucent glass backdrop.
    pub fn add_modal(
        &mut self,
        screen_w: f32,
        screen_h: f32,
        w: f32,
        h: f32,
        title: &str,
        accent_color: [f32; 4],
    ) -> (f32, f32) {
        // Translucent backdrop dimming (reveals vibrant 3D simulation behind modals)
        self.add_rect(0.0, 0.0, screen_w, screen_h, [0.0, 0.0, 0.0, 0.30]);

        // Centered window
        let x = (screen_w - w) * 0.5;
        let y = (screen_h - h) * 0.5;

        // Dark translucent glass window body
        self.add_rect(x, y, w, h, [0.06, 0.08, 0.13, 0.85]);
        self.add_rect_outline(x, y, w, h, 2.0, [0.3, 0.45, 0.65, 0.8]);

        // Top title bar with accent line
        self.add_rect(x, y, w, 44.0, [0.10, 0.13, 0.20, 0.98]);
        self.add_rect(x, y, w, 3.0, accent_color);
        self.add_text(x + 18.0, y + 14.0, 1.8, title, [1.0, 1.0, 1.0, 1.0]);

        (x, y)
    }

    /// Adds an interactive city catalog browser modal window with list navigation.
    pub fn add_city_catalog_modal(
        &mut self,
        screen_w: f32,
        screen_h: f32,
        selected_idx: usize,
        cities: &[(&str, &str, &str, &str, f32, bool, &str)],
    ) {
        self.add_city_catalog_modal_interactive(
            screen_w,
            screen_h,
            selected_idx,
            None,
            None,
            None,
            None,
            cities,
        );
    }

    /// Adds an interactive city catalog browser modal window with list navigation and pointer feedback.
    #[allow(clippy::too_many_arguments)]
    pub fn add_city_catalog_modal_interactive(
        &mut self,
        screen_w: f32,
        screen_h: f32,
        selected_idx: usize,
        hovered_idx: Option<usize>,
        pressed_idx: Option<usize>,
        hovered_btn: Option<CatalogHit>,
        pressed_btn: Option<CatalogHit>,
        cities: &[(&str, &str, &str, &str, f32, bool, &str)],
    ) {
        let layout = CatalogLayout::compute(screen_w, screen_h, selected_idx, cities.len());
        let modal_w = layout.modal_rect.w;
        let modal_h = layout.modal_rect.h;
        let (mx, my) = self.add_modal(
            screen_w,
            screen_h,
            modal_w,
            modal_h,
            "UNITED STATES GEOGRAPHY CATALOG — 38 AUDITED METROPOLITAN REGIONS",
            [0.2, 0.65, 1.0, 1.0],
        );

        // Subtitle / Column Headers
        let header_y = my + 52.0;
        self.add_rect(
            mx + 10.0,
            header_y,
            modal_w - 20.0,
            24.0,
            [0.12, 0.16, 0.24, 0.9],
        );
        self.add_text(
            mx + 20.0,
            header_y + 6.0,
            1.1,
            "STATE / CITY NAME",
            [0.6, 0.75, 0.9, 1.0],
        );
        self.add_text(
            mx + 260.0,
            header_y + 6.0,
            1.1,
            "SURVEYED ANCHOR LANDMARK",
            [0.6, 0.75, 0.9, 1.0],
        );
        self.add_text(
            mx + 540.0,
            header_y + 6.0,
            1.1,
            "MAX ELEV",
            [0.6, 0.75, 0.9, 1.0],
        );
        self.add_text(
            mx + 630.0,
            header_y + 6.0,
            1.1,
            "PORT",
            [0.6, 0.75, 0.9, 1.0],
        );
        self.add_text(
            mx + 690.0,
            header_y + 6.0,
            1.1,
            "TIER",
            [0.6, 0.75, 0.9, 1.0],
        );

        let list_y = header_y + 28.0;
        let item_h = 36.0;
        let start_idx = layout.start_idx;
        let visible_cities = &cities[start_idx..cities.len().min(start_idx + layout.visible_count)];

        for (i, &(_id, name, state, landmark, elev_max, has_port, priority_str)) in
            visible_cities.iter().enumerate()
        {
            let actual_idx = start_idx + i;
            let is_sel = actual_idx == selected_idx;
            let is_hovered = hovered_idx == Some(actual_idx);
            let is_pressed = pressed_idx == Some(actual_idx);
            let iy = list_y + i as f32 * (item_h + 2.0);

            let (row_bg, border_col) = if is_pressed {
                ([0.22, 0.42, 0.70, 0.98], [0.55, 0.95, 1.0, 1.0])
            } else if is_hovered {
                ([0.18, 0.36, 0.60, 0.95], [0.45, 0.90, 1.0, 1.0])
            } else if is_sel {
                ([0.14, 0.28, 0.48, 0.90], [0.35, 0.8, 1.0, 1.0])
            } else {
                ([0.08, 0.11, 0.16, 0.80], [0.20, 0.25, 0.35, 0.4])
            };

            self.add_rect(mx + 10.0, iy, modal_w - 20.0, item_h, row_bg);
            self.add_rect_outline(
                mx + 10.0,
                iy,
                modal_w - 20.0,
                item_h,
                if is_sel || is_hovered { 1.5 } else { 0.5 },
                border_col,
            );

            // Selection indicator
            if is_sel || is_hovered {
                self.add_rect(mx + 10.0, iy, 4.0, item_h, [0.0, 0.85, 1.0, 1.0]);
            }

            // State badge
            let state_badge = format!("[{}]", state);
            self.add_text(
                mx + 20.0,
                iy + 10.0,
                1.1,
                &state_badge,
                [0.4, 0.85, 1.0, 1.0],
            );

            // City Name
            self.add_text(
                mx + 55.0,
                iy + 10.0,
                1.2,
                name,
                if is_sel || is_hovered {
                    [1.0, 1.0, 1.0, 1.0]
                } else {
                    [0.85, 0.9, 0.95, 1.0]
                },
            );

            // Landmark
            let truncated_landmark = if landmark.len() > 32 {
                &landmark[..32]
            } else {
                landmark
            };
            self.add_text(
                mx + 260.0,
                iy + 10.0,
                1.1,
                truncated_landmark,
                [0.75, 0.80, 0.85, 0.9],
            );

            // Elev
            let elev_str = format!("{:.0}m", elev_max);
            self.add_text(mx + 540.0, iy + 10.0, 1.1, &elev_str, [0.7, 0.85, 0.7, 1.0]);

            // Port
            let port_str = if has_port { "⚓ Yes" } else { "—" };
            self.add_text(
                mx + 630.0,
                iy + 10.0,
                1.1,
                port_str,
                if has_port {
                    [0.3, 0.8, 1.0, 1.0]
                } else {
                    [0.5, 0.5, 0.5, 0.6]
                },
            );

            // Priority Tier
            self.add_text(
                mx + 690.0,
                iy + 10.0,
                1.0,
                priority_str,
                [0.95, 0.75, 0.2, 1.0],
            );
        }

        // Footer Navigation Controls Banner
        let footer_y = my + modal_h - 36.0;
        self.add_rect(mx, footer_y, modal_w, 36.0, [0.08, 0.10, 0.15, 0.98]);
        self.add_rect(mx, footer_y, modal_w, 1.0, [0.25, 0.35, 0.50, 0.6]);
        self.add_text(
            mx + 20.0,
            footer_y + 11.0,
            1.05,
            "NAVIGATE: [↑/↓] or Scroll  |  LOAD: [Enter] / Click",
            [0.35, 0.85, 1.0, 1.0],
        );

        // Clickable Warp Button
        let wb = layout.warp_btn;
        let is_wb_hovered = hovered_btn == Some(CatalogHit::WarpCity);
        let is_wb_pressed = pressed_btn == Some(CatalogHit::WarpCity);
        let (wb_bg, wb_border) = if is_wb_pressed {
            ([0.15, 0.50, 0.35, 0.98], [0.4, 1.0, 0.7, 1.0])
        } else if is_wb_hovered {
            ([0.12, 0.42, 0.28, 0.95], [0.3, 0.95, 0.6, 1.0])
        } else {
            ([0.08, 0.28, 0.20, 0.85], [0.2, 0.75, 0.45, 0.7])
        };
        self.add_rect(wb.x, wb.y, wb.w, wb.h, wb_bg);
        self.add_rect_outline(wb.x, wb.y, wb.w, wb.h, 1.0, wb_border);
        self.add_text(
            wb.x + 12.0,
            wb.y + 7.0,
            1.1,
            "▶ WARP TO CITY",
            [0.3, 1.0, 0.6, 1.0],
        );

        // Clickable Back Button
        let bb = layout.back_btn;
        let is_bb_hovered = hovered_btn == Some(CatalogHit::Back);
        let is_bb_pressed = pressed_btn == Some(CatalogHit::Back);
        let (bb_bg, bb_border) = if is_bb_pressed {
            ([0.35, 0.15, 0.15, 0.98], [1.0, 0.5, 0.5, 1.0])
        } else if is_bb_hovered {
            ([0.28, 0.12, 0.12, 0.95], [0.95, 0.4, 0.4, 1.0])
        } else {
            ([0.16, 0.08, 0.08, 0.85], [0.70, 0.3, 0.3, 0.7])
        };
        self.add_rect(bb.x, bb.y, bb.w, bb.h, bb_bg);
        self.add_rect_outline(bb.x, bb.y, bb.w, bb.h, 1.0, bb_border);
        self.add_text(bb.x + 10.0, bb.y + 7.0, 1.1, "◀ BACK", [1.0, 0.6, 0.6, 1.0]);
    }

    /// Adds a compact glassmorphic multiplayer status chip in the top right.
    pub fn add_multiplayer_chip(
        &mut self,
        screen_w: f32,
        is_connected: bool,
        is_reconnecting: bool,
        ping_ms: f32,
        remote_player_count: usize,
    ) {
        let chip_w = 280.0;
        let chip_h = 32.0;
        let x = screen_w - chip_w - 20.0;
        let y = 16.0;

        let (bg_color, border_color, status_text) = if is_connected {
            (
                [0.05, 0.12, 0.08, 0.85],
                [0.15, 0.85, 0.35, 0.9],
                format!(
                    "NET: CONNECTED | {:.0}ms | {} Online",
                    ping_ms,
                    remote_player_count + 1
                ),
            )
        } else if is_reconnecting {
            (
                [0.14, 0.10, 0.04, 0.85],
                [0.95, 0.65, 0.10, 0.9],
                "NET: RECONNECTING... [N]".to_string(),
            )
        } else {
            (
                [0.08, 0.09, 0.12, 0.75],
                [0.35, 0.45, 0.55, 0.5],
                "NET: OFFLINE | Press [N] for Multiplayer".to_string(),
            )
        };

        self.add_rect(x, y, chip_w, chip_h, bg_color);
        self.add_rect_outline(x, y, chip_w, chip_h, 1.5, border_color);
        self.add_text(
            x + 12.0,
            y + 9.0,
            1.05,
            &status_text,
            if is_connected {
                [0.4, 1.0, 0.6, 1.0]
            } else if is_reconnecting {
                [1.0, 0.8, 0.2, 1.0]
            } else {
                [0.7, 0.75, 0.85, 0.8]
            },
        );
    }

    /// Adds an interactive authoritative multiplayer lobby and session management modal.
    #[allow(clippy::too_many_arguments)]
    pub fn add_multiplayer_modal(
        &mut self,
        screen_w: f32,
        screen_h: f32,
        is_connected: bool,
        is_hosting: bool,
        server_addr: &str,
        ping_ms: f32,
        player_name: &str,
        selected_action: usize,
        players_roster: &[(&str, &str, f32)],
    ) {
        let modal_w = (screen_w - 40.0).min(760.0);
        let modal_h = (screen_h - 40.0).min(500.0);
        let (mx, my) = self.add_modal(
            screen_w,
            screen_h,
            modal_w,
            modal_h,
            "AUTHORITATIVE MULTIPLAYER NETWORK — SESSION & LOBBY",
            [0.2, 0.75, 1.0, 1.0],
        );

        // Section 1: Session & Server Info Banner
        let info_y = my + 52.0;
        self.add_rect(
            mx + 16.0,
            info_y,
            modal_w - 32.0,
            68.0,
            [0.10, 0.13, 0.18, 0.9],
        );
        self.add_rect_outline(
            mx + 16.0,
            info_y,
            modal_w - 32.0,
            68.0,
            1.0,
            [0.25, 0.35, 0.5, 0.6],
        );

        let status_label = if is_hosting {
            "● HOSTING (Dedicated Server on Local Thread)"
        } else if is_connected {
            "● CONNECTED (Authoritative Client Session)"
        } else {
            "○ OFFLINE (Single-Player Mode Active)"
        };
        let status_color = if is_hosting || is_connected {
            [0.2, 1.0, 0.5, 1.0]
        } else {
            [0.6, 0.65, 0.75, 1.0]
        };

        self.add_text(mx + 28.0, info_y + 12.0, 1.25, status_label, status_color);

        let details = format!(
            "Server: {}   |   Latency: {:.1} ms   |   Driver Name: {}   |   Protocol: v{}",
            server_addr, ping_ms, player_name, 1
        );
        self.add_text(
            mx + 28.0,
            info_y + 38.0,
            1.1,
            &details,
            [0.75, 0.85, 0.95, 0.9],
        );

        // Section 2: Action Menu Buttons
        let actions_y = info_y + 80.0;
        let actions = [
            (
                "1. HOST LOCAL SERVER",
                "Host headless authoritative server on port 7777 and join automatically",
            ),
            (
                "2. JOIN DEDICATED SERVER",
                "Connect non-blockingly to authoritative server at configured address",
            ),
            (
                "3. DISCONNECT / LEAVE SESSION",
                "Cleanly leave session and return to zero-overhead single-player mode",
            ),
            (
                "4. RESUME SIMULATION",
                "Close multiplayer modal and return to driver cockpit",
            ),
        ];

        for (i, &(title, desc)) in actions.iter().enumerate() {
            let is_sel = i == selected_action;
            let by = actions_y + i as f32 * 46.0;
            let bg = if is_sel {
                [0.18, 0.32, 0.52, 0.95]
            } else {
                [0.10, 0.12, 0.17, 0.8]
            };
            let border = if is_sel {
                [0.35, 0.75, 1.0, 1.0]
            } else {
                [0.2, 0.25, 0.35, 0.5]
            };

            self.add_rect(mx + 16.0, by, modal_w - 32.0, 42.0, bg);
            self.add_rect_outline(
                mx + 16.0,
                by,
                modal_w - 32.0,
                42.0,
                if is_sel { 1.5 } else { 1.0 },
                border,
            );

            self.add_text(
                mx + 30.0,
                by + 8.0,
                1.2,
                title,
                if is_sel {
                    [1.0, 1.0, 1.0, 1.0]
                } else {
                    [0.85, 0.9, 0.95, 0.9]
                },
            );
            self.add_text(
                mx + 30.0,
                by + 24.0,
                0.95,
                desc,
                if is_sel {
                    [0.7, 0.85, 1.0, 1.0]
                } else {
                    [0.55, 0.6, 0.7, 0.7]
                },
            );
        }

        // Section 3: Live Connected Players Roster Table
        let roster_y = actions_y + actions.len() as f32 * 46.0 + 8.0;
        self.add_rect(
            mx + 16.0,
            roster_y,
            modal_w - 32.0,
            22.0,
            [0.12, 0.16, 0.22, 0.9],
        );
        self.add_text(
            mx + 28.0,
            roster_y + 5.0,
            1.0,
            "CONNECTED PARTICIPANT",
            [0.6, 0.75, 0.9, 1.0],
        );
        self.add_text(
            mx + 340.0,
            roster_y + 5.0,
            1.0,
            "VEHICLE CLASS",
            [0.6, 0.75, 0.9, 1.0],
        );
        self.add_text(
            mx + 580.0,
            roster_y + 5.0,
            1.0,
            "PING / RTT",
            [0.6, 0.75, 0.9, 1.0],
        );

        if players_roster.is_empty() {
            let row_y = roster_y + 26.0;
            self.add_rect(
                mx + 16.0,
                row_y,
                modal_w - 32.0,
                28.0,
                [0.08, 0.10, 0.14, 0.7],
            );
            self.add_text(
                mx + 28.0,
                row_y + 7.0,
                1.05,
                "No remote peers connected. Host or join a session above.",
                [0.6, 0.65, 0.75, 0.8],
            );
        } else {
            for (idx, &(pname, vtype, prtt)) in players_roster.iter().take(3).enumerate() {
                let row_y = roster_y + 24.0 + idx as f32 * 26.0;
                self.add_rect(
                    mx + 16.0,
                    row_y,
                    modal_w - 32.0,
                    24.0,
                    [0.08, 0.10, 0.14, 0.8],
                );
                self.add_text(mx + 28.0, row_y + 6.0, 1.05, pname, [0.9, 0.95, 1.0, 1.0]);
                self.add_text(
                    mx + 340.0,
                    row_y + 6.0,
                    1.05,
                    vtype,
                    [0.75, 0.85, 0.75, 1.0],
                );
                let rtt_str = format!("{:.0} ms", prtt);
                self.add_text(
                    mx + 580.0,
                    row_y + 6.0,
                    1.05,
                    &rtt_str,
                    [0.4, 0.9, 0.5, 1.0],
                );
            }
        }

        // Footer Banner
        let footer_y = my + modal_h - 36.0;
        self.add_rect(mx, footer_y, modal_w, 36.0, [0.08, 0.10, 0.15, 0.98]);
        self.add_rect(mx, footer_y, modal_w, 1.0, [0.25, 0.35, 0.50, 0.6]);
        self.add_text(
            mx + 20.0,
            footer_y + 11.0,
            1.1,
            "NAVIGATE: [↑/↓] or [W/S]  |  EXECUTE: [Enter]  |  DISMISS: [N] or [Esc]  |  ZERO-OVERHEAD OFFLINE PRESERVED",
            [0.35, 0.85, 1.0, 1.0],
        );
    }

    /// Adds a polished, responsive start / home screen interface.
    #[allow(clippy::too_many_arguments)]
    pub fn add_start_screen(
        &mut self,
        screen_w: f32,
        screen_h: f32,
        selected_opt: usize,
        options: &[(&str, &str)],
        player_name: &str,
        player_wallet: f64,
        completed_jobs: u32,
    ) {
        self.add_start_screen_interactive(
            screen_w,
            screen_h,
            selected_opt,
            None,
            None,
            None,
            None,
            options,
            player_name,
            player_wallet,
            completed_jobs,
            true,
        );
    }

    /// Adds a modern, responsive, non-obstructive start / home screen interface with full interactive pointer feedback.
    #[allow(clippy::too_many_arguments)]
    pub fn add_start_screen_interactive(
        &mut self,
        screen_w: f32,
        screen_h: f32,
        selected_opt: usize,
        hovered_opt: Option<usize>,
        pressed_opt: Option<usize>,
        hovered_btn: Option<StartScreenHit>,
        pressed_btn: Option<StartScreenHit>,
        options: &[(&str, &str)],
        player_name: &str,
        player_wallet: f64,
        completed_jobs: u32,
        continue_enabled: bool,
    ) {
        let layout = StartScreenLayout::compute(screen_w, screen_h, options.len());

        // Top translucent header banner (preserves unobstructed 3D scene view)
        self.add_rect(0.0, 0.0, screen_w, 92.0, [0.03, 0.05, 0.08, 0.35]);
        self.add_line(0.0, 92.0, screen_w, 92.0, 1.0, [0.15, 0.40, 0.70, 0.4]);

        // Title Header
        let title_x = 60.0;
        let title_y = screen_h * 0.04;
        self.add_text(
            title_x,
            title_y,
            3.0,
            "REAL EARTH WORLD",
            [1.0, 1.0, 1.0, 1.0],
        );
        self.add_text(
            title_x,
            title_y + 32.0,
            1.15,
            "HIGH-FIDELITY PLANETARY SIMULATION & MULTIMODAL GAMEPLAY",
            [0.2, 0.85, 1.0, 0.9],
        );

        // Player Profile Badge (Top Right)
        let profile_w = 340.0;
        let profile_x = screen_w - profile_w - 40.0;
        let profile_y = title_y;
        self.add_panel(profile_x, profile_y, profile_w, 54.0, [0.2, 0.9, 0.5, 0.9]);
        self.add_text(
            profile_x + 14.0,
            profile_y + 8.0,
            1.15,
            player_name,
            [1.0, 1.0, 1.0, 1.0],
        );
        let stats_str = format!("WALLET: ${:.2}  |  JOBS: {}", player_wallet, completed_jobs);
        self.add_text(
            profile_x + 14.0,
            profile_y + 28.0,
            1.0,
            &stats_str,
            [1.0, 0.85, 0.2, 1.0],
        );

        // Menu Card Translucent Panel
        let card = layout.menu_card;
        self.add_rect(card.x, card.y, card.w, card.h, [0.04, 0.06, 0.10, 0.60]);
        self.add_rect_outline(card.x, card.y, card.w, card.h, 1.5, [0.20, 0.35, 0.55, 0.7]);

        // Left decorative accent bar on menu card
        self.add_rect(card.x, card.y, 4.0, card.h, [0.0, 0.85, 1.0, 0.8]);

        // Quick Roam prominent banner button (below title)
        let qr = layout.quick_roam_btn;
        let is_qr_pressed = pressed_btn == Some(StartScreenHit::QuickRoam);
        let is_qr_hovered = hovered_btn == Some(StartScreenHit::QuickRoam);
        let (qr_bg, qr_border) = if is_qr_pressed {
            ([0.18, 0.55, 0.35, 0.98], [0.4, 1.0, 0.7, 1.0])
        } else if is_qr_hovered {
            ([0.14, 0.46, 0.30, 0.95], [0.3, 0.95, 0.6, 1.0])
        } else {
            ([0.08, 0.30, 0.20, 0.80], [0.2, 0.75, 0.45, 0.7])
        };
        self.add_rect(qr.x, qr.y, qr.w, qr.h, qr_bg);
        self.add_rect_outline(
            qr.x,
            qr.y,
            qr.w,
            qr.h,
            if is_qr_hovered { 1.5 } else { 1.0 },
            qr_border,
        );
        self.add_text(
            qr.x + 12.0,
            qr.y + 8.0,
            1.15,
            "▶ FREE ROAM SIMULATION [CLICK / ESC]",
            [0.35, 1.0, 0.65, 1.0],
        );

        for (i, &(label, sub)) in options.iter().enumerate() {
            let rect = layout.option_rects[i];
            let is_disabled = i == 0 && !continue_enabled;
            let is_pressed = pressed_opt == Some(i) && !is_disabled;
            let is_hovered = hovered_opt == Some(i) && !is_disabled;
            let is_sel = selected_opt == i && !is_disabled;

            let (bg, border, text_col, sub_col) = if is_disabled {
                (
                    [0.05, 0.07, 0.10, 0.50],
                    [0.18, 0.22, 0.30, 0.4],
                    [0.45, 0.50, 0.55, 0.6],
                    [0.35, 0.40, 0.45, 0.5],
                )
            } else if is_pressed {
                (
                    [0.22, 0.45, 0.75, 0.98],
                    [0.55, 0.95, 1.0, 1.0],
                    [1.0, 1.0, 1.0, 1.0],
                    [0.9, 0.98, 1.0, 1.0],
                )
            } else if is_hovered {
                (
                    [0.18, 0.38, 0.65, 0.95],
                    [0.45, 0.92, 1.0, 1.0],
                    [1.0, 1.0, 1.0, 1.0],
                    [0.85, 0.95, 1.0, 1.0],
                )
            } else if is_sel {
                (
                    [0.14, 0.30, 0.50, 0.90],
                    [0.35, 0.85, 1.0, 1.0],
                    [1.0, 1.0, 1.0, 1.0],
                    [0.80, 0.92, 1.0, 1.0],
                )
            } else {
                (
                    [0.07, 0.09, 0.14, 0.75],
                    [0.20, 0.28, 0.40, 0.5],
                    [0.88, 0.92, 0.96, 0.95],
                    [0.55, 0.65, 0.75, 0.75],
                )
            };

            self.add_rect(rect.x, rect.y, rect.w, rect.h, bg);
            self.add_rect_outline(
                rect.x,
                rect.y,
                rect.w,
                rect.h,
                if is_pressed || is_hovered || is_sel {
                    2.0
                } else {
                    1.0
                },
                border,
            );

            if is_sel || is_hovered {
                self.add_rect(rect.x, rect.y, 5.0, rect.h, [0.0, 0.9, 1.0, 1.0]);
                self.add_text(
                    rect.x + rect.w - 24.0,
                    rect.y + 14.0,
                    1.4,
                    ">",
                    [0.0, 0.9, 1.0, 1.0],
                );
            }

            let display_label = if is_disabled {
                "CONTINUE CAREER (NO SAVE FOUND)"
            } else {
                label
            };
            let display_sub = if is_disabled {
                "No saved progress on disk — select Free Roam or Contracts"
            } else {
                sub
            };

            self.add_text(rect.x + 20.0, rect.y + 12.0, 1.35, display_label, text_col);
            if !display_sub.is_empty() {
                self.add_text(rect.x + 20.0, rect.y + 28.0, 0.95, display_sub, sub_col);
            }
        }

        // Right Info Card / Feature Highlights (Translucent)
        let info_x = card.x + card.w + 32.0;
        let info_w = (screen_w - info_x - 40.0).max(300.0);
        let info_h = card.h;
        if info_w >= 280.0 {
            self.add_panel(info_x, card.y, info_w, info_h, [0.15, 0.55, 0.95, 0.8]);
            self.add_text(
                info_x + 18.0,
                card.y + 16.0,
                1.3,
                "SAN FRANCISCO PILOT & 38 US CITIES",
                [1.0, 1.0, 1.0, 1.0],
            );
            self.add_text(
                info_x + 18.0,
                card.y + 38.0,
                1.0,
                "• Real surveyed USGS 3DEP topography & OSM road networks",
                [0.85, 0.90, 0.95, 0.95],
            );
            self.add_text(
                info_x + 18.0,
                card.y + 56.0,
                1.0,
                "• Taxi, courier, heavy freight & free-roam careers",
                [0.85, 0.90, 0.95, 0.95],
            );
            self.add_text(
                info_x + 18.0,
                card.y + 74.0,
                1.0,
                "• 10 multimodal vehicles: Cars, Bus, Truck, Train, Plane, Boat",
                [0.85, 0.90, 0.95, 0.95],
            );
            self.add_text(
                info_x + 18.0,
                card.y + 92.0,
                1.0,
                "• IDM autonomous traffic simulation & traffic signals",
                [0.85, 0.90, 0.95, 0.95],
            );
            self.add_text(
                info_x + 18.0,
                card.y + 110.0,
                1.0,
                "• Full garage upgrades, earnings, achievements & quicksave",
                [0.85, 0.90, 0.95, 0.95],
            );
            self.add_text(
                info_x + 18.0,
                card.y + 135.0,
                1.0,
                "Active City: San Francisco Bay Area (CA)",
                [0.2, 0.9, 0.5, 1.0],
            );
            self.add_text(
                info_x + 18.0,
                card.y + 152.0,
                1.0,
                "Surveyed Anchor: Transamerica Pyramid (37.7952 N, 122.4028 W)",
                [0.70, 0.80, 0.90, 0.9],
            );
        }

        // Translucent Footer Bar
        let footer_y = screen_h - 48.0;
        self.add_rect(0.0, footer_y, screen_w, 48.0, [0.04, 0.06, 0.09, 0.55]);
        self.add_line(
            0.0,
            footer_y,
            screen_w,
            footer_y,
            1.0,
            [0.20, 0.30, 0.45, 0.5],
        );
        self.add_text(
            40.0,
            footer_y + 16.0,
            1.15,
            "CONTROLS: Mouse Click / [↑/↓] Select  |  [Enter] Confirm  |  [Esc] Quick Free Roam / Exit  |  [F3] Dev Overlay",
            [0.35, 0.85, 1.0, 1.0],
        );
        let ver_str = "v1.0.0-STAGE1-RESTORED";
        let (ver_w, _) = self.measure_text_proportional(ver_str, 1.05 * 13.5);
        self.add_text(
            screen_w - ver_w - 40.0,
            footer_y + 16.0,
            1.05,
            ver_str,
            [0.6, 0.7, 0.8, 0.7],
        );
    }

    /// Adds a modern centered pause menu window with responsive backdrop.
    pub fn add_pause_menu(
        &mut self,
        screen_w: f32,
        screen_h: f32,
        selected_opt: usize,
        options: &[&str],
        city_name: &str,
    ) {
        self.add_pause_menu_interactive(
            screen_w,
            screen_h,
            selected_opt,
            None,
            None,
            None,
            None,
            options,
            city_name,
        );
    }

    /// Adds a modern centered pause menu window with responsive backdrop and pointer feedback.
    #[allow(clippy::too_many_arguments)]
    pub fn add_pause_menu_interactive(
        &mut self,
        screen_w: f32,
        screen_h: f32,
        selected_opt: usize,
        hovered_opt: Option<usize>,
        pressed_opt: Option<usize>,
        hovered_btn: Option<PauseMenuHit>,
        pressed_btn: Option<PauseMenuHit>,
        options: &[&str],
        city_name: &str,
    ) {
        let layout = PauseMenuLayout::compute(screen_w, screen_h, options.len());
        let modal_w = layout.modal_rect.w;
        let modal_h = layout.modal_rect.h;
        let (mx, my) = self.add_modal(
            screen_w,
            screen_h,
            modal_w,
            modal_h,
            "SIMULATION PAUSED — CONTROL CENTER",
            [0.2, 0.7, 1.0, 1.0],
        );

        // Header Action: Resume Button
        let rb = layout.resume_btn;
        let is_rb_pressed = pressed_btn == Some(PauseMenuHit::Resume);
        let is_rb_hovered = hovered_btn == Some(PauseMenuHit::Resume);
        let (rb_bg, rb_border) = if is_rb_pressed {
            ([0.18, 0.55, 0.35, 0.98], [0.4, 1.0, 0.7, 1.0])
        } else if is_rb_hovered {
            ([0.14, 0.45, 0.30, 0.95], [0.3, 0.95, 0.6, 1.0])
        } else {
            ([0.08, 0.28, 0.20, 0.85], [0.2, 0.75, 0.45, 0.7])
        };
        self.add_rect(rb.x, rb.y, rb.w, rb.h, rb_bg);
        self.add_rect_outline(rb.x, rb.y, rb.w, rb.h, 1.0, rb_border);
        self.add_text(
            rb.x + 10.0,
            rb.y + 6.0,
            1.05,
            "▶ RESUME",
            [0.35, 1.0, 0.65, 1.0],
        );

        // Header Action: Close [X] Button
        let cb = layout.close_btn;
        let is_cb_hovered = hovered_btn == Some(PauseMenuHit::Close);
        let (cb_bg, cb_border) = if is_cb_hovered {
            ([0.35, 0.15, 0.15, 0.95], [1.0, 0.5, 0.5, 1.0])
        } else {
            ([0.18, 0.08, 0.08, 0.80], [0.6, 0.3, 0.3, 0.6])
        };
        self.add_rect(cb.x, cb.y, cb.w, cb.h, cb_bg);
        self.add_rect_outline(cb.x, cb.y, cb.w, cb.h, 1.0, cb_border);
        self.add_text(cb.x + 8.0, cb.y + 6.0, 1.1, "X", [1.0, 0.7, 0.7, 1.0]);

        // Subtitle with active location
        self.add_text(
            mx + 18.0,
            my + 50.0,
            1.05,
            &format!(
                "ACTIVE REGION: {}  |  PRESS [ESC] OR CLICK TO RESUME",
                city_name
            ),
            [0.4, 0.85, 1.0, 0.9],
        );

        for (i, &opt) in options.iter().enumerate() {
            let rect = layout.option_rects[i];
            let is_sel = i == selected_opt;
            let is_hovered = hovered_opt == Some(i);
            let is_pressed = pressed_opt == Some(i);

            let (bg, border, text_col) = if is_pressed {
                (
                    [0.22, 0.45, 0.75, 0.98],
                    [0.55, 0.95, 1.0, 1.0],
                    [1.0, 1.0, 1.0, 1.0],
                )
            } else if is_hovered {
                (
                    [0.18, 0.38, 0.65, 0.95],
                    [0.45, 0.92, 1.0, 1.0],
                    [1.0, 1.0, 1.0, 1.0],
                )
            } else if is_sel {
                (
                    [0.15, 0.30, 0.52, 0.95],
                    [0.35, 0.85, 1.0, 1.0],
                    [1.0, 1.0, 1.0, 1.0],
                )
            } else {
                (
                    [0.08, 0.11, 0.17, 0.80],
                    [0.22, 0.28, 0.38, 0.5],
                    [0.85, 0.90, 0.95, 0.9],
                )
            };

            self.add_rect(rect.x, rect.y, rect.w, rect.h, bg);
            self.add_rect_outline(
                rect.x,
                rect.y,
                rect.w,
                rect.h,
                if is_pressed || is_hovered || is_sel {
                    1.5
                } else {
                    0.5
                },
                border,
            );

            if is_sel || is_hovered {
                self.add_rect(rect.x, rect.y, 4.0, rect.h, [0.0, 0.9, 1.0, 1.0]);
            }

            self.add_text(rect.x + 16.0, rect.y + 10.0, 1.25, opt, text_col);
        }

        // Footer
        let footer_y = my + modal_h - 34.0;
        self.add_rect(mx, footer_y, modal_w, 34.0, [0.08, 0.10, 0.15, 0.98]);
        self.add_rect(mx, footer_y, modal_w, 1.0, [0.25, 0.35, 0.50, 0.6]);
        self.add_text(
            mx + 18.0,
            footer_y + 10.0,
            1.05,
            "[↑/↓] or Scroll Navigate  |  Click / [Enter] Select  |  [Esc] Resume",
            [0.35, 0.85, 1.0, 1.0],
        );
    }

    /// Adds a modern dispatch and career contract browser modal.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn add_job_browser_modal(
        &mut self,
        screen_w: f32,
        screen_h: f32,
        selected_idx: usize,
        jobs: &[(&str, &str, &str, &str, f64, f64, u32, &str, bool)],
        active_job_id: Option<&str>,
    ) {
        self.add_job_browser_modal_interactive(
            screen_w,
            screen_h,
            selected_idx,
            None,
            None,
            None,
            None,
            jobs,
            active_job_id,
        );
    }

    /// Adds an interactive dispatch and career contract browser modal with hover/pressed feedback and buttons.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn add_job_browser_modal_interactive(
        &mut self,
        screen_w: f32,
        screen_h: f32,
        selected_idx: usize,
        hovered_idx: Option<usize>,
        pressed_idx: Option<usize>,
        hovered_btn: Option<JobBrowserHit>,
        pressed_btn: Option<JobBrowserHit>,
        jobs: &[(&str, &str, &str, &str, f64, f64, u32, &str, bool)],
        active_job_id: Option<&str>,
    ) {
        let layout = JobBrowserLayout::compute(screen_w, screen_h, selected_idx, jobs.len());
        let modal_w = layout.modal_rect.w;
        let modal_h = layout.modal_rect.h;
        let (mx, my) = self.add_modal(
            screen_w,
            screen_h,
            modal_w,
            modal_h,
            "REGIONAL DISPATCH & CAREER CONTRACTS",
            [0.2, 0.9, 0.5, 1.0],
        );

        // Column Headers
        let header_y = my + 50.0;
        self.add_rect(
            mx + 12.0,
            header_y,
            modal_w - 24.0,
            24.0,
            [0.12, 0.16, 0.24, 0.9],
        );
        self.add_text(
            mx + 20.0,
            header_y + 6.0,
            1.05,
            "CAREER",
            [0.6, 0.75, 0.9, 1.0],
        );
        self.add_text(
            mx + 140.0,
            header_y + 6.0,
            1.05,
            "CONTRACT MISSION TITLE",
            [0.6, 0.75, 0.9, 1.0],
        );
        self.add_text(
            mx + 460.0,
            header_y + 6.0,
            1.05,
            "DISTANCE",
            [0.6, 0.75, 0.9, 1.0],
        );
        self.add_text(
            mx + 560.0,
            header_y + 6.0,
            1.05,
            "PAYOUT",
            [0.6, 0.75, 0.9, 1.0],
        );
        self.add_text(
            mx + 660.0,
            header_y + 6.0,
            1.05,
            "XP / TIER",
            [0.6, 0.75, 0.9, 1.0],
        );
        self.add_text(
            mx + 740.0,
            header_y + 6.0,
            1.05,
            "STATUS",
            [0.6, 0.75, 0.9, 1.0],
        );

        let list_y = header_y + 28.0;
        let item_h = 38.0;
        let start_idx = layout.start_idx;
        let visible_jobs = &jobs[start_idx..jobs.len().min(start_idx + layout.visible_count)];

        for (i, &(id, career, title, _route, dist_km, payout, xp, _diff, can_accept)) in
            visible_jobs.iter().enumerate()
        {
            let actual_idx = start_idx + i;
            let is_sel = actual_idx == selected_idx;
            let is_hovered = hovered_idx == Some(actual_idx);
            let is_pressed = pressed_idx == Some(actual_idx);
            let is_active = active_job_id == Some(id);
            let iy = list_y + i as f32 * (item_h + 3.0);

            let (bg, border) = if is_pressed {
                ([0.24, 0.45, 0.75, 0.98], [0.55, 0.95, 1.0, 1.0])
            } else if is_hovered {
                ([0.20, 0.38, 0.65, 0.95], [0.45, 0.90, 1.0, 1.0])
            } else if is_sel {
                ([0.18, 0.35, 0.55, 0.98], [0.35, 0.85, 1.0, 1.0])
            } else if is_active {
                ([0.10, 0.28, 0.20, 0.90], [0.2, 0.85, 0.5, 0.8])
            } else {
                ([0.08, 0.11, 0.17, 0.85], [0.20, 0.25, 0.35, 0.4])
            };

            self.add_rect(mx + 12.0, iy, modal_w - 24.0, item_h, bg);
            self.add_rect_outline(
                mx + 12.0,
                iy,
                modal_w - 24.0,
                item_h,
                if is_sel || is_hovered { 1.5 } else { 0.5 },
                border,
            );

            if is_sel || is_hovered {
                self.add_rect(mx + 12.0, iy, 4.0, item_h, [0.0, 0.9, 1.0, 1.0]);
            }

            let short_career = if career.contains("Taxi") {
                "TAXI"
            } else if career.contains("Courier")
                || career.contains("Package")
                || career.contains("Delivery")
            {
                "COURIER"
            } else if career.contains("Freight") {
                "FREIGHT"
            } else {
                "EXPLORE"
            };
            self.add_text(
                mx + 20.0,
                iy + 11.0,
                1.1,
                short_career,
                [0.3, 0.85, 1.0, 1.0],
            );

            let trunc_title = if title.len() > 36 {
                &title[..36]
            } else {
                title
            };
            self.add_text(
                mx + 130.0,
                iy + 11.0,
                1.15,
                trunc_title,
                [1.0, 1.0, 1.0, 1.0],
            );
            self.add_text(
                mx + 460.0,
                iy + 11.0,
                1.1,
                &format!("{:.1} km", dist_km),
                [0.8, 0.85, 0.9, 1.0],
            );
            self.add_text(
                mx + 560.0,
                iy + 11.0,
                1.15,
                &format!("${:.0}", payout),
                [1.0, 0.85, 0.2, 1.0],
            );
            self.add_text(
                mx + 660.0,
                iy + 11.0,
                1.0,
                &format!("+{} XP", xp),
                [0.3, 0.9, 0.6, 1.0],
            );

            let (stat_str, stat_col) = if is_active {
                ("ACTIVE", [0.2, 0.95, 0.5, 1.0])
            } else if can_accept {
                ("READY", [0.95, 0.85, 0.3, 1.0])
            } else {
                ("LOCKED", [0.6, 0.6, 0.6, 0.8])
            };
            self.add_text(mx + 740.0, iy + 11.0, 1.05, stat_str, stat_col);
        }

        // Briefing Preview Card for selected job
        if selected_idx < jobs.len() {
            let (_id, career, title, route, dist_km, payout, xp, diff, _can_accept) =
                jobs[selected_idx];
            let prev_y = list_y + layout.visible_count as f32 * (item_h + 3.0) + 6.0;
            let prev_h = modal_h - (prev_y - my) - 42.0;
            if prev_h >= 50.0 {
                self.add_rect(
                    mx + 12.0,
                    prev_y,
                    modal_w - 24.0,
                    prev_h,
                    [0.06, 0.08, 0.12, 0.95],
                );
                self.add_rect_outline(
                    mx + 12.0,
                    prev_y,
                    modal_w - 24.0,
                    prev_h,
                    1.0,
                    [0.25, 0.35, 0.50, 0.5],
                );
                self.add_text(
                    mx + 22.0,
                    prev_y + 10.0,
                    1.15,
                    &format!("CONTRACT: {}  [{}]", title, career),
                    [1.0, 1.0, 1.0, 1.0],
                );
                self.add_text(
                    mx + 22.0,
                    prev_y + 28.0,
                    1.0,
                    &format!("ROUTE: {}  |  DIFFICULTY: {}", route, diff),
                    [0.75, 0.85, 0.95, 0.9],
                );
                self.add_text(
                    mx + 22.0,
                    prev_y + 44.0,
                    1.0,
                    &format!(
                        "ESTIMATED DISTANCE: {:.2} km  |  REWARD: ${:.2}  |  +{} XP",
                        dist_km, payout, xp
                    ),
                    [1.0, 0.85, 0.2, 1.0],
                );
            }
        }

        // Footer
        let footer_y = my + modal_h - 36.0;
        self.add_rect(mx, footer_y, modal_w, 36.0, [0.08, 0.10, 0.15, 0.98]);
        self.add_rect(mx, footer_y, modal_w, 1.0, [0.25, 0.35, 0.50, 0.6]);
        self.add_text(
            mx + 20.0,
            footer_y + 11.0,
            1.05,
            "[↑/↓] or Scroll Navigate  |  Click Item to Select",
            [0.35, 0.85, 1.0, 1.0],
        );

        // Clickable Accept Contract Button
        let ab = layout.accept_btn;
        let is_ab_hovered = hovered_btn == Some(JobBrowserHit::AcceptContract);
        let is_ab_pressed = pressed_btn == Some(JobBrowserHit::AcceptContract);
        let (ab_bg, ab_border) = if is_ab_pressed {
            ([0.18, 0.55, 0.35, 0.98], [0.4, 1.0, 0.7, 1.0])
        } else if is_ab_hovered {
            ([0.14, 0.45, 0.30, 0.95], [0.3, 0.95, 0.6, 1.0])
        } else {
            ([0.08, 0.28, 0.20, 0.85], [0.2, 0.75, 0.45, 0.7])
        };
        self.add_rect(ab.x, ab.y, ab.w, ab.h, ab_bg);
        self.add_rect_outline(ab.x, ab.y, ab.w, ab.h, 1.0, ab_border);
        self.add_text(
            ab.x + 12.0,
            ab.y + 7.0,
            1.1,
            "▶ ACCEPT CONTRACT",
            [0.3, 1.0, 0.6, 1.0],
        );

        // Clickable Back Button
        let bb = layout.back_btn;
        let is_bb_hovered = hovered_btn == Some(JobBrowserHit::Back);
        let is_bb_pressed = pressed_btn == Some(JobBrowserHit::Back);
        let (bb_bg, bb_border) = if is_bb_pressed {
            ([0.35, 0.15, 0.15, 0.98], [1.0, 0.5, 0.5, 1.0])
        } else if is_bb_hovered {
            ([0.28, 0.12, 0.12, 0.95], [0.95, 0.4, 0.4, 1.0])
        } else {
            ([0.16, 0.08, 0.08, 0.85], [0.70, 0.3, 0.3, 0.7])
        };
        self.add_rect(bb.x, bb.y, bb.w, bb.h, bb_bg);
        self.add_rect_outline(bb.x, bb.y, bb.w, bb.h, 1.0, bb_border);
        self.add_text(bb.x + 10.0, bb.y + 7.0, 1.1, "◀ BACK", [1.0, 0.6, 0.6, 1.0]);
    }

    /// Adds a modern interactive vehicle garage and fleet inspection modal.
    pub fn add_garage_modal(
        &mut self,
        screen_w: f32,
        screen_h: f32,
        selected_idx: usize,
        vehicles: &[(&str, &str, &str, f32, f32, f32, bool)],
    ) {
        self.add_garage_modal_interactive(
            screen_w,
            screen_h,
            selected_idx,
            None,
            None,
            None,
            None,
            vehicles,
        );
    }

    /// Adds an interactive vehicle garage and fleet inspection modal with pointer feedback.
    #[allow(clippy::too_many_arguments)]
    pub fn add_garage_modal_interactive(
        &mut self,
        screen_w: f32,
        screen_h: f32,
        selected_idx: usize,
        hovered_idx: Option<usize>,
        pressed_idx: Option<usize>,
        hovered_btn: Option<GarageHit>,
        pressed_btn: Option<GarageHit>,
        vehicles: &[(&str, &str, &str, f32, f32, f32, bool)],
    ) {
        self.add_garage_modal_interactive_with_color(
            screen_w,
            screen_h,
            selected_idx,
            0,
            hovered_idx,
            pressed_idx,
            hovered_btn,
            pressed_btn,
            vehicles,
        );
    }

    /// Adds an interactive vehicle garage modal with custom paint palette selection.
    #[allow(clippy::too_many_arguments)]
    pub fn add_garage_modal_interactive_with_color(
        &mut self,
        screen_w: f32,
        screen_h: f32,
        selected_idx: usize,
        selected_color_idx: usize,
        hovered_idx: Option<usize>,
        pressed_idx: Option<usize>,
        hovered_btn: Option<GarageHit>,
        pressed_btn: Option<GarageHit>,
        vehicles: &[(&str, &str, &str, f32, f32, f32, bool)],
    ) {
        let layout = GarageLayout::compute(screen_w, screen_h, vehicles.len());
        let modal_w = layout.modal_rect.w;
        let modal_h = layout.modal_rect.h;
        let (mx, my) = self.add_modal(
            screen_w,
            screen_h,
            modal_w,
            modal_h,
            "VEHICLE GARAGE & MULTIMODAL FLEET",
            [1.0, 0.45, 0.1, 1.0],
        );

        // Headers
        let header_y = my + 50.0;
        self.add_rect(
            mx + 12.0,
            header_y,
            modal_w - 24.0,
            24.0,
            [0.12, 0.16, 0.24, 0.9],
        );
        self.add_text(
            mx + 20.0,
            header_y + 6.0,
            1.05,
            "VEHICLE MODEL",
            [0.6, 0.75, 0.9, 1.0],
        );
        self.add_text(
            mx + 240.0,
            header_y + 6.0,
            1.05,
            "CLASS / CATEGORY",
            [0.6, 0.75, 0.9, 1.0],
        );
        self.add_text(
            mx + 420.0,
            header_y + 6.0,
            1.05,
            "DRIVETRAIN",
            [0.6, 0.75, 0.9, 1.0],
        );
        self.add_text(
            mx + 540.0,
            header_y + 6.0,
            1.05,
            "CURB MASS",
            [0.6, 0.75, 0.9, 1.0],
        );
        self.add_text(
            mx + 640.0,
            header_y + 6.0,
            1.05,
            "TOP SPEED",
            [0.6, 0.75, 0.9, 1.0],
        );
        self.add_text(
            mx + 720.0,
            header_y + 6.0,
            1.05,
            "STATUS",
            [0.6, 0.75, 0.9, 1.0],
        );

        let list_y = header_y + 28.0;
        let item_h = 36.0;

        for (i, &(name, cat, drive, mass, _hp, top_speed, is_active)) in vehicles.iter().enumerate()
        {
            let is_sel = i == selected_idx;
            let is_hovered = hovered_idx == Some(i);
            let is_pressed = pressed_idx == Some(i);
            let iy = list_y + i as f32 * (item_h + 3.0);

            let (bg, border) = if is_pressed {
                ([0.28, 0.38, 0.60, 0.98], [0.6, 0.95, 1.0, 1.0])
            } else if is_hovered {
                ([0.22, 0.34, 0.55, 0.95], [0.45, 0.90, 1.0, 1.0])
            } else if is_sel {
                ([0.20, 0.30, 0.48, 0.98], [0.4, 0.85, 1.0, 1.0])
            } else if is_active {
                ([0.15, 0.25, 0.35, 0.90], [0.3, 0.75, 0.9, 0.7])
            } else {
                ([0.08, 0.11, 0.16, 0.85], [0.20, 0.25, 0.35, 0.4])
            };

            self.add_rect(mx + 12.0, iy, modal_w - 24.0, item_h, bg);
            self.add_rect_outline(
                mx + 12.0,
                iy,
                modal_w - 24.0,
                item_h,
                if is_sel || is_hovered { 1.5 } else { 0.5 },
                border,
            );

            if is_sel || is_hovered {
                self.add_rect(mx + 12.0, iy, 4.0, item_h, [1.0, 0.5, 0.1, 1.0]);
            }

            self.add_text(mx + 20.0, iy + 10.0, 1.2, name, [1.0, 1.0, 1.0, 1.0]);
            self.add_text(mx + 240.0, iy + 10.0, 1.1, cat, [0.8, 0.85, 0.9, 1.0]);
            self.add_text(mx + 420.0, iy + 10.0, 1.05, drive, [0.3, 0.85, 1.0, 1.0]);
            self.add_text(
                mx + 540.0,
                iy + 10.0,
                1.05,
                &format!("{:.0} kg", mass),
                [0.85, 0.85, 0.88, 1.0],
            );
            self.add_text(
                mx + 640.0,
                iy + 10.0,
                1.05,
                &format!("{:.0} km/h", top_speed),
                [1.0, 0.85, 0.2, 1.0],
            );

            let (stat_str, stat_col) = if is_active {
                ("ACTIVE", [0.2, 0.95, 0.5, 1.0])
            } else {
                ("SELECT", [0.6, 0.7, 0.8, 0.7])
            };
            self.add_text(mx + 720.0, iy + 10.0, 1.0, stat_str, stat_col);
        }

        // Footer with paint palette swatches
        let footer_y = my + modal_h - 40.0;
        self.add_rect(mx, footer_y, modal_w, 40.0, [0.08, 0.10, 0.15, 0.98]);
        self.add_rect(mx, footer_y, modal_w, 1.0, [0.25, 0.35, 0.50, 0.6]);

        let color_name =
            GARAGE_PAINT_PALETTE[selected_color_idx.min(GARAGE_PAINT_PALETTE.len() - 1)].0;
        self.add_text(
            mx + 16.0,
            footer_y + 12.0,
            1.0,
            &format!("PAINT: {}", color_name),
            [1.0, 0.75, 0.25, 1.0],
        );

        for (c_idx, rect) in layout.color_rects.iter().enumerate() {
            let col = GARAGE_PAINT_PALETTE[c_idx].1;
            self.add_rect(rect.x + 80.0, rect.y, rect.w, rect.h, col);
            if c_idx == selected_color_idx {
                self.add_rect_outline(
                    rect.x + 78.0,
                    rect.y - 2.0,
                    rect.w + 4.0,
                    rect.h + 4.0,
                    2.0,
                    [1.0, 1.0, 1.0, 1.0],
                );
            } else {
                self.add_rect_outline(
                    rect.x + 80.0,
                    rect.y,
                    rect.w,
                    rect.h,
                    1.0,
                    [0.3, 0.4, 0.5, 0.7],
                );
            }
        }

        // Clickable Select & Spawn Button
        let sb = layout.select_btn;
        let is_sb_hovered = hovered_btn == Some(GarageHit::SelectAndSpawn);
        let is_sb_pressed = pressed_btn == Some(GarageHit::SelectAndSpawn);
        let (sb_bg, sb_border) = if is_sb_pressed {
            ([0.35, 0.25, 0.10, 0.98], [1.0, 0.7, 0.3, 1.0])
        } else if is_sb_hovered {
            ([0.28, 0.20, 0.08, 0.95], [1.0, 0.6, 0.2, 1.0])
        } else {
            ([0.20, 0.12, 0.05, 0.85], [0.8, 0.45, 0.1, 0.7])
        };
        self.add_rect(sb.x, sb.y, sb.w, sb.h, sb_bg);
        self.add_rect_outline(sb.x, sb.y, sb.w, sb.h, 1.0, sb_border);
        self.add_text(
            sb.x + 12.0,
            sb.y + 7.0,
            1.1,
            "▶ SPAWN VEHICLE",
            [1.0, 0.75, 0.3, 1.0],
        );

        // Clickable Back Button
        let bb = layout.back_btn;
        let is_bb_hovered = hovered_btn == Some(GarageHit::Back);
        let is_bb_pressed = pressed_btn == Some(GarageHit::Back);
        let (bb_bg, bb_border) = if is_bb_pressed {
            ([0.35, 0.15, 0.15, 0.98], [1.0, 0.5, 0.5, 1.0])
        } else if is_bb_hovered {
            ([0.28, 0.12, 0.12, 0.95], [0.95, 0.4, 0.4, 1.0])
        } else {
            ([0.16, 0.08, 0.08, 0.85], [0.70, 0.3, 0.3, 0.7])
        };
        self.add_rect(bb.x, bb.y, bb.w, bb.h, bb_bg);
        self.add_rect_outline(bb.x, bb.y, bb.w, bb.h, 1.0, bb_border);
        self.add_text(bb.x + 10.0, bb.y + 7.0, 1.1, "◀ BACK", [1.0, 0.6, 0.6, 1.0]);
    }

    /// Adds a modern settings and user preferences modal.
    pub fn add_settings_modal(
        &mut self,
        screen_w: f32,
        screen_h: f32,
        selected_idx: usize,
        settings: &[(&str, &str, &str)],
    ) {
        self.add_settings_modal_interactive(
            screen_w,
            screen_h,
            selected_idx,
            None,
            None,
            None,
            None,
            settings,
        );
    }

    /// Adds an interactive settings and preferences modal with pointer feedback.
    #[allow(clippy::too_many_arguments)]
    pub fn add_settings_modal_interactive(
        &mut self,
        screen_w: f32,
        screen_h: f32,
        selected_idx: usize,
        hovered_idx: Option<usize>,
        pressed_idx: Option<usize>,
        hovered_btn: Option<SettingsHit>,
        pressed_btn: Option<SettingsHit>,
        settings: &[(&str, &str, &str)],
    ) {
        let layout = SettingsLayout::compute(screen_w, screen_h, settings.len());
        let modal_w = layout.modal_rect.w;
        let modal_h = layout.modal_rect.h;
        let (mx, my) = self.add_modal(
            screen_w,
            screen_h,
            modal_w,
            modal_h,
            "SETTINGS & SIMULATION PREFERENCES",
            [0.4, 0.6, 1.0, 1.0],
        );

        let list_y = my + 56.0;
        let item_h = 42.0;

        for (i, &(name, val, desc)) in settings.iter().enumerate() {
            let is_sel = i == selected_idx;
            let is_hovered = hovered_idx == Some(i);
            let is_pressed = pressed_idx == Some(i);
            let iy = list_y + i as f32 * (item_h + 4.0);

            let (bg, border) = if is_pressed {
                ([0.24, 0.42, 0.65, 0.98], [0.55, 0.95, 1.0, 1.0])
            } else if is_hovered {
                ([0.20, 0.36, 0.58, 0.95], [0.45, 0.90, 1.0, 1.0])
            } else if is_sel {
                ([0.18, 0.32, 0.52, 0.98], [0.35, 0.85, 1.0, 1.0])
            } else {
                ([0.08, 0.11, 0.16, 0.85], [0.20, 0.25, 0.35, 0.4])
            };

            self.add_rect(mx + 16.0, iy, modal_w - 32.0, item_h, bg);
            self.add_rect_outline(
                mx + 16.0,
                iy,
                modal_w - 32.0,
                item_h,
                if is_sel || is_hovered { 1.5 } else { 0.5 },
                border,
            );

            if is_sel || is_hovered {
                self.add_rect(mx + 16.0, iy, 4.0, item_h, [0.0, 0.9, 1.0, 1.0]);
            }

            self.add_text(mx + 26.0, iy + 8.0, 1.25, name, [1.0, 1.0, 1.0, 1.0]);
            self.add_text(mx + 26.0, iy + 24.0, 0.95, desc, [0.65, 0.75, 0.85, 0.8]);

            // Value badge on right
            let val_w = (val.len() as f32 * 6.0 * 1.2) + 16.0;
            let val_x = mx + modal_w - 32.0 - val_w - 12.0;
            self.add_rect(val_x, iy + 8.0, val_w, 24.0, [0.12, 0.18, 0.28, 0.9]);
            self.add_rect_outline(
                val_x,
                iy + 8.0,
                val_w,
                24.0,
                1.0,
                if is_sel || is_hovered {
                    [0.35, 0.85, 1.0, 1.0]
                } else {
                    [0.3, 0.4, 0.5, 0.5]
                },
            );
            self.add_text(val_x + 8.0, iy + 14.0, 1.15, val, [0.2, 0.9, 1.0, 1.0]);
        }

        // Footer
        let footer_y = my + modal_h - 36.0;
        self.add_rect(mx, footer_y, modal_w, 36.0, [0.08, 0.10, 0.15, 0.98]);
        self.add_rect(mx, footer_y, modal_w, 1.0, [0.25, 0.35, 0.50, 0.6]);
        self.add_text(
            mx + 20.0,
            footer_y + 11.0,
            1.05,
            "[↑/↓] or Scroll  |  Click Item to Cycle Value",
            [0.35, 0.85, 1.0, 1.0],
        );

        // Clickable Back Button
        let bb = layout.back_btn;
        let is_bb_hovered = hovered_btn == Some(SettingsHit::Back);
        let is_bb_pressed = pressed_btn == Some(SettingsHit::Back);
        let (bb_bg, bb_border) = if is_bb_pressed {
            ([0.35, 0.15, 0.15, 0.98], [1.0, 0.5, 0.5, 1.0])
        } else if is_bb_hovered {
            ([0.28, 0.12, 0.12, 0.95], [0.95, 0.4, 0.4, 1.0])
        } else {
            ([0.16, 0.08, 0.08, 0.85], [0.70, 0.3, 0.3, 0.7])
        };
        self.add_rect(bb.x, bb.y, bb.w, bb.h, bb_bg);
        self.add_rect_outline(bb.x, bb.y, bb.w, bb.h, 1.0, bb_border);
        self.add_text(bb.x + 10.0, bb.y + 7.0, 1.1, "◀ BACK", [1.0, 0.6, 0.6, 1.0]);
    }

    /// Adds an interactive boarding/loading dwell progress card when vehicle is stopped at objective.
    pub fn add_dwell_progress(
        &mut self,
        screen_w: f32,
        screen_h: f32,
        progress: f32,
        operation_text: &str,
        is_stopped: bool,
    ) {
        let card_w = 420.0;
        let card_h = 76.0;
        let x = (screen_w - card_w) * 0.5;
        let y = screen_h * 0.68;

        let accent = if is_stopped {
            [0.1, 0.9, 0.4, 1.0]
        } else {
            [1.0, 0.2, 0.2, 1.0]
        };
        self.add_panel(x, y, card_w, card_h, accent);

        let status_title = if is_stopped {
            format!("{} ({:.0}%)", operation_text, progress * 100.0)
        } else {
            "STOP VEHICLE TO INTERACT!".to_string()
        };
        self.add_text(
            x + 16.0,
            y + 12.0,
            1.25,
            &status_title,
            if is_stopped {
                [1.0, 1.0, 1.0, 1.0]
            } else {
                [1.0, 0.3, 0.3, 1.0]
            },
        );

        let fill_col = if is_stopped {
            [0.1, 0.9, 0.4, 1.0]
        } else {
            [0.9, 0.3, 0.1, 1.0]
        };
        self.add_bar(
            x + 16.0,
            y + 34.0,
            card_w - 32.0,
            14.0,
            progress,
            fill_col,
            [0.12, 0.15, 0.22, 1.0],
        );

        let prompt = if is_stopped {
            "HOLD BRAKES AT COMPLETE STOP WHILE LOADING"
        } else {
            "APPLY BRAKES [S / Down] TO BRING SPEED TO 0"
        };
        self.add_text(x + 16.0, y + 54.0, 0.95, prompt, [0.75, 0.85, 0.95, 0.9]);
    }

    /// Adds a mission debrief summary modal when a contract is completed.
    #[allow(clippy::too_many_arguments)]
    pub fn add_debrief_modal(
        &mut self,
        screen_w: f32,
        screen_h: f32,
        title: &str,
        career: &str,
        base_pay: f64,
        tip_bonus: f64,
        speed_bonus: f64,
        net_pay: f64,
        xp_gained: u32,
        comfort_stars: f32,
    ) {
        let modal_w = 480.0;
        let modal_h = 320.0;
        let (mx, my) = self.add_modal(
            screen_w,
            screen_h,
            modal_w,
            modal_h,
            "CONTRACT MISSION COMPLETED!",
            [0.2, 0.95, 0.45, 1.0],
        );

        self.add_text(mx + 20.0, my + 54.0, 1.35, title, [1.0, 1.0, 1.0, 1.0]);
        self.add_text(
            mx + 20.0,
            my + 74.0,
            1.05,
            &format!("CAREER: {}", career),
            [0.35, 0.9, 1.0, 1.0],
        );

        self.add_line(
            mx + 20.0,
            my + 94.0,
            mx + modal_w - 20.0,
            my + 94.0,
            1.0,
            [0.25, 0.35, 0.50, 0.5],
        );

        self.add_text(
            mx + 20.0,
            my + 106.0,
            1.1,
            &format!("BASE CONTRACT PAYOUT:    ${:.2}", base_pay),
            [0.85, 0.88, 0.92, 1.0],
        );
        self.add_text(
            mx + 20.0,
            my + 126.0,
            1.1,
            &format!(
                "PASSENGER TIP BONUS:     +${:.2} ({:.1} Stars)",
                tip_bonus, comfort_stars
            ),
            [0.3, 0.9, 0.5, 1.0],
        );
        self.add_text(
            mx + 20.0,
            my + 146.0,
            1.1,
            &format!("ON-TIME SPEED BONUS:     +${:.2}", speed_bonus),
            [1.0, 0.85, 0.2, 1.0],
        );
        self.add_text(
            mx + 20.0,
            my + 166.0,
            1.1,
            &format!("XP REWARD:               +{} XP", xp_gained),
            [0.4, 0.9, 1.0, 1.0],
        );

        self.add_line(
            mx + 20.0,
            my + 188.0,
            mx + modal_w - 20.0,
            my + 188.0,
            1.5,
            [0.3, 0.85, 1.0, 0.8],
        );

        self.add_text(
            mx + 20.0,
            my + 200.0,
            1.45,
            &format!("TOTAL NET EARNINGS:  ${:.2}", net_pay),
            [1.0, 0.95, 0.2, 1.0],
        );

        // Footer button
        let btn_y = my + modal_h - 44.0;
        self.add_rect(
            mx + 20.0,
            btn_y,
            modal_w - 40.0,
            32.0,
            [0.15, 0.35, 0.55, 0.95],
        );
        self.add_rect_outline(
            mx + 20.0,
            btn_y,
            modal_w - 40.0,
            32.0,
            1.5,
            [0.35, 0.85, 1.0, 1.0],
        );
        self.add_text(
            mx + (modal_w - 40.0) * 0.5 - 60.0,
            btn_y + 9.0,
            1.2,
            "CONTINUE [ENTER]",
            [1.0, 1.0, 1.0, 1.0],
        );
    }

    /// Adds a modern automotive instrument cluster (Speedometer, RPM tachometer, gear badge, steer angle).
    #[allow(clippy::too_many_arguments)]
    pub fn add_speedometer_cluster(
        &mut self,
        x: f32,
        y: f32,
        speed_kph: f32,
        gear_str: &str,
        rpm: f32,
        max_rpm: f32,
        steer_deg: f32,
        metric_units: bool,
        vehicle_model: &str,
    ) {
        self.add_speedometer_cluster_assisted(
            x,
            y,
            speed_kph,
            gear_str,
            rpm,
            max_rpm,
            steer_deg,
            metric_units,
            vehicle_model,
            true,
            true,
            true,
            false,
        );
    }

    /// Adds an instrument cluster with real-time driving assist indicators (ABS, TCS, ESC, handbrake) and dynamic color ramp.
    #[allow(clippy::too_many_arguments)]
    pub fn add_speedometer_cluster_assisted(
        &mut self,
        x: f32,
        y: f32,
        speed_kph: f32,
        gear_str: &str,
        rpm: f32,
        max_rpm: f32,
        steer_deg: f32,
        metric_units: bool,
        vehicle_model: &str,
        abs_on: bool,
        tcs_on: bool,
        esc_on: bool,
        handbrake_on: bool,
    ) {
        let cluster_w = 345.0;
        let cluster_h = 138.0;

        self.add_panel(x, y, cluster_w, cluster_h, [1.0, 0.45, 0.10, 1.0]);

        // Display speed (KM/H or MPH)
        let (speed_val, unit_str) = if metric_units {
            (speed_kph.abs(), "KM/H")
        } else {
            (speed_kph.abs() * 0.621371, "MPH")
        };

        // Dynamic Speed Digits Color Ramp
        let speed_color = if speed_val > 105.0 {
            [1.0, 0.25, 0.25, 1.0] // Crimson warning
        } else if speed_val > 75.0 {
            [1.0, 0.85, 0.20, 1.0] // Amber
        } else {
            [1.0, 1.0, 1.0, 1.0] // Crisp white
        };

        // Large Speed Readout with proportional typography
        let speed_str = format!("{:>3.0}", speed_val);
        self.add_text_proportional(
            x + 16.0,
            y + 14.0,
            &speed_str,
            FontSize::Hero.pixels(),
            speed_color,
            FontWeight::Bold,
            true,
        );
        self.add_text_proportional(
            x + 122.0,
            y + 36.0,
            unit_str,
            FontSize::Body.pixels(),
            [0.95, 0.55, 0.20, 1.0],
            FontWeight::Bold,
            true,
        );

        // Transmission Gear Badge (High-contrast pill)
        let gear_x = x + cluster_w - 75.0;
        let gear_y = y + 14.0;
        let is_reverse = gear_str == "R";
        let gear_bg = if is_reverse {
            [0.85, 0.15, 0.15, 0.95]
        } else if gear_str == "P" || gear_str == "N" {
            [0.25, 0.30, 0.40, 0.95]
        } else {
            [0.10, 0.70, 0.95, 0.95]
        };
        self.add_rect(gear_x, gear_y, 55.0, 36.0, gear_bg);
        self.add_rect_outline(gear_x, gear_y, 55.0, 36.0, 1.5, [1.0, 1.0, 1.0, 0.6]);
        self.add_text_proportional(
            gear_x + 14.0,
            gear_y + 8.0,
            gear_str,
            FontSize::Title.pixels(),
            [1.0, 1.0, 1.0, 1.0],
            FontWeight::Bold,
            true,
        );

        // Tachometer RPM Gauge Bar
        let rpm_frac = (rpm / max_rpm.max(1000.0)).clamp(0.0, 1.0);
        let rpm_color = if rpm_frac > 0.85 {
            [0.95, 0.20, 0.20, 1.0] // Redline
        } else if rpm_frac > 0.65 {
            [0.95, 0.75, 0.15, 1.0] // Warning
        } else {
            [0.15, 0.85, 0.45, 1.0] // Cruising
        };
        self.add_text_proportional(
            x + 16.0,
            y + 60.0,
            &format!("{:.0} RPM", rpm),
            FontSize::Caption.pixels(),
            [0.8, 0.85, 0.9, 0.9],
            FontWeight::Regular,
            false,
        );
        self.add_bar(
            x + 16.0,
            y + 74.0,
            cluster_w - 32.0,
            8.0,
            rpm_frac,
            rpm_color,
            [0.12, 0.15, 0.20, 1.0],
        );

        // Driving Assist Status Badges: [ABS] [TCS] [ESC] [P-BRAKE]
        let chip_y = y + 88.0;
        let mut chip_x = x + 16.0;
        let assist_chips = [
            ("ABS", abs_on, [0.2, 0.85, 0.4, 0.9]),
            ("TCS", tcs_on, [0.2, 0.85, 0.4, 0.9]),
            ("ESC", esc_on, [0.2, 0.85, 0.4, 0.9]),
            ("P-BRAKE", handbrake_on, [0.95, 0.25, 0.25, 0.95]),
        ];
        for (label, active, active_col) in assist_chips {
            let (bg, border, text_col) = if active {
                (
                    [
                        active_col[0] * 0.3,
                        active_col[1] * 0.3,
                        active_col[2] * 0.3,
                        0.8,
                    ],
                    active_col,
                    [1.0, 1.0, 1.0, 1.0],
                )
            } else {
                (
                    [0.08, 0.10, 0.14, 0.6],
                    [0.25, 0.30, 0.40, 0.4],
                    [0.45, 0.50, 0.55, 0.6],
                )
            };
            let chip_w = label.len() as f32 * 6.0 + 10.0;
            self.add_rect(chip_x, chip_y, chip_w, 16.0, bg);
            self.add_rect_outline(chip_x, chip_y, chip_w, 16.0, 0.8, border);
            self.add_text_proportional(
                chip_x + 5.0,
                chip_y + 3.0,
                label,
                9.0,
                text_col,
                FontWeight::Bold,
                false,
            );
            chip_x += chip_w + 6.0;
        }

        // Steer angle & Vehicle model subtitle
        let steer_str = format!("STEER: {:>+3.0} deg", steer_deg);
        self.add_text_proportional(
            x + 16.0,
            y + 114.0,
            &steer_str,
            FontSize::Caption.pixels(),
            [0.5, 0.8, 1.0, 0.9],
            FontWeight::Regular,
            false,
        );
        self.add_text_proportional(
            x + 140.0,
            y + 114.0,
            vehicle_model,
            FontSize::Caption.pixels(),
            [0.75, 0.75, 0.75, 0.8],
            FontWeight::Regular,
            false,
        );
    }

    /// Adds a modern location pill banner displaying city district, street, and cardinal heading.
    pub fn add_location_pill(
        &mut self,
        x: f32,
        y: f32,
        district: &str,
        street: &str,
        heading_cardinal: &str,
    ) {
        let pill_w = 280.0;
        let pill_h = 42.0;
        self.add_panel(x, y, pill_w, pill_h, [0.2, 0.7, 1.0, 0.9]);
        // Compass badge
        self.add_rect(x + 8.0, y + 8.0, 34.0, 26.0, [0.10, 0.35, 0.65, 0.9]);
        self.add_rect_outline(x + 8.0, y + 8.0, 34.0, 26.0, 1.0, [0.4, 0.8, 1.0, 0.7]);
        self.add_text_proportional(
            x + 14.0,
            y + 12.0,
            heading_cardinal,
            FontSize::Body.pixels(),
            [1.0, 1.0, 1.0, 1.0],
            FontWeight::Bold,
            true,
        );
        // District & Street text
        self.add_text_proportional(
            x + 50.0,
            y + 8.0,
            district,
            FontSize::Body.pixels(),
            [0.4, 0.9, 1.0, 1.0],
            FontWeight::Bold,
            true,
        );
        self.add_text_proportional(
            x + 50.0,
            y + 23.0,
            street,
            FontSize::Caption.pixels(),
            [0.85, 0.90, 0.95, 0.85],
            FontWeight::Regular,
            false,
        );
    }

    /// Adds a stylish contextual action prompt chip (e.g. above speedometer or bottom-center).
    pub fn add_contextual_action_prompt(
        &mut self,
        cx: f32,
        y: f32,
        key_badge: &str,
        action_text: &str,
    ) {
        let badge_w = (key_badge.len() as f32 * 9.0 + 16.0).max(36.0);
        let text_w = action_text.len() as f32 * 7.5;
        let total_w = badge_w + text_w + 24.0;
        let x = cx - total_w * 0.5;
        let h = 32.0;

        self.add_panel(x, y, total_w, h, [0.15, 0.85, 0.55, 0.95]);
        self.add_rect(
            x + 5.0,
            y + 5.0,
            badge_w,
            h - 10.0,
            [0.12, 0.55, 0.35, 0.95],
        );
        self.add_rect_outline(
            x + 5.0,
            y + 5.0,
            badge_w,
            h - 10.0,
            1.0,
            [0.4, 1.0, 0.7, 0.8],
        );
        self.add_text_proportional(
            x + 10.0,
            y + 8.0,
            key_badge,
            FontSize::Body.pixels(),
            [1.0, 1.0, 1.0, 1.0],
            FontWeight::Bold,
            true,
        );
        self.add_text_proportional(
            x + badge_w + 14.0,
            y + 9.0,
            action_text,
            FontSize::Body.pixels(),
            [0.95, 1.0, 0.95, 1.0],
            FontWeight::Regular,
            true,
        );
    }

    /// Adds a tactical GPS circular radar minimap widget showing vehicle orientation, heading, and waypoints.
    pub fn add_gps_minimap(
        &mut self,
        cx: f32,
        cy: f32,
        radius: f32,
        player_yaw: f32,
        relative_target: Option<(f32, f32)>,
        is_pickup: bool,
    ) {
        // Dark glass background circle
        self.add_panel(
            cx - radius - 6.0,
            cy - radius - 6.0,
            (radius + 6.0) * 2.0,
            (radius + 6.0) * 2.0,
            [0.1, 0.6, 1.0, 1.0],
        );

        // Radial range rings (e.g. 50m and 100m)
        self.add_circle_outline(cx, cy, radius * 0.5, 1.0, 16, [0.25, 0.40, 0.55, 0.4]);
        self.add_circle_outline(cx, cy, radius * 0.95, 1.5, 24, [0.30, 0.50, 0.75, 0.7]);

        // Crosshairs
        self.add_line(
            cx - radius * 0.9,
            cy,
            cx + radius * 0.9,
            cy,
            1.0,
            [0.25, 0.35, 0.45, 0.3],
        );
        self.add_line(
            cx,
            cy - radius * 0.9,
            cx,
            cy + radius * 0.9,
            1.0,
            [0.25, 0.35, 0.45, 0.3],
        );

        // North arrow on radar ring
        let north_angle = -player_yaw - std::f32::consts::FRAC_PI_2;
        let nx = cx + north_angle.cos() * (radius * 0.82);
        let ny = cy + north_angle.sin() * (radius * 0.82);
        self.add_text(nx - 3.0, ny - 4.0, 1.2, "N", [0.95, 0.3, 0.3, 1.0]);

        // Player vehicle chevron (Center, always pointing UP)
        self.add_line(cx, cy - 8.0, cx - 6.0, cy + 6.0, 2.0, [0.2, 0.95, 0.4, 1.0]);
        self.add_line(cx, cy - 8.0, cx + 6.0, cy + 6.0, 2.0, [0.2, 0.95, 0.4, 1.0]);
        self.add_line(cx - 6.0, cy + 6.0, cx, cy + 3.0, 1.5, [0.2, 0.95, 0.4, 1.0]);
        self.add_line(cx + 6.0, cy + 6.0, cx, cy + 3.0, 1.5, [0.2, 0.95, 0.4, 1.0]);

        // Target waypoint beacon pin (Pickup or Dropoff)
        if let Some((dx, dz)) = relative_target {
            let scale_m_to_px = (radius * 0.85) / 250.0;
            let pin_x_rel = dx * scale_m_to_px;
            let pin_y_rel = -dz * scale_m_to_px;
            let dist = (pin_x_rel * pin_x_rel + pin_y_rel * pin_y_rel).sqrt();
            let (clamped_x, clamped_y) = if dist > radius * 0.85 {
                let factor = (radius * 0.85) / dist;
                (pin_x_rel * factor, pin_y_rel * factor)
            } else {
                (pin_x_rel, pin_y_rel)
            };

            let pin_cx = cx + clamped_x;
            let pin_cy = cy + clamped_y;
            let pin_col = if is_pickup {
                [0.1, 0.95, 0.45, 1.0]
            } else {
                [1.0, 0.75, 0.15, 1.0]
            };
            self.add_rect(pin_cx - 4.0, pin_cy - 4.0, 8.0, 8.0, pin_col);
            self.add_circle_outline(pin_cx, pin_cy, 7.0, 1.0, 8, pin_col);
        }
    }

    /// Adds an active contract mission card to the top right of the HUD.
    #[allow(clippy::too_many_arguments)]
    pub fn add_mission_hud_card(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        career_title: &str,
        mission_title: &str,
        objective_step: &str,
        distance_m: f64,
        comfort_stars: f32,
        cargo_integrity: f32,
        payout_credits: f64,
    ) {
        self.add_panel(x, y, w, h, [0.25, 0.85, 0.50, 1.0]);

        // Career title & potential payout
        self.add_text(
            x + 14.0,
            y + 12.0,
            1.1,
            career_title,
            [0.35, 0.95, 0.65, 1.0],
        );
        let pay_str = format!("REWARD: Rs.{:.0}", payout_credits);
        let (pay_w, _) = self.measure_text_proportional(&pay_str, 1.1 * 13.5);
        self.add_text(
            x + w - pay_w - 14.0,
            y + 12.0,
            1.1,
            &pay_str,
            [1.0, 0.85, 0.2, 1.0],
        );

        // Mission title
        self.add_text(x + 14.0, y + 28.0, 1.4, mission_title, [1.0, 1.0, 1.0, 1.0]);

        // Objective step description
        self.add_text(
            x + 14.0,
            y + 48.0,
            1.15,
            objective_step,
            [0.95, 0.95, 0.50, 1.0],
        );

        // Distance to active waypoint
        let dist_str = if distance_m >= 1000.0 {
            format!("Distance: {:.1} km", distance_m / 1000.0)
        } else {
            format!("Distance: {:.0} m", distance_m)
        };
        self.add_text(x + 14.0, y + 66.0, 1.0, &dist_str, [0.75, 0.85, 1.0, 0.9]);

        // Quality / Health / Star rating
        let rating_str = format!(
            "Rating: {:.1} Stars | Cargo: {:.0}%",
            comfort_stars,
            cargo_integrity * 100.0
        );
        self.add_text(x + 14.0, y + 80.0, 0.95, &rating_str, [0.7, 0.9, 0.8, 0.85]);
    }

    /// Adds a compact developer debug telemetry overlay (toggled via F3).
    #[allow(clippy::too_many_arguments)]
    pub fn add_debug_overlay(
        &mut self,
        x: f32,
        y: f32,
        fps: f64,
        one_pct_low: f64,
        avg_ms: f64,
        preset: &str,
        coord: &str,
        active_chunks: usize,
        culled_chunks: usize,
        triangles: usize,
        rss_mb: f64,
        vram_mb: f64,
    ) {
        let w = 480.0;
        let h = 95.0;
        self.add_rect(x, y, w, h, [0.03, 0.04, 0.06, 0.80]);
        self.add_rect_outline(x, y, w, h, 1.0, [0.25, 0.40, 0.60, 0.4]);

        self.add_text(
            x + 10.0,
            y + 10.0,
            1.05,
            &format!(
                "DEV DEBUG [F3 to hide] | FPS: {:>4.1} (1% Low: {:>4.1}) | {:.2}ms | Preset: {}",
                fps, one_pct_low, avg_ms, preset
            ),
            [0.2, 0.95, 0.45, 1.0],
        );
        self.add_text(x + 10.0, y + 28.0, 0.95, coord, [0.75, 0.85, 0.95, 1.0]);
        self.add_text(
            x + 10.0,
            y + 46.0,
            0.95,
            &format!(
                "LOD Chunks: {} active ({} culled) | Triangles: {}",
                active_chunks, culled_chunks, triangles
            ),
            [0.85, 0.75, 0.40, 1.0],
        );
        self.add_text(
            x + 10.0,
            y + 64.0,
            0.95,
            &format!(
                "Memory: Working Set RSS {:.1} MB | GPU VRAM {:.1} MB",
                rss_mb, vram_mb
            ),
            [0.95, 0.65, 0.30, 1.0],
        );
    }

    /// Adds a dedicated motorcycle instrument cluster with dynamic lean angle gauge.
    #[allow(clippy::too_many_arguments)]
    pub fn add_motorcycle_cluster(
        &mut self,
        x: f32,
        y: f32,
        speed_kph: f32,
        lean_angle_deg: f32,
        steer_deg: f32,
        rpm: f32,
        is_crashed: bool,
    ) {
        let w = 400.0;
        let h = 125.0;
        self.add_panel(x, y, w, h, [0.0, 0.85, 0.95, 1.0]);

        // Speedometer
        self.add_text(
            x + 16.0,
            y + 14.0,
            1.35,
            &format!("SPEED: {:>5.1} KM/H", speed_kph),
            [1.0, 0.95, 0.2, 1.0],
        );

        // Lean Angle Indicator
        let lean_dir = if lean_angle_deg >= 0.0 { "R" } else { "L" };
        let lean_abs = lean_angle_deg.abs();
        let (lean_col, status_text) = if is_crashed {
            ([1.0, 0.15, 0.15, 1.0], "CRASHED - RESET ROAD")
        } else if lean_abs > 42.0 {
            ([1.0, 0.35, 0.15, 1.0], "FOOTPEG SCRAPE WARNING")
        } else {
            ([0.2, 0.95, 0.45, 1.0], "STABLE LEAN")
        };

        self.add_text(
            x + 16.0,
            y + 36.0,
            1.1,
            &format!(
                "LEAN: {:>4.1}° {} | STEER: {:>+3.0}°",
                lean_abs, lean_dir, steer_deg
            ),
            lean_col,
        );

        // Visual Lean Angle Meter Bar (-48° to +48°)
        let meter_y = y + 54.0;
        let meter_w = 360.0;
        self.add_rect(x + 16.0, meter_y, meter_w, 8.0, [0.12, 0.15, 0.20, 1.0]);
        // Center mark
        self.add_rect(
            x + 16.0 + meter_w * 0.5 - 1.0,
            meter_y - 2.0,
            2.0,
            12.0,
            [0.8, 0.8, 0.8, 0.8],
        );
        // Current lean needle
        let norm_lean = (lean_angle_deg / 48.0).clamp(-1.0, 1.0);
        let needle_x = x + 16.0 + (meter_w * 0.5) + (norm_lean * meter_w * 0.5);
        self.add_rect(needle_x - 3.0, meter_y - 3.0, 6.0, 14.0, lean_col);

        // Tachometer RPM
        self.add_text(
            x + 16.0,
            y + 72.0,
            0.95,
            &format!("ENGINE: {:.0} RPM | STATUS: {}", rpm, status_text),
            [0.85, 0.88, 0.92, 0.9],
        );
        self.add_bar(
            x + 16.0,
            y + 86.0,
            meter_w,
            6.0,
            (rpm / 11500.0).clamp(0.0, 1.0),
            [0.1, 0.85, 0.7, 1.0],
            [0.15, 0.18, 0.22, 1.0],
        );

        self.add_text(
            x + 16.0,
            y + 102.0,
            0.85,
            "Controls: [W/S] Throttle/Brake | [A/D] Lean Steer | [M] Cycle Mode",
            [0.65, 0.75, 0.85, 0.8],
        );
    }

    /// Adds a dedicated heavy truck instrument cluster with trailer articulation and rollover safety.
    #[allow(clippy::too_many_arguments)]
    pub fn add_heavy_truck_cluster(
        &mut self,
        x: f32,
        y: f32,
        speed_kph: f32,
        gear_str: &str,
        trailer_deg: f32,
        rollover_margin_g: f32,
        air_brake_bar: f32,
        is_rolled_over: bool,
    ) {
        let w = 420.0;
        let h = 130.0;
        self.add_panel(x, y, w, h, [0.95, 0.45, 0.10, 1.0]);

        self.add_text(
            x + 16.0,
            y + 14.0,
            1.35,
            &format!("HEAVY HAUL: {:>5.1} KM/H | GEAR: {}", speed_kph, gear_str),
            [1.0, 0.85, 0.2, 1.0],
        );

        // Trailer Articulation
        let jackknife_warn = trailer_deg.abs() > 45.0;
        let trailer_col = if jackknife_warn {
            [1.0, 0.2, 0.2, 1.0]
        } else {
            [0.75, 0.85, 1.0, 1.0]
        };
        self.add_text(
            x + 16.0,
            y + 36.0,
            1.05,
            &format!("TRAILER ANGLE: {:>+4.1}° | KINGPIN HITCH", trailer_deg),
            trailer_col,
        );

        // Rollover Safety Gauge
        let (roll_col, roll_txt) = if is_rolled_over {
            ([1.0, 0.1, 0.1, 1.0], "ROLLED OVER! [R to recover]")
        } else if rollover_margin_g > 0.40 {
            ([1.0, 0.4, 0.1, 1.0], "HIGH ROLLOVER RISK")
        } else {
            ([0.2, 0.95, 0.45, 1.0], "STABILITY MARGIN NORMAL")
        };
        self.add_text(
            x + 16.0,
            y + 54.0,
            1.0,
            &format!(
                "LATERAL G: {:.2}G / 0.58G | {}",
                rollover_margin_g, roll_txt
            ),
            roll_col,
        );
        self.add_bar(
            x + 16.0,
            y + 68.0,
            380.0,
            8.0,
            (rollover_margin_g / 0.58).clamp(0.0, 1.0),
            roll_col,
            [0.15, 0.18, 0.22, 1.0],
        );

        // Pneumatic Air Brakes
        self.add_text(
            x + 16.0,
            y + 84.0,
            0.95,
            &format!(
                "PNEUMATIC BRAKE PRESSURE: {:.1} BAR (OPERATING)",
                air_brake_bar
            ),
            [0.85, 0.88, 0.92, 0.9],
        );
        self.add_text(
            x + 16.0,
            y + 104.0,
            0.85,
            "Controls: [W/S] Power/S-Cam Air Brakes | [A/D] Steer | [R] Upright",
            [0.65, 0.75, 0.85, 0.8],
        );
    }

    /// Adds a dedicated public transit bus instrument cluster with passenger count and ride comfort.
    #[allow(clippy::too_many_arguments)]
    pub fn add_transit_bus_cluster(
        &mut self,
        x: f32,
        y: f32,
        speed_kph: f32,
        passenger_count: u16,
        total_mass_tons: f32,
        comfort_rating: f32,
        air_brake_bar: f32,
    ) {
        let w = 420.0;
        let h = 130.0;
        self.add_panel(x, y, w, h, [0.10, 0.75, 0.40, 1.0]);

        self.add_text(
            x + 16.0,
            y + 14.0,
            1.35,
            &format!("TRANSIT BUS: {:>5.1} KM/H", speed_kph),
            [0.35, 0.95, 0.65, 1.0],
        );

        // Passenger Payload & Swept Path
        self.add_text(
            x + 16.0,
            y + 36.0,
            1.05,
            &format!(
                "ONBOARD RIDERS: {} / 65 | GROSS MASS: {:.1} T",
                passenger_count, total_mass_tons
            ),
            [1.0, 1.0, 1.0, 1.0],
        );

        // Passenger Comfort Rating
        self.add_text(
            x + 16.0,
            y + 54.0,
            1.0,
            &format!("PASSENGER COMFORT: {:.1} / 5.0 STARS", comfort_rating),
            [1.0, 0.85, 0.2, 1.0],
        );
        self.add_bar(
            x + 16.0,
            y + 68.0,
            380.0,
            8.0,
            (comfort_rating / 5.0).clamp(0.0, 1.0),
            [0.2, 0.9, 0.4, 1.0],
            [0.15, 0.18, 0.22, 1.0],
        );

        // Air Brake Chamber
        self.add_text(
            x + 16.0,
            y + 84.0,
            0.95,
            &format!(
                "AIR BRAKE SYSTEM: {:.1} BAR | AIR SUSPENSION OK",
                air_brake_bar
            ),
            [0.85, 0.88, 0.92, 0.9],
        );
        self.add_text(
            x + 16.0,
            y + 104.0,
            0.85,
            "Controls: [W/S] Throttle/Air Brakes | [A/D] Steer Wide (Rear Overhang)",
            [0.65, 0.75, 0.85, 0.8],
        );
    }

    /// Adds a dedicated railway locomotive cab console.
    #[allow(clippy::too_many_arguments)]
    pub fn add_train_cluster(
        &mut self,
        x: f32,
        y: f32,
        speed_kph: f32,
        throttle_pct: f32,
        brake_pipe_bar: f32,
        brake_cyl_bar: f32,
        tractive_kn: f32,
        reverser: i8,
        signal_aspect: &str,
        wheel_slip: bool,
    ) {
        let w = 440.0;
        let h = 140.0;
        self.add_panel(x, y, w, h, [0.85, 0.65, 0.15, 1.0]);

        let rev_str = match reverser {
            1 => "FORWARD",
            -1 => "REVERSE",
            _ => "NEUTRAL",
        };
        self.add_text(
            x + 16.0,
            y + 14.0,
            1.35,
            &format!("LOCOMOTIVE: {:>5.1} KM/H | {}", speed_kph, rev_str),
            [1.0, 0.95, 0.25, 1.0],
        );

        // Throttle Notch & Tractive Effort
        let slip_str = if wheel_slip { " [WHEEL SLIP!]" } else { "" };
        self.add_text(
            x + 16.0,
            y + 36.0,
            1.05,
            &format!(
                "NOTCH: {:.0}% | TRACTIVE EFFORT: {:.0} kN{}",
                throttle_pct * 100.0,
                tractive_kn,
                slip_str
            ),
            if wheel_slip {
                [1.0, 0.2, 0.2, 1.0]
            } else {
                [0.3, 0.9, 0.6, 1.0]
            },
        );
        self.add_bar(
            x + 16.0,
            y + 50.0,
            400.0,
            8.0,
            throttle_pct.clamp(0.0, 1.0),
            [0.2, 0.85, 0.5, 1.0],
            [0.15, 0.18, 0.22, 1.0],
        );

        // Westinghouse Air Brakes
        self.add_text(
            x + 16.0,
            y + 66.0,
            1.0,
            &format!(
                "BRAKE PIPE: {:.2} BAR | CYLINDER: {:.2} BAR",
                brake_pipe_bar, brake_cyl_bar
            ),
            [0.85, 0.85, 0.9, 1.0],
        );

        // Signal Aspect
        let sig_col = match signal_aspect {
            "Clear" => [0.2, 0.95, 0.4, 1.0],
            "Approach" => [1.0, 0.85, 0.2, 1.0],
            _ => [1.0, 0.15, 0.15, 1.0],
        };
        self.add_text(
            x + 16.0,
            y + 84.0,
            1.05,
            &format!("TRACK SIGNAL: {}", signal_aspect.to_uppercase()),
            sig_col,
        );

        self.add_text(
            x + 16.0,
            y + 106.0,
            0.85,
            "Controls: [W/S] Throttle Notch / Air Brakes | [Space] Emergency Vent | [T] Routes",
            [0.65, 0.75, 0.85, 0.8],
        );
    }

    /// Adds an authentic railway station and route dispatch selector modal.
    #[allow(clippy::too_many_arguments)]
    pub fn add_rail_route_modal(
        &mut self,
        screen_w: f32,
        screen_h: f32,
        selected_route_idx: usize,
        line_name: &str,
        routes: &[(&str, &str, &str, &str, f32, &str)],
        hovered_idx: Option<usize>,
    ) {
        let modal_w = (screen_w * 0.72).clamp(650.0, 920.0);
        let modal_h = (screen_h * 0.70).clamp(480.0, 680.0);
        let x = (screen_w - modal_w) * 0.5;
        let y = (screen_h - modal_h) * 0.5;

        // Semi-transparent backdrop and modal panel
        self.add_rect(0.0, 0.0, screen_w, screen_h, [0.02, 0.04, 0.08, 0.78]);
        self.add_panel(x, y, modal_w, modal_h, [0.2, 0.85, 1.0, 1.0]);

        // Header
        self.add_text(
            x + 24.0,
            y + 20.0,
            1.8,
            "TRAIN ROUTE & DISPATCH SELECTOR",
            [0.2, 0.85, 1.0, 1.0],
        );
        self.add_text(
            x + 24.0,
            y + 52.0,
            1.1,
            &format!("CORRIDOR: {} | AUTHENTIC TRACK CENTERLINES", line_name),
            [0.7, 0.8, 0.9, 1.0],
        );

        // List routes
        let list_y = y + 85.0;
        let row_h = 75.0;
        for (i, (r_name, dep, dest, plat, len_km, tracks)) in routes.iter().enumerate() {
            let ry = list_y + (i as f32) * (row_h + 12.0);
            let is_selected = i == selected_route_idx;
            let is_hovered = hovered_idx == Some(i);

            let bg_color = if is_selected {
                [0.15, 0.35, 0.65, 0.9]
            } else if is_hovered {
                [0.12, 0.22, 0.38, 0.8]
            } else {
                [0.06, 0.09, 0.16, 0.8]
            };

            self.add_rect(x + 24.0, ry, modal_w - 48.0, row_h, bg_color);
            self.add_rect_outline(
                x + 24.0,
                ry,
                modal_w - 48.0,
                row_h,
                1.0,
                if is_selected {
                    [0.4, 0.75, 1.0, 0.8]
                } else {
                    [0.2, 0.3, 0.45, 0.4]
                },
            );

            let title_color = if is_selected {
                [1.0, 0.9, 0.3, 1.0]
            } else {
                [0.9, 0.95, 1.0, 1.0]
            };
            self.add_text(x + 36.0, ry + 10.0, 1.3, r_name, title_color);

            let info_str = format!(
                "From: {}  ->  To: {}  |  Platform: {}  |  Length: {:.1} km",
                dep, dest, plat, len_km
            );
            self.add_text(x + 36.0, ry + 32.0, 1.0, &info_str, [0.75, 0.85, 0.95, 1.0]);

            let track_str = format!("Track Segments: {}", tracks);
            self.add_text(
                x + 36.0,
                ry + 50.0,
                0.9,
                &track_str,
                [0.55, 0.70, 0.85, 1.0],
            );
        }

        // Footer instructions
        let foot_y = y + modal_h - 45.0;
        self.add_text(
            x + 24.0,
            foot_y,
            1.05,
            "[UP/DOWN / CLICK] Select Route  |  [ENTER / SPACE] Dispatch Consist  |  [ESC/T] Close",
            [0.6, 0.75, 0.9, 1.0],
        );
    }

    /// Adds a dedicated marine bridge & helm navigation console.
    #[allow(clippy::too_many_arguments)]
    pub fn add_marine_cluster(
        &mut self,
        x: f32,
        y: f32,
        speed_knots: f32,
        throttle: f32,
        rudder_deg: f32,
        heading_deg: f32,
        draft_m: f32,
        gm_m: f32,
    ) {
        let w = 420.0;
        let h = 130.0;
        self.add_panel(x, y, w, h, [0.15, 0.55, 0.85, 1.0]);

        self.add_text(
            x + 16.0,
            y + 14.0,
            1.35,
            &format!("VESSEL SPEED: {:>5.1} KNOTS", speed_knots),
            [0.2, 0.9, 1.0, 1.0],
        );

        // Engine Order Telegraph
        let teleg_str = if throttle > 0.75 {
            "AHEAD FULL"
        } else if throttle > 0.35 {
            "AHEAD HALF"
        } else if throttle > 0.05 {
            "AHEAD SLOW"
        } else if throttle < -0.05 {
            "ASTERN ENGINE"
        } else {
            "STOP ENGINES"
        };
        self.add_text(
            x + 16.0,
            y + 36.0,
            1.05,
            &format!(
                "TELEGRAPH: {} | HEADING: {:03.0}° T",
                teleg_str, heading_deg
            ),
            [1.0, 1.0, 1.0, 1.0],
        );

        // Rudder Angle Indicator
        let rud_dir = if rudder_deg >= 0.0 { "STBD" } else { "PORT" };
        self.add_text(
            x + 16.0,
            y + 54.0,
            1.0,
            &format!(
                "RUDDER: {:>3.0}° {} | HYDRO DRAFT: {:.1}M",
                rudder_deg.abs(),
                rud_dir,
                draft_m
            ),
            [0.4, 0.95, 0.75, 1.0],
        );
        // Rudder bar (-35 to +35)
        let rud_norm = (rudder_deg / 35.0).clamp(-1.0, 1.0);
        let rud_bar_x = x + 16.0 + 190.0 + (rud_norm * 180.0);
        self.add_rect(x + 16.0, y + 68.0, 380.0, 6.0, [0.12, 0.18, 0.25, 1.0]);
        self.add_rect(rud_bar_x - 3.0, y + 66.0, 6.0, 10.0, [0.2, 0.85, 1.0, 1.0]);

        // Metacentric Height
        self.add_text(
            x + 16.0,
            y + 84.0,
            0.95,
            &format!(
                "METACENTRIC HEIGHT GM: {:.2}M [POSITIVE RIGHTING STABILITY]",
                gm_m
            ),
            [0.85, 0.88, 0.92, 0.9],
        );
        self.add_text(
            x + 16.0,
            y + 104.0,
            0.85,
            "Controls: [W/S] Telegraph Propulsion | [A/D] Rudder Helm | [M] Cycle",
            [0.65, 0.75, 0.85, 0.8],
        );
    }

    /// Adds a Primary Flight Display (PFD) aviation cockpit instrument HUD.
    #[allow(clippy::too_many_arguments)]
    pub fn add_aviation_pfd(
        &mut self,
        x: f32,
        y: f32,
        ias_knots: f32,
        alt_m: f32,
        climb_rate_mps: f32,
        pitch_deg: f32,
        roll_deg: f32,
        flaps: f32,
        gear_down: bool,
        is_stalled: bool,
    ) {
        let w = 340.0;
        let h = 220.0;
        self.add_rect(x, y, w, h, [0.03, 0.04, 0.06, 0.85]);
        self.add_rect_outline(x, y, w, h, 1.5, [0.25, 0.65, 0.95, 0.8]);

        // Center Artificial Horizon Window (Width 160, Height 140)
        let horizon_x = x + 90.0;
        let horizon_y = y + 20.0;
        let horizon_w = 160.0;
        let horizon_h = 130.0;
        let horizon_cy = horizon_y + horizon_h * 0.5;

        // Sky (Blue top half) and Ground (Brown bottom half)
        let pitch_px = (pitch_deg * 2.0).clamp(-horizon_h * 0.45, horizon_h * 0.45);
        let divide_y = horizon_cy - pitch_px;
        let sky_h = (divide_y - horizon_y).clamp(0.0, horizon_h);
        self.add_rect(
            horizon_x,
            horizon_y,
            horizon_w,
            sky_h,
            [0.15, 0.42, 0.82, 0.95],
        ); // Sky blue
        self.add_rect(
            horizon_x,
            horizon_y + sky_h,
            horizon_w,
            horizon_h - sky_h,
            [0.42, 0.28, 0.15, 0.95],
        ); // Earth brown
        self.add_rect_outline(
            horizon_x,
            horizon_y,
            horizon_w,
            horizon_h,
            1.0,
            [0.7, 0.8, 0.9, 0.6],
        );

        // Artificial Horizon Crosshairs / Airplane Symbol
        self.add_line(
            horizon_x + 30.0,
            horizon_cy,
            horizon_x + 65.0,
            horizon_cy,
            2.5,
            [1.0, 0.9, 0.1, 1.0],
        );
        self.add_line(
            horizon_x + horizon_w - 65.0,
            horizon_cy,
            horizon_x + horizon_w - 30.0,
            horizon_cy,
            2.5,
            [1.0, 0.9, 0.1, 1.0],
        );
        self.add_rect(
            horizon_x + horizon_w * 0.5 - 2.0,
            horizon_cy - 2.0,
            4.0,
            4.0,
            [1.0, 0.9, 0.1, 1.0],
        );

        // Left Airspeed Tape
        self.add_rect(x + 10.0, y + 20.0, 70.0, horizon_h, [0.08, 0.10, 0.14, 0.9]);
        self.add_rect_outline(
            x + 10.0,
            y + 20.0,
            70.0,
            horizon_h,
            1.0,
            [0.3, 0.4, 0.5, 0.6],
        );
        self.add_text(x + 16.0, y + 26.0, 0.95, "AIRSPEED", [0.7, 0.8, 0.9, 0.9]);
        self.add_text(
            x + 14.0,
            horizon_cy - 8.0,
            1.35,
            &format!("{:>3.0}", ias_knots),
            [0.2, 0.95, 0.4, 1.0],
        );
        self.add_text(
            x + 24.0,
            horizon_cy + 10.0,
            0.9,
            "KTS",
            [0.7, 0.8, 0.9, 0.8],
        );

        // Right Altitude Tape
        self.add_rect(
            x + w - 80.0,
            y + 20.0,
            70.0,
            horizon_h,
            [0.08, 0.10, 0.14, 0.9],
        );
        self.add_rect_outline(
            x + w - 80.0,
            y + 20.0,
            70.0,
            horizon_h,
            1.0,
            [0.3, 0.4, 0.5, 0.6],
        );
        self.add_text(
            x + w - 74.0,
            y + 26.0,
            0.95,
            "ALT MSL",
            [0.7, 0.8, 0.9, 0.9],
        );
        self.add_text(
            x + w - 76.0,
            horizon_cy - 8.0,
            1.30,
            &format!("{:>4.0}", alt_m),
            [0.3, 0.9, 1.0, 1.0],
        );
        self.add_text(
            x + w - 66.0,
            horizon_cy + 10.0,
            0.9,
            "METERS",
            [0.7, 0.8, 0.9, 0.8],
        );

        // Vertical Speed Readout
        let vsi_dir = if climb_rate_mps >= 0.0 { "+" } else { "" };
        self.add_text(
            x + w - 76.0,
            y + horizon_h - 10.0,
            0.85,
            &format!("VSI {}{:.1}", vsi_dir, climb_rate_mps),
            [0.85, 0.88, 0.92, 0.8],
        );

        // Stall Warning Banner (Flashing red when in stall condition)
        if is_stalled {
            self.add_rect(x + 30.0, y + 60.0, w - 60.0, 32.0, [0.95, 0.05, 0.05, 0.95]);
            self.add_text(
                x + 52.0,
                y + 68.0,
                1.3,
                "STALL! PITCH DOWN!",
                [1.0, 1.0, 1.0, 1.0],
            );
        }

        // Lower Status Strip: Flaps, Gear, Pitch/Roll degrees
        let gear_str = if gear_down {
            "DOWN [LOCK]"
        } else {
            "UP [RETRACT]"
        };
        let gear_col = if gear_down {
            [0.2, 0.95, 0.4, 1.0]
        } else {
            [0.7, 0.7, 0.7, 0.8]
        };
        self.add_text(
            x + 16.0,
            y + 160.0,
            0.95,
            &format!("FLAPS: {:>2.0}% | GEAR: {}", flaps * 100.0, gear_str),
            gear_col,
        );
        self.add_text(
            x + 16.0,
            y + 176.0,
            0.90,
            &format!(
                "PITCH: {:>+4.1}° | ROLL: {:>+4.1}° | 6-DOF AERODYNAMICS",
                pitch_deg, roll_deg
            ),
            [0.8, 0.85, 0.9, 0.85],
        );
        self.add_text(
            x + 16.0,
            y + 196.0,
            0.85,
            "Controls: [W/S] Pitch | [A/D] Roll | [Space] Thrust | [M] Cycle",
            [0.65, 0.75, 0.85, 0.8],
        );
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
