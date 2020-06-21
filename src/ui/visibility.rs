use super::{parent_child_types, util::Rect, EntryIdentifier, EntryType, ENTRY_HEIGHT, Y_PADDING};
use std::cmp::{max, min};

pub fn adjust_scroll(
    page_entries: &Vec<EntryIdentifier>,
    scroll: &mut usize,
    selected: &mut usize,
) -> bool {
    let (_, h) = crossterm::terminal::size().unwrap();
    let h = h - *Y_PADDING;

    let sel = (*selected).clone();
    let scr = (*scroll).clone();

    if *selected >= page_entries.len() {
        if page_entries.len() > 0 {
            *selected = page_entries.len() - 1
        } else {
            *selected = 0;
        };
    }

    let num = h / (*ENTRY_HEIGHT);
    *scroll = *selected / num as usize;

    !(sel == *selected && scr == *scroll)
}

pub fn is_entry_visible(index: usize, scroll: usize) -> Option<Rect> {
    let (w, h) = crossterm::terminal::size().unwrap();
    let num = (h / (*ENTRY_HEIGHT)) as usize;

    let screen_i = (index as i32) - (scroll * num) as i32;

    if screen_i < 0 || screen_i >= num as i32 {
        return None;
    } else {
        return Some(Rect::new(
            2,
            2 + screen_i as u16 * (*ENTRY_HEIGHT),
            w - 4,
            *ENTRY_HEIGHT,
        ));
    }
}

#[derive(PartialEq)]
pub enum EntrySpaceLvl {
    Empty,
    Parent,
    ParentWithoutChild,
    MidChild,
    LastChild,
}

pub fn check_lvl(
    index: usize,
    page_entries: &Vec<EntryIdentifier>,
    parent_type: EntryType,
) -> EntrySpaceLvl {
    if index >= page_entries.len() {
        return EntrySpaceLvl::Empty;
    }
    if page_entries[index].entry_type == parent_type {
        if index + 1 >= page_entries.len() || page_entries[index + 1].entry_type == parent_type {
            return EntrySpaceLvl::ParentWithoutChild;
        } else {
            return EntrySpaceLvl::Parent;
        }
    } else {
        if index + 1 >= page_entries.len() || page_entries[index + 1].entry_type == parent_type {
            return EntrySpaceLvl::LastChild;
        } else {
            return EntrySpaceLvl::MidChild;
        }
    }
}

pub fn visible_range_with_lvl(
    page_entries: Vec<EntryIdentifier>,
    scroll: usize,
    parent_type: EntryType,
) -> Box<dyn Iterator<Item = (usize, EntrySpaceLvl)>> {
    let (_, h) = crossterm::terminal::size().unwrap();
    let h = h - *Y_PADDING;
    let num = (h / *ENTRY_HEIGHT) as usize;
    let start = min(num * scroll, page_entries.len());

    Box::new(
        (start..num * (scroll + 1)).map(move |x| -> (usize, EntrySpaceLvl) {
            (x, check_lvl(x, &page_entries, parent_type))
        }),
    )
}
