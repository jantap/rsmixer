use crate::ui::Rect;

#[derive(PartialEq, Clone, Debug)]
pub struct CardProfile {
    pub name: String,
    pub description: String,
    #[cfg(any(feature = "pa_v13"))]
    pub available: bool,
    pub area: Rect,
    pub is_selected: bool,
}
impl Eq for CardProfile {}

#[derive(PartialEq, Clone, Debug)]
pub struct CardEntry {
    pub profiles: Vec<CardProfile>,
    pub selected_profile: Option<usize>,
    pub area: Rect,
    pub is_selected: bool,
    pub name: String,
}
impl Eq for CardEntry {}

