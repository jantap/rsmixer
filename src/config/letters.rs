use crate::{
    models::PageType,
    Letter, RSError,
};

use std::convert::TryFrom;

impl TryFrom<String> for Letter {
    type Error = RSError;

    fn try_from(st: String) -> Result<Letter, Self::Error> {
        let mut s = &st[..];
        let mut a = String::new();

        if let Some(lparen) = st.chars().position(|c| c == '(') {
            let rparen = match st.chars().position(|c| c == ')') {
                Some(r) => r,
                None => {
                    return Err(RSError::ActionBindingError(st.clone()));
                }
            };
            a = st
                .chars()
                .skip(lparen + 1)
                .take(rparen - lparen - 1)
                .collect();
            s = &st[0..lparen];
        }

        let x = match s {
            "exit" => Letter::ExitSignal,
            "mute" => Letter::RequestMute,
            "show_output" => Letter::ChangePage(PageType::Output),
            "show_input" => Letter::ChangePage(PageType::Input),
            "show_cards" => Letter::ChangePage(PageType::Cards),
            "context_menu" => Letter::OpenContextMenu,
            "help" => Letter::ShowHelp,
            "lower_volume" => {
                let a = match a.parse::<i16>() {
                    Ok(x) => x,
                    Err(_) => {
                        return Err(RSError::ActionBindingError(st.clone()));
                    }
                };
                Letter::RequstChangeVolume(-a)
            }
            "raise_volume" => {
                let a = match a.parse::<i16>() {
                    Ok(x) => x,
                    Err(_) => {
                        return Err(RSError::ActionBindingError(st.clone()));
                    }
                };
                Letter::RequstChangeVolume(a)
            }
            "up" => {
                let a = match a.parse::<u16>() {
                    Ok(x) => x,
                    Err(_) => {
                        return Err(RSError::ActionBindingError(st.clone()));
                    }
                };
                Letter::MoveUp(a)
            }
            "down" => {
                let a = match a.parse::<u16>() {
                    Ok(x) => x,
                    Err(_) => {
                        return Err(RSError::ActionBindingError(st.clone()));
                    }
                };
                Letter::MoveDown(a)
            }
            "cycle_pages_forward" => Letter::CyclePages(1),
            "cycle_pages_backward" => Letter::CyclePages(-1),
            "close_context_menu" => Letter::CloseContextMenu,
            "hide" => Letter::Hide,
            _ => {
                return Err(RSError::ActionBindingError(st.clone()));
            }
        };
        Ok(x)
    }
}
