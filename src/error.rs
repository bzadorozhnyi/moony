#[derive(Debug)]
pub enum AppError {
    FailedOpenImg,
    FailedSaveImg,
    Other(String)
}