use anyhow::Result;
use std::cell::RefCell;
use trunkfun::{
    dom_struct::*,
    dom_text::DomText,
    dom_vec::DomVec,
    web_support::{DocumentHandle, ElementFactory},
};

type DomBr = DomStruct<(), web_sys::HtmlBrElement>;

type LineNumber = DomStruct<(DomText, ()), web_sys::HtmlSpanElement>;

type EditLineType = (LineNumber, (DomText, (DomBr, ())));

type EditLine = DomStruct<EditLineType, web_sys::HtmlSpanElement>;

type Editor = DomVec<EditLine, web_sys::HtmlDivElement>;

type Body = DomStruct<(Editor, ()), web_sys::HtmlBodyElement>;

type Document = DocumentHandle<Body>;

fn new_line(index: usize, string: &str, factory: &ElementFactory) -> EditLine {
    let mut inner = (
        DomStruct::new((DomText::new(&format!("{index}. ")), ()), factory.span()),
        (DomText::new(string), (DomBr::new((), factory.br()), ())),
    );
    inner.0.set_attribute("contenteditable", "false");
    EditLine::new(inner, factory.span())
}

thread_local! {
    static DOCUMENT: RefCell<Document> = Document::default().into();
}

fn init(doc: &RefCell<Document>) -> Result<()> {
    let mut doc = doc.borrow_mut();
    let factory = doc.element_factory();
    doc.set_body(Body::new((Editor::new(factory.div()), ()), factory.body()));
    let body = doc.body_mut().expect("body");

    let editor: &mut Editor = &mut body.get_mut().0;

    editor.set_attribute("class", "textentry");
    editor.set_attribute("contenteditable", "true");
    editor.set_attribute("spellcheck", "false");

    editor.push(new_line(0, "Hello, world.", &factory));
    editor.push(new_line(1, "This is a test.", &factory));
    editor.push(new_line(2, "Here's another line.", &factory));

    editor.remove(1);

    editor
        .get_mut(1)
        .unwrap()
        .get_mut()
        .0
        .get_mut()
        .0
        .set_data("1. ");

    editor
        .get_mut(1)
        .unwrap()
        .get_mut()
        .1
        .0
        .set_data("This is now the second line.");

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
