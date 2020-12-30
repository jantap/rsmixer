use crate::{models::PageType, Action, RSError};

use std::convert::TryFrom;

impl TryFrom<String> for Action {
    type Error = RSError;

    fn try_from(st: String) -> Result<Action, Self::Error> {
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
            "exit" => Action::ExitSignal,
            "mute" => Action::RequestMute,
            "show_output" => Action::ChangePage(PageType::Output),
            "show_input" => Action::ChangePage(PageType::Input),
            "show_cards" => Action::ChangePage(PageType::Cards),
            "context_menu" => Action::OpenContextMenu,
            "help" => Action::ShowHelp,
            "input_volume_value" => Action::InputVolumeValue,
            "lower_volume" => {
                let a = match a.parse::<i16>() {
                    Ok(x) => x,
                    Err(_) => {
                        return Err(RSError::ActionBindingError(st.clone()));
                    }
                };
                Action::RequstChangeVolume(-a)
            }
            "raise_volume" => {
                let a = match a.parse::<i16>() {
                    Ok(x) => x,
                    Err(_) => {
                        return Err(RSError::ActionBindingError(st.clone()));
                    }
                };
                Action::RequstChangeVolume(a)
            }
            "up" => {
                let a = match a.parse::<u16>() {
                    Ok(x) => x,
                    Err(_) => {
                        return Err(RSError::ActionBindingError(st.clone()));
                    }
                };
                Action::MoveUp(a)
            }
            "down" => {
                let a = match a.parse::<u16>() {
                    Ok(x) => x,
                    Err(_) => {
                        return Err(RSError::ActionBindingError(st.clone()));
                    }
                };
                Action::MoveDown(a)
            }
            "cycle_pages_forward" => Action::CyclePages(1),
            "cycle_pages_backward" => Action::CyclePages(-1),
            "close_context_menu" => Action::CloseContextMenu,
            "hide" => Action::Hide,
            _ => {
                return Err(RSError::ActionBindingError(st.clone()));
            }
        };
        Ok(x)
    }
}
