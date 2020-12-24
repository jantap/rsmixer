use super::{common::*, draw_entries, widgets::BlockWidget};

use crate::draw_at;

pub struct UIPage {
    pub inner_area: Rect,
}

pub async fn draw_page<W: Write>(stdout: &mut W, state: &mut RSState) -> Result<(), RSError> {
    let (w, h) = crossterm::terminal::size()?;

    let mut b = BlockWidget::default()
        .clean_inside(true)
        .title_len(22)
        .title(state.current_page.as_styled_string());
    b.render(Rect::new(0, 0, w, h), stdout)?;

    draw_entries(stdout, state, state.ui_page.inner_area, None).await?;

    stdout.flush()?;

    Ok(())
}

pub async fn draw_disconnected_page<W: Write>(stdout: &mut W, time: u64) -> Result<(), RSError> {
    let (w, h) = crossterm::terminal::size()?;

    draw_rect!(stdout, " ", Rect::new(0, 0, w, h), get_style("normal"));
    draw_at!(
        stdout,
        format!("PulseAudio disconnected. Retrying in {}s", time),
        0,
        0,
        get_style("normal")
    );

    stdout.flush()?;

    Ok(())
}
