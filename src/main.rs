#![allow(unstable)]

extern crate html5ever;
extern crate url;

use html5ever::sink::rcdom::RcDom;
use std::collections::HashMap;
use std::error::FromError;
use std::os;
use std::path::Path;
use url::Url;

mod extractor;

fn main() {
  let root = os::args().get(1)
    .and_then(|rooturl| Url::parse(&rooturl[]).ok());

  let report = match root {
    Some(url) => check_root(url),
    None => {
      println!("No URL or invalid URL specified! Usage: legend_of_link [url]");
      return;
    }
  };
  print_report(report);
  println!("Done checking!");
}

fn print_report(mut r: CheckReport) {
  println!("Checked {} pages...", r.len());
  println!("Errors:");
  for (url, mut report) in r.into_iter() {
    use PageStatus::{Invalid, InProgress};
    match report.status {
      Invalid(ref err) => {
        match err {
          &CheckError::IoError(std::io::IoError { kind: std::io::IoErrorKind::FileNotFound, .. }) => {
            println!("    NOT FOUND {:?}", url.serialize());
            for reference in report.references.into_iter() {
              println!("        - Referred to by {:?}", reference.serialize())
            }
          },
          _ => ()
        }
      },
      InProgress => unreachable!(),
      _ => ()
    }
  }
}

fn check_root(url: Url) -> CheckReport {
  let mut report: CheckReport = HashMap::new();

  let mut urls_checking = Vec::<Url>::new();
  let mut urls_to_check = Vec::<Url>::new();

  urls_checking.push(url);

  let mut last_info = 0;
  while urls_checking.len() > 0 {
    if report.len() - last_info > 10 {
      println!("{} {}", report.len(), urls_checking.len());
      last_info = report.len()
    }

    for url in urls_checking.drain() {
      use std::collections::hash_map::Entry;

      let result = check(&url, &mut report, &mut urls_to_check);
      let status = match result { 
        Ok(()) => PageStatus::Valid,
        Err(e) => PageStatus::Invalid(e)
      };
      match report.entry(url.clone()) {
        Entry::Occupied(mut o) => {
          debug_assert_eq!(o.get().status, PageStatus::InProgress);
          o.get_mut().status = status;
        },
        Entry::Vacant(v) => {
          v.insert(PageCheckReport {
            status: status,
            references: Vec::new()
          });
        }
      }
    }

    std::mem::swap(&mut urls_checking, &mut urls_to_check);
  }

  report
}

#[derive(PartialEq, Show)]
enum PageStatus {
  Valid,
  InProgress,
  Invalid(CheckError)
}

struct PageCheckReport {
  status: PageStatus,
  references: Vec<Url>
}
type CheckReport = HashMap<Url, PageCheckReport>;

#[derive(Show, PartialEq)]
enum CheckError {
  BadPath,
  IoError(std::io::IoError),
}
impl FromError<std::io::IoError> for CheckError {
  fn from_error(err: std::io::IoError) -> CheckError {
    CheckError::IoError(err)
  }
}

fn check(this: &Url, report: &mut CheckReport, urls_to_check: &mut Vec<Url>) -> Result<(), CheckError> {
  let path = try!(
    url_to_path(this).map_err(|_| CheckError::BadPath)
    );

  let dom = try!(read_and_parse_path(path));

  let mut links = Vec::new();
  extractor::extract_links(&dom.document, &mut links);

  let mut urls = links.into_iter()
    .filter_map(|link| normalize_url(&link[], this).ok() );

  for url in urls {
    use std::collections::hash_map::Entry;
    match report.entry(url.clone()) {
      Entry::Occupied(mut o) => {
        let should_track_references = match o.get().status {
          PageStatus::Valid => false,
          _ => true
        };
        if should_track_references {
          o.get_mut().references.push(this.clone());
        }
      },
      Entry::Vacant(v) => {
        v.insert(PageCheckReport {
          status: PageStatus::InProgress,
          references: vec![this.clone()]
        });
        urls_to_check.push(url);
      }
    };
  }

  Ok(())
}

fn normalize_url(url: &str, base: &Url) -> Result<Url, ()> {
  let mut parser = url::UrlParser::new();
  let parser = parser.base_url(base);

  let mut abs_url = try!(parser.parse(url).map_err(|_| ()));
  abs_url.fragment = None;
  Ok(abs_url)
}

fn url_to_path(url: &Url) -> Result<Path, ()> {
  if &url.scheme[] == "file" {
    url.to_file_path()
  } else {
    Err(())
  }
}

fn read_and_parse_path(path: Path) -> Result<RcDom, CheckError> {
  use std::default::Default;
  use std::io::File;
  use html5ever::one_input;
  use html5ever::parse;

  let contents = try!(File::open(&path).read_to_string());
  Ok(parse(one_input(contents), Default::default()))
}
