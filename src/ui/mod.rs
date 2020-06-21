mod entries;
mod macros;
mod util;
mod visibility;
mod widgets;

use entries::Entries;

use crate::{comms, EntryIdentifier, EntryType, Letter, Result, DISPATCH, SENDERS};
use crate::{draw_rect, input};
use util::{get_style, Rect};
use visibility::{adjust_scroll, is_entry_visible, EntrySpaceLvl};
use widgets::*;

pub use entries::Entry;
use std::io::Write;
pub use util::PageType;

use async_std::sync::channel;
use pulse::volume;

use std::cmp::{max, min};
use std::io;

use async_std::prelude::*;

use async_std::task;
use crossterm::{
    cursor::Hide,
    execute,
};
use lazy_static::lazy_static;

lazy_static! {
    static ref ENTRY_HEIGHT: u16 = 3;
    static ref Y_PADDING: u16 = 4;
}

fn parent_child_types(page: PageType) -> (EntryType, EntryType) {
    if page == PageType::Output {
        (EntryType::Sink, EntryType::SinkInput)
    } else {
        (EntryType::Source, EntryType::SourceOutput)
    }
}

pub async fn start() -> Result<()> {
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
    crossterm::terminal::enable_raw_mode()?;

    let (sx, mut rx) = channel(comms::CHANNEL_CAPACITY);
    SENDERS.register(comms::UI_MESSAGE, sx).await;

    let w = task::spawn(input::start());

    let e = task::spawn(async move {
        #[derive(PartialEq)]
        enum RedrawType {
            Full,
            Entries,
            PeakVolume(EntryIdentifier),
            None,
        };

        let mut current_page = PageType::Output;
        let mut entries = Entries::new();
        let mut page_entries = Vec::new();
        let mut selected = 0;
        let mut scroll = 0;
        let mut redraw = RedrawType::None;
        execute!(stdout, Hide).unwrap();
        draw(
            &mut stdout,
            &entries,
            &page_entries,
            &current_page,
            selected,
            scroll,
        )
        .await
        .unwrap();

        while let Some(msg) = rx.next().await {
            match msg {
                Letter::ExitSignal => {
                    let mut stdout = io::stdout();
                    crossterm::execute!(
                        stdout,
                        crossterm::cursor::Show,
                        crossterm::terminal::LeaveAlternateScreen
                    )
                    .unwrap();
                    crossterm::terminal::disable_raw_mode().unwrap();
                    break;
                }
                Letter::Redraw => {
                    redraw = RedrawType::Full;
                }
                Letter::EntryRemoved(ident) => {
                    entries.remove(&ident);
                }
                Letter::EntryUpdate(ident) => {
                    let entry = match ident.into_entry() {
                        Ok(entry) => entry,
                        Err(_) => {
                            continue;
                        }
                    };
                    entries.insert(ident, entry);
                    if page_entries.iter().any(|&i| i == ident) {
                        redraw = RedrawType::Entries;
                    }
                }
                Letter::PeakVolumeUpdate(ident, peak) => {
                    if let Some(e) = entries.get_mut(&ident) {
                        e.peak = peak;
                    }
                    if page_entries.iter().any(|&i| i == ident) {
                        redraw = RedrawType::PeakVolume(ident);
                    }
                }
                Letter::MoveUp(how_much) => {
                    selected = max(selected as i32 - how_much as i32, 0) as usize;
                    redraw = RedrawType::Entries;
                }
                Letter::MoveDown(how_much) => {
                    selected = min(selected + how_much as usize, page_entries.len());
                    redraw = RedrawType::Entries;
                }
                Letter::ChangePage(page) => {
                    current_page = page;
                    redraw = RedrawType::Full;
                }
                Letter::RequestMute => {
                    if selected < page_entries.len() {
                        let mute = match entries.get(&page_entries[selected]) {
                            Some(e) => e.mute,
                            None => {
                                continue;
                            }
                        };
                        DISPATCH
                            .event(Letter::MuteEntry(page_entries[selected], !mute))
                            .await;
                    }
                }
                Letter::RequstChangeVolume(how_much) => {
                    if let Some(entry) = entries.get_mut(&page_entries[selected]) {
                        let mut vols = entry.volume.clone();
                        for v in vols.get_mut() {
                            // @TODO add config
                            // @TODO don't overflow
                            let amount =
                                (volume::VOLUME_NORM.0 as f32 * how_much as f32 / 100.0) as i64;
                            if (v.0 as i64) < volume::VOLUME_MUTED.0 as i64 - amount {
                                v.0 = volume::VOLUME_MUTED.0;
                            } else if (v.0 as i64)
                                > (volume::VOLUME_NORM.0 as f32 * 1.5) as i64 - amount
                            {
                                v.0 = (volume::VOLUME_NORM.0 as f32 * 1.5) as u32;
                            } else {
                                v.0 = (v.0 as i64 + amount) as u32;
                            }
                        }
                        DISPATCH
                            .event(Letter::SetVolume(page_entries[selected], vols))
                            .await;
                    }
                }
                _ => {}
            };

            page_entries = current_page
                .generate_page(&entries)
                .map(|x| *x.0)
                .collect::<Vec<EntryIdentifier>>();

            if adjust_scroll(&page_entries, &mut scroll, &mut selected)
                && redraw != RedrawType::Full
            {
                redraw = RedrawType::Entries;
            }

            if redraw == RedrawType::Full {
                draw(
                    &mut stdout,
                    &entries,
                    &page_entries,
                    &current_page,
                    selected,
                    scroll,
                )
                .await
                .unwrap();
            }
            match redraw {
                RedrawType::PeakVolume(ident) => {
                    if let Some(index) = page_entries.iter().position(|p| *p == ident) {
                        if let Some(mut area) = is_entry_visible(index, scroll) {
                            area.y += 2;
                            area.height = 1;

                            let area = EntryWidget::calc_area(
                                visibility::check_lvl(
                                    index,
                                    &page_entries,
                                    parent_child_types(current_page).0,
                                ),
                                area,
                            );

                            let ent = match entries.get(&ident) {
                                Some(x) => x,
                                None => {
                                    break;
                                }
                            };
                            let vol = VolumeWidget::default().volume(ent.peak);
                            vol.render(area, &mut stdout).unwrap();
                        }
                    }
                }
                RedrawType::Entries => {
                    draw_entities(
                        &mut stdout,
                        &entries,
                        &page_entries,
                        &current_page,
                        selected,
                        scroll,
                    )
                    .await
                    .unwrap();
                }
                _ => {}
            };
        }
    });

    let x = e.join(w);
    x.await;
    Ok(())
}

pub trait Widget<W: Write> {
    fn render(self, area: Rect, buf: &mut W) -> Result<()>;
}

pub async fn draw_entities<W: Write>(
    stdout: &mut W,
    entries: &Entries,
    page_entries: &Vec<EntryIdentifier>,
    current_page: &PageType,
    selected: usize,
    scroll: usize,
) -> Result<()> {
    let (w, _) = crossterm::terminal::size()?;
    let mut entry_size = Rect::new(2, 2, w - 4, 3);
    let (parent_type, _) = parent_child_types(*current_page);

    for (i, lvl) in visibility::visible_range_with_lvl(page_entries.clone(), scroll, parent_type) {
        if lvl == EntrySpaceLvl::Empty {
            draw_rect!(stdout, " ", entry_size, get_style("normal"));
            entry_size.y += 3;
            continue;
        }

        let ent = match entries.get(&page_entries[i]) {
            Some(x) => x,
            None => {
                continue;
            }
        };

        let ew = EntryWidget::from(ent).bold(selected == i).position(lvl);
        ew.render(entry_size, stdout)?;
        entry_size.y += 3;
    }

    stdout.flush()?;

    Ok(())
}

pub async fn draw<W: Write>(
    stdout: &mut W,
    entries: &Entries,
    page_entries: &Vec<EntryIdentifier>,
    current_page: &PageType,
    selected: usize,
    scroll: usize,
) -> Result<()> {
    let (w, h) = crossterm::terminal::size()?;

    let b = BlockWidget::default()
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
