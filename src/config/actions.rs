use crate::{models::PageType, Action, RSError};

use std::convert::TryFrom;

impl ToString for Action {
    fn to_string(&self) -> String {
        match self {
            Action::ExitSignal => "exit".to_string(),
            Action::RequestMute => "mute".to_string(),
            Action::ChangePage(PageType::Output) => "show_output".to_string(),
            Action::ChangePage(PageType::Input) => "show_input".to_string(),
            Action::ChangePage(PageType::Cards) => "show_cards".to_string(),
            Action::OpenContextMenu => "context_menu".to_string(),
            Action::ShowHelp => "help".to_string(),
            Action::InputVolumeValue => "input_volume_value".to_string(),
            Action::RequstChangeVolume(num) => {
                if *num < 0 {
                    format!("lower_volume({})", num)
                } else {
                    format!("raise_volume({})", num)
                }
            }
            Action::MoveUp(num) => format!("up({})", num),
            Action::MoveDown(num) => format!("down({})", num),
            Action::MoveLeft => "left".to_string(),
            Action::MoveRight => "right".to_string(),
            Action::CyclePages(1) => "cycle_pages_forward".to_string(),
            Action::CyclePages(-1) => "cycle_pages_backward".to_string(),
            Action::CloseContextMenu => "close_context_menu".to_string(),
            Action::Confirm => "confirm".to_string(),
            Action::Hide => "hide".to_string(),

            _ => "".to_string(),
        }
    }
}

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
            "left" => Action::MoveLeft,
            "right" => Action::MoveRight,
            "cycle_pages_forward" => Action::CyclePages(1),
            "cycle_pages_backward" => Action::CyclePages(-1),
            "close_context_menu" => Action::CloseContextMenu,
            "confirm" => Action::Confirm,
            "hide" => Action::Hide,
            _ => {
                return Err(RSError::ActionBindingError(st.clone()));
            }
        };
        Ok(x)
    }
}
