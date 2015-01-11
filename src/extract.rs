use html5ever::tokenizer::{TokenSink, Token};
use html5ever::driver::{tokenize_to, one_input};
use std::default::Default;

struct LinkExtractor<'a> {
  into: &'a mut Vec<String>
}
impl<'a> TokenSink for LinkExtractor<'a>  {
  fn process_token(&mut self, token: Token) {
    match token {
      Token::TagToken(tag) => {
        if tag.name.as_slice() == "a" {
          self.into.extend(
            tag.attrs.into_iter()
            .filter(|attr| attr.name.local.as_slice() == "href")
            .map(|attr| attr.value)
            .take(1)
            );
        }
      },
      _ => ()
    }
  }
}
pub fn extract_links(document: String, into: &mut Vec<String>) {
  let linksink = LinkExtractor { into: into };

  tokenize_to(linksink, one_input(document), Default::default());
}

#[test]
fn extract_links_test() {
  use std::default::Default;
  let contents = "
    <body>
    <a href=\"test\">hi</a>
    <a href=\"othertest\">hi</a>
    <A HREF=\"capstest\">hi</a>
    </body>".to_string();

  let mut links = Vec::new();
  extract_links(contents, &mut links);
  assert_eq!(links, vec!["test", "othertest", "capstest"]);
}
