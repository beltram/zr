#[derive(Default, Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct ArgCmd {
    pub order: u8,
    pub cmd: String,
    pub is_default: bool,
}