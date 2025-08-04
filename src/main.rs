use anyhow::Result;
use std::cell::RefCell;
use trunkfun::dom_struct::*;
use trunkfun::editor::_Editor;
use trunkfun::web_support::DocumentHandle;

type Body = DomStruct<(_Editor, ()), web_sys::HtmlBodyElement>;

type Document = DocumentHandle<Body>;

thread_local! {
    static DOCUMENT: RefCell<Document> = Document::default().into();
}

fn init(doc: &RefCell<Document>) -> Result<()> {
    let mut doc = doc.borrow_mut();
    let factory = doc.element_factory();
    let selection = doc.get_selection().expect("Get Selection");
    doc.set_body(Body::new(
        (_Editor::new(factory.clone(), selection), ()),
        factory.body(),
    ));
    let body = doc.body_mut().expect("body");
    let editor = &mut body.get_mut().0;
    editor.insert(0);
    editor.insert(1);
    editor.insert(2);
    editor.insert(3);
    editor.insert(4);
    editor.insert(2);

    doc.audit();

    web_sys::console::log_1(&"successful audit".into());

    Ok(())
}

fn main() {
    console_error_panic_hook::set_once();
    if let Err(x) = DOCUMENT.with(init) {
        panic!("error: {x}")
    }
}
