use anyhow::Result;
use std::cell::RefCell;
use trunkfun::dom_struct::*;
use trunkfun::editor::Editor;
use trunkfun::web_support::DocumentHandle;

type Body = DomStruct<(Editor, ()), web_sys::HtmlBodyElement>;

type Document = DocumentHandle<Body>;

thread_local! {
    static DOCUMENT: RefCell<Document> = Document::default().into();
}

fn init(doc: &RefCell<Document>) -> Result<()> {
    let mut doc = doc.borrow_mut();
    let factory = doc.element_factory();
    let selection = doc.get_selection().expect("Get Selection");
    doc.set_body(Body::new(
        (Editor::new(factory.clone(), selection), ()),
        factory.body(),
    ));

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
