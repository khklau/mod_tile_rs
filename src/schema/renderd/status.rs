use std::time::SystemTime;


pub enum DataImportStatus {
    InProgress,
    Completed(SystemTime),
}
