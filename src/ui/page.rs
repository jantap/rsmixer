use super::{
    common::*,
    widgets::BlockWidget,
    draw_entries,
};

pub async fn draw_page<W: Write>(
    stdout: &mut W,
    state: &mut RSState,
) -> Result<(), RSError> {
    let (w, h) = crossterm::terminal::size()?;

    let mut b = BlockWidget::default()
        .clean_inside(true)
        .title(state.current_page.as_str().to_string());
    b.render(Rect::new(0, 0, w, h), stdout)?;

    draw_entries(
        stdout,
        state,
        None,
    )
    .await?;

    stdout.flush()?;

    Ok(())
}

