use crate::draw_rect;
use crate::{
    entry::{Entries, EntryType, Entry},
    ui::{
        models::PageEntries,
        output::{RedrawType, UIState},
        util::{entry_height, get_style, PageType, Rect, Y_PADDING},
        widgets::{BlockWidget, ContextMenuWidget, VolumeWidget, Widget},
    },
};
use crossterm::execute;

use crate::RSError;

use std::io::Write;

pub async fn draw_entities<W: Write>(
    stdout: &mut W,
    entries: &mut Entries,
    page_entries: &PageEntries,
    _current_page: &PageType,
    selected: usize,
    scroll: usize,
) -> Result<(), RSError> {
    let (w, h) = crossterm::terminal::size()?;
    let mut entry_size = Rect::new(2, 2, w - 4, 3);

    let mut bg = entry_size.clone();
    bg.height = h - *Y_PADDING;

    draw_rect!(stdout, " ", bg, get_style("normal"));

    for (i, lvl) in page_entries.visible_range_with_lvl(scroll) {
        let ent = match entries.get_mut(&page_entries.get(i).unwrap()) {
            Some(x) => x,
            None => {
                continue;
            }
        };
        ent.position = lvl;
        ent.is_selected = selected == i;

        ent.render(entry_size, stdout)?;
        entry_size.y += entry_height(lvl);
    }

    stdout.flush()?;

    Ok(())
}

pub async fn draw_page<W: Write>(
    stdout: &mut W,
    entries: &mut Entries,
    page_entries: &PageEntries,
    current_page: &PageType,
    selected: usize,
    scroll: usize,
) -> Result<(), RSError> {
    let (w, h) = crossterm::terminal::size()?;

    log::error!("REDRAW PAGE {:?}", current_page);

    let mut b = BlockWidget::default()
        .clean_inside(true)
        .title(current_page.as_str().to_string());
    b.render(Rect::new(0, 0, w, h), stdout)?;

    draw_entities(
        stdout,
        entries,
        page_entries,
        current_page,
        selected,
        scroll,
    )
    .await?;

    stdout.flush()?;

    Ok(())
}

pub async fn redraw<W: Write>(stdout: &mut W, state: &mut UIState) -> Result<(), RSError> {
    let (w, h) = crossterm::terminal::size()?;
    if w < 20 || h < 5 {
        execute!(stdout, crossterm::cursor::MoveTo(0, 0))?;
        let x = get_style("normal").apply(format!(
            "terminal too small{}",
            (0..w * h - 18).map(|_| " ").collect::<String>()
        ));
        write!(stdout, "{}", x)?;
        stdout.flush()?;
        return Ok(());
    }
    match state.redraw {
        RedrawType::Full => {
            return draw_page(
                stdout,
                &mut state.entries,
                &state.page_entries,
                &state.current_page,
                state.selected,
                state.scroll,
            )
            .await;
        }
        RedrawType::PeakVolume(ident) => {
            if ident.entry_type == EntryType::Card {
                return Ok(());
            }
            if let Some(index) = state.page_entries.iter_entries().position(|p| *p == ident) {
                if let Some(mut area) = state.page_entries.is_entry_visible(index, state.scroll)? {
                    area.y += 2;
                    area.height = 1;
                    area.width -= 1;

                    let ent = match state.entries.get_mut(&ident) {
                        Some(x) => x,
                        None => {
                            return Ok(());
                        }
                    };

                    let area = Entry::calc_area(state.page_entries.lvls[index], area);
                    let play = ent.play_entry.as_mut().unwrap();

                    let vol = VolumeWidget::default().volume(play.peak);
                    return vol.mute(play.mute).render(area, stdout);
                }
            }
        }
        RedrawType::Entries => {
            return draw_entities(
                stdout,
                &mut state.entries,
                &state.page_entries,
                &state.current_page,
                state.selected,
                state.scroll,
            )
            .await;
        }
        RedrawType::ContextMenu => {
            let (w, h) = crossterm::terminal::size()?;
            let mut b = ContextMenuWidget::new(state.page_entries.get(state.selected).unwrap())
                .selected(state.selected_context)
                .options(state.context_options.clone());

            let a = Rect::new(2, 2, w - 4, h - 4);
            return b.render(a, stdout);
        }
        _ => {}
    };
    Ok(())
}
