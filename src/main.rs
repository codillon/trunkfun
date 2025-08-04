use anyhow::Result;
use std::cell::RefCell;
use trunkfun::{
    dom_struct::*,
    web_support::DocumentHandle,
};

use trunkfun::editor::Editor;

type Body = DomStruct<(Editor, ()), web_sys::HtmlBodyElement>;

type Document = DocumentHandle<Body>;

thread_local! {
    static DOCUMENT: RefCell<Document> = Document::default().into();
}

fn init(doc: &RefCell<Document>) -> Result<()> {
    let doc = doc.borrow_mut();

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
