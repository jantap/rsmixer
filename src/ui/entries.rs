use super::common::*;

use std::collections::HashSet;

pub async fn draw_entries<W: Write>(
    stdout: &mut W,
    state: &mut RSState,
    affected: Option<HashSet<usize>>,
) -> Result<(), RSError> {
    let (w, h) = crossterm::terminal::size()?;
    let mut entry_size = Rect::new(2, 2, w - 4, 3);

    let mut bg = entry_size;
    bg.height = h - *Y_PADDING;

    if affected.is_none() {
        draw_rect!(stdout, " ", bg, get_style("normal"));
    }

    for (i, lvl) in state.page_entries.visible_range_with_lvl(state.scroll) {
        if let Some(aff) = affected.clone() {
            if aff.get(&i).is_none() {
                entry_size.y += entry_height(lvl);
                continue;
            }
        }

        let ent = match state.entries.get_mut(&state.page_entries.get(i).unwrap()) {
            Some(x) => x,
            None => {
                continue;
            }
        };
        ent.position = lvl;
        ent.is_selected = state.selected == i;

        ent.render(entry_size, stdout)?;
        entry_size.y += entry_height(lvl);
    }

    stdout.flush()?;

    Ok(())
}

