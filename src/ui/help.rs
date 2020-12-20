use super::{common::*, widgets::BlockWidget};

use crate::help;

pub async fn draw_help<W: Write>(stdout: &mut W) -> Result<(), RSError> {
    let (w, h) = crossterm::terminal::size()?;

    let lines = help::generate();
    let (mut width, lines) = help::help_lines_to_strings(&lines, w - 4)?;

    width += 4;
    let height = std::cmp::min(lines.len() + 4, h as usize) as u16;
    let width = std::cmp::min(width, w);

    let mut block = BlockWidget::default()
        .clean_inside(true)
        .title("Help".to_string());
    let x = w / 2 - width / 2;
    let y = h / 2 - height / 2;
    block.render(Rect::new(x, y, width, height), stdout)?;

    for (i, l) in lines.iter().enumerate() {
        execute!(stdout, crossterm::cursor::MoveTo(x + 2, y + 2 + i as u16))?;

        write!(stdout, "{}", get_style("normal").apply(l))?;
    }
    stdout.flush()?;

    Ok(())
}
