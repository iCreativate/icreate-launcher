//! Controls for the toolbar row: the grid/table switch drawn as two icons, and the refresh button.
//!
//! Invariants: every colour comes from [`Tokens`]; the icons are drawn with the painter, so no
//! font or image has to carry them; nothing here owns state, the caller passes the value in.

use egui::{CornerRadius, Rect, Sense, Ui, vec2};

use crate::settings::ViewMode;
use crate::theme::{self, Tokens};

/// Grid and table as two rounded icons in one hairline box; the chosen one is filled. In a
/// right-to-left parent they still read grid, then table.
pub fn view_toggle(ui: &mut Ui, view: &mut ViewMode, tips: [&str; 2]) {
    let t = Tokens::get(ui.ctx());
    let rtl = ui.layout().prefer_right_to_left();
    let mut modes = [(ViewMode::Grid, tips[0]), (ViewMode::Table, tips[1])];
    if rtl {
        modes.reverse();
    }
    let mut rects = Vec::with_capacity(2);
    ui.scope(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        for (mode, tip) in modes {
            let on = *view == mode;
            let (rect, resp) = ui.allocate_exact_size(vec2(theme::CONTROL_H, theme::CONTROL_H), Sense::click());
            let (fill, ink) = match (on, resp.hovered()) {
                (true, _) => (t.invert_bg, t.invert_fg),
                (false, true) => (t.sunken, t.ink_strong),
                (false, false) => (egui::Color32::TRANSPARENT, t.muted),
            };
            let p = ui.painter();
            let rounding = if rtl {
                if rects.is_empty() { CornerRadius { nw: 12, ne: 0, sw: 12, se: 0 } } else { CornerRadius { nw: 0, ne: 12, sw: 0, se: 12 } }
            } else if rects.is_empty() {
                CornerRadius { nw: 12, ne: 0, sw: 12, se: 0 }
            } else {
                CornerRadius { nw: 0, ne: 12, sw: 0, se: 12 }
            };
            p.rect_filled(rect, rounding, fill);
            let c = rect.center();
            match mode {
                ViewMode::Grid => {
                    for (dx, dy) in [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
                        p.rect_filled(Rect::from_center_size(c + vec2(dx * 3.75, dy * 3.75), vec2(6.0, 6.0)), theme::RADIUS_SM, ink);
                    }
                }
                ViewMode::Table => {
                    for dy in [-5.0, 0.0, 5.0] {
                        p.rect_filled(Rect::from_center_size(c + vec2(0.0, dy), vec2(15.0, 2.5)), theme::RADIUS_SM, ink);
                    }
                }
            }
            resp.widget_info(|| egui::WidgetInfo::selected(egui::Role::Button, true, on, tip));
            if resp.on_hover_text(tip).on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                *view = mode;
            }
            rects.push(rect);
        }
    });
    if let (Some(first), Some(last)) = (rects.first(), rects.last()) {
        let p = ui.painter();
        p.rect_stroke(first.union(*last), theme::RADIUS_MD, t.hairline(), egui::StrokeKind::Inside);
        for r in rects.iter().skip(1) {
            let x = if rtl { r.right() } else { r.left() };
            p.vline(x, first.top()..=first.bottom(), t.hairline());
        }
    }
}

/// The refresh control: a rounded control in the same family as the grid/table switch. While `busy`
/// it shows a spinner and ignores clicks.
pub fn refresh_button(ui: &mut Ui, tip: &str, busy: bool) -> egui::Response {
    let t = Tokens::get(ui.ctx());
    let (rect, resp) = ui.allocate_exact_size(vec2(theme::CONTROL_H, theme::CONTROL_H), if busy { Sense::hover() } else { Sense::click() });
    let hovered = resp.hovered() && !busy;
    let p = ui.painter();
    if hovered {
        p.rect_filled(rect, theme::RADIUS_MD, t.sunken);
    }
    p.rect_stroke(rect, theme::RADIUS_MD, t.hairline(), egui::StrokeKind::Inside);
    if busy {
        egui::Spinner::new().size(16.0).color(t.muted).paint_at(ui, Rect::from_center_size(rect.center(), vec2(16.0, 16.0)));
    } else {
        crate::widgets::paint_glyph(p, rect, crate::widgets::Glyph::Refresh, if hovered { t.ink_strong } else { t.muted });
    }
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, !busy, tip));
    if busy { resp } else { resp.on_hover_cursor(egui::CursorIcon::PointingHand) }
}
