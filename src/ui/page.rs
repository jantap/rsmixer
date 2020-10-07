use super::{
    common::*,
    widgets::BlockWidget,
    draw_entries,
};

pub struct UIPage {
    pub inner_area: Rect,
}

pub async fn draw_page<W: Write>(
    stdout: &mut W,
    state: &mut RSState,
) -> Result<(), RSError> {
    let (w, h) = crossterm::terminal::size()?;

    let mut b = BlockWidget::default()
        .clean_inside(true)
        .title(state.current_page.as_styled_string());
    b.render(Rect::new(0, 0, w, h), stdout)?;

    draw_entries(
        stdout,
        state,
        state.ui_page.inner_area,
        None,
    )
    .await?;

    stdout.flush()?;

    Ok(())
}

