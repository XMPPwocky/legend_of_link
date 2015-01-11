#![allow(unstable)]

extern crate html5ever;
extern crate url;

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

  let all_valid = print_report(report);
  if all_valid {
    println!("Congratulations! No dead links were found!");
    os::set_exit_status(0);
  } else {
    os::set_exit_status(1);
  }
}

// it's sort of ugly to have this return a bool
// but the traversal is pretty expensive so can't do it twice
// later we could generate a "report report" with higher level info
// or prune out all the valid pages?
fn print_report(r: CheckReport) -> bool {
  let mut all_valid = true;

  println!("Checked {} pages...", r.len());
  println!("Errors:");

  for (url, mut report) in r.into_iter() {
    use PageStatus::{Invalid, InProgress};
    match report.status {
      Invalid(ref err) => {
        all_valid = false;
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

  all_valid
}

fn check_root(url: Url) -> CheckReport {
  let mut report: CheckReport = HashMap::new();

  let mut urls_checking = Vec::<Url>::new();
  let mut urls_to_check = Vec::<Url>::new();

  urls_checking.push(url);

  while urls_checking.len() > 0 {
    println!("Checked {} pages, checking {}...",
             report.len(),
             urls_checking.len());

    for url in urls_checking.drain() {
      use std::collections::hash_map::Entry;
      
      let result = check(&url, &mut report, &mut urls_to_check);
      let status = match result { 
        Ok(()) => PageStatus::Valid,
        Err(e) => PageStatus::Invalid(e)
      };
      match report.entry(url.clone()) {
        Entry::Occupied(mut o) => {
          // note that the status here is not necessarily InProgress
          // however, it is less expensive to do the checking twice
          // than it is to actually check for duplicates.
          let report = o.get_mut();
          report.status = status;
          if report.status == PageStatus::Valid {
            report.references = Vec::new();
          } 
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

  let contents = try!(fetch_path(path));

  let mut links = Vec::new();
  extractor::extract_links(contents, &mut links);

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

fn fetch_path(path: Path) -> Result<String, CheckError> {
  use std::io::File;

  Ok(try!(File::open(&path).read_to_string()))
}
