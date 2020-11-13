use tui::layout::{Rect};

pub fn centered_area(base: Rect, dimension: (u16, u16)) -> Rect {
    let width_diff = base.width as i16 - dimension.0 as i16;
    let height_diff = base.height as i16 - dimension.1 as i16;
    let x = if width_diff > 0 { width_diff / 2 } else { 0 };
    let y = if height_diff > 0 { height_diff / 2 } else { 0 };
    let width = if base.width > dimension.0 { dimension.0 } else { base.width };
    let height = if base.height > dimension.1 { dimension.1 } else { base.height };
    Rect::new(x as u16, y as u16, width, height)
}
