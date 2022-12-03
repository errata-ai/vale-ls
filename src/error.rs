use thiserror::Error;

#[derive(Error, Debug)]
#[non_exhaustive]
pub(crate) enum Error {
    #[error(transparent)]
    Io(#[from] ::std::io::Error),
    #[error(transparent)]
    Zip(#[from] ::zip_extract::ZipExtractError),
    #[error(transparent)]
    Http(#[from] ::reqwest::Error),
    #[error(transparent)]
    Json(#[from] ::serde_json::Error),
    #[error(transparent)]
    Utf8(#[from] ::std::string::FromUtf8Error),
    #[error(transparent)]
    SemVer(#[from] ::semver::Error),
    #[error("{0}")]
    Msg(String),
}

impl From<&'static str> for Error {
    fn from(s: &'static str) -> Self {
        Error::Msg(s.to_owned())
    }
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error::Msg(s)
    }
}
