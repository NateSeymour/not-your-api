pub enum Table {
    IAM,
    TASKER,
}

impl Table {
    pub fn as_str(&self) -> &'static str {
        match self {
            Table::IAM => "NYS_iam",
            Table::TASKER => "NYS_tasker",
        }
    }
}