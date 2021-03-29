use std::convert::TryFrom;

use crate::{
    models::{PageType, UserAction},
    RsError,
};

impl ToString for UserAction {
    fn to_string(&self) -> String {
        match self {
            UserAction::RequestQuit => "exit".to_string(),
            UserAction::RequestMute(_) => "mute".to_string(),
            UserAction::ChangePage(PageType::Output) => "show_output".to_string(),
            UserAction::ChangePage(PageType::Input) => "show_input".to_string(),
            UserAction::ChangePage(PageType::Cards) => "show_cards".to_string(),
            UserAction::OpenContextMenu(_) => "context_menu".to_string(),
            UserAction::ShowHelp => "help".to_string(),
            UserAction::RequstChangeVolume(num, _) => {
                if *num < 0 {
                    format!("lower_volume({})", num)
                } else {
                    format!("raise_volume({})", num)
                }
            }
            UserAction::MoveUp(num) => format!("up({})", num),
            UserAction::MoveDown(num) => format!("down({})", num),
            UserAction::MoveLeft => "left".to_string(),
            UserAction::MoveRight => "right".to_string(),
            UserAction::CyclePages(x) => {
                if *x > 0 {
                    "cycle_pages_forward".to_string()
                } else {
                    "cycle_pages_backward".to_string()
                }
            }
            UserAction::CloseContextMenu => "close_context_menu".to_string(),
            UserAction::Confirm => "confirm".to_string(),
            UserAction::Hide(_) => "hide".to_string(),
            UserAction::SetSelected(_) => "unsupported".to_string(),
        }
    }
}

impl TryFrom<String> for UserAction {
    type Error = RsError;

    fn try_from(st: String) -> Result<UserAction, Self::Error> {
        let mut s = &st[..];
        let mut a = String::new();

        if let Some(lparen) = st.chars().position(|c| c == '(') {
            let rparen = match st.chars().position(|c| c == ')') {
                Some(r) => r,
                None => {
                    return Err(RsError::ActionBindingError(st.clone()));
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
            "exit" => UserAction::RequestQuit,
            "mute" => UserAction::RequestMute(None),
            "show_output" => UserAction::ChangePage(PageType::Output),
            "show_input" => UserAction::ChangePage(PageType::Input),
            "show_cards" => UserAction::ChangePage(PageType::Cards),
            "context_menu" => UserAction::OpenContextMenu(None),
            "help" => UserAction::ShowHelp,
            "lower_volume" => {
                let a = match a.parse::<i16>() {
                    Ok(x) => x,
                    Err(_) => {
                        return Err(RsError::ActionBindingError(st.clone()));
                    }
                };
                UserAction::RequstChangeVolume(-a, None)
            }
            "raise_volume" => {
                let a = match a.parse::<i16>() {
                    Ok(x) => x,
                    Err(_) => {
                        return Err(RsError::ActionBindingError(st.clone()));
                    }
                };
                UserAction::RequstChangeVolume(a, None)
            }
            "up" => {
                let a = match a.parse::<u16>() {
                    Ok(x) => x,
                    Err(_) => {
                        return Err(RsError::ActionBindingError(st.clone()));
                    }
                };
                UserAction::MoveUp(a)
            }
            "down" => {
                let a = match a.parse::<u16>() {
                    Ok(x) => x,
                    Err(_) => {
                        return Err(RsError::ActionBindingError(st.clone()));
                    }
                };
                UserAction::MoveDown(a)
            }
            "left" => UserAction::MoveLeft,
            "right" => UserAction::MoveRight,
            "cycle_pages_forward" => UserAction::CyclePages(1),
            "cycle_pages_backward" => UserAction::CyclePages(-1),
            "close_context_menu" => UserAction::CloseContextMenu,
            "confirm" => UserAction::Confirm,
            "hide" => UserAction::Hide(None),
            _ => {
                return Err(RsError::ActionBindingError(st.clone()));
            }
        };
        Ok(x)
    }
}
