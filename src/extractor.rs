use html5ever::sink::rcdom::Handle;
use html5ever::sink::common::Element;

pub fn extract_links(handle: &Handle, into: &mut Vec<String>) {
  let node = handle.borrow();

  
  let mylink = match node.node {
    Element(ref name, ref attrs) if name.local.as_slice() == "a" =>  {
      attrs.iter()
        .filter_map(|attr|
                    if attr.name.local.as_slice() == "href" {
                      Some(attr.value.clone())
                    } else {
                      None
                    })
      .last()
    },
    _ => None
  }.into_iter();
  
  into.extend(mylink);

  for child in node.children.iter() {
    extract_links(child, into);
  }
}

#[test]
fn extract_links_test() {
  use html5ever::sink::rcdom::RcDom;
  use html5ever::{parse, one_input};
  use std::default::Default;
  let contents = "
    <body>
    <a href=\"test\">hi</a>
    <a href=\"othertest\">hi</a>
    <A HREF=\"capstest\">hi</a>
    </body>".to_string();

  let dom: RcDom = parse(one_input(contents), Default::default());
  let mut links = Vec::new();
  extract_links(&dom.document, &mut links);
  assert_eq!(links, vec!["test", "othertest", "capstest"]);
}
