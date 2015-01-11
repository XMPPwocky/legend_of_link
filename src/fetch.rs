use url::Url;

#[derive(Show, PartialEq)]
pub enum FetchError {
  MalformedUrl,
  NotFound,
  IoError(::std::io::IoError)
}

pub fn fetch_url(url: &Url) -> Result<String, FetchError> {
  let path = try!(url_to_path(url).map_err(|_| FetchError::MalformedUrl));
  
  fetch_path(path)
}

fn url_to_path(url: &Url) -> Result<Path, ()> {
  if &url.scheme[] == "file" {
    url.to_file_path()
  } else {
    Err(())
  }
}

fn fetch_path(path: Path) -> Result<String, FetchError> {
  use std::io::{File, IoError, IoErrorKind};

  let result = File::open(&path).read_to_string();
  result.map_err(|e| match e {
    IoError { kind: IoErrorKind::FileNotFound, .. } => {
      FetchError::NotFound
    },
    e => FetchError::IoError(e)
  })
}
