use anyhow::Result;
use trunkfun::{
    dom_struct::*,
    dom_text::DomText,
    dom_vec::DomVec,
    web_support::{Component, DocumentHandle, ElementComponent},
};

type DomBr = DomStruct<(), web_sys::HtmlBrElement>;

type LineNumber = DomStruct<(DomText, ()), web_sys::HtmlSpanElement>;

type EditLine = (LineNumber, (DomText, (DomBr, ())));

trait NewLine {
    fn new(index: usize, string: &str, doc: &DocumentHandle) -> Self;
}

impl NewLine for EditLine {
    fn new(index: usize, string: &str, doc: &DocumentHandle) -> Self {
        let mut ret = (
            DomStruct::new((DomText::new(&format!("{index}. ")), ()), doc.create_span()),
            (DomText::new(string), (DomBr::new((), doc.create_br()), ())),
        );
        ret.0.set_attribute("contenteditable", "false");
        ret
    }
}

fn init() -> Result<()> {
    let document = DocumentHandle::default();
    let mut body = DomVec::new(document.create_body());
    document.set_body(body.element());

    body.set_contents(DomVec::new(document.create_div()));
    let container = body.get_mut(0).unwrap();

    container.set_attribute("class", "textentry");
    container.set_attribute("contenteditable", "true");
    container.set_attribute("spellcheck", "false");

    container.push(DomStruct::new(
        EditLine::new(0, "Hello, world.", &document),
        document.create_span(),
    ));

    container.push(DomStruct::new(
        EditLine::new(1, "How are you?", &document),
        document.create_span(),
    ));

    container.push(DomStruct::new(
        EditLine::new(2, "Fine, thanks!", &document),
        document.create_span(),
    ));

    body.audit();

    web_sys::console::log_1(&"successful audit".into());

    Ok(())
}

fn main() {
    console_error_panic_hook::set_once();

    if let Err(x) = init() {
        panic!("error: {x}")
    }
}
