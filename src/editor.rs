use crate::{
    dom_struct::DomStruct,
    dom_text::DomText,
    dom_vec::DomVec,
    web_support::{Component, ElementComponent, ElementFactory, ElementRef, NodeRef},
};
use std::{cell::RefCell, collections::HashMap, rc::Rc};
use web_sys::{HtmlBrElement, HtmlDivElement, HtmlSpanElement, InputEvent};

type DomBr = DomStruct<(), HtmlBrElement>;
type LineContents = (DomText, (DomBr, ()));
type EditLine = DomStruct<LineContents, HtmlSpanElement>;

struct _Editor {
    next_id: usize,
    id_map: HashMap<usize, usize>,
    component: DomVec<EditLine, HtmlDivElement>,
}

pub struct Editor(Rc<RefCell<_Editor>>);

impl Editor {
    pub fn new(factory: &ElementFactory) -> Self {
        let mut inner = _Editor {
            next_id: 0,
            id_map: HashMap::default(),
            component: DomVec::new(factory.div()),
        };

        inner.component.set_attribute("class", "textentry");
        inner.component.set_attribute("contenteditable", "true");
        inner.component.set_attribute("spellcheck", "false");

        let mut ret = Editor(Rc::new(RefCell::new(inner)));

        let editor_ref = Rc::clone(&ret.0);
        ret.0
            .borrow_mut()
            .component
            .set_onbeforeinput(move |ev| Editor(editor_ref.clone()).handle_input(ev));

        ret.push_line(factory, "Hello, world.");

        ret
    }

    fn push_line(&mut self, factory: &ElementFactory, string: &str) {
        self.0.borrow_mut().component.push(EditLine::new(
            (DomText::new(string), (DomBr::new((), factory.br()), ())),
            factory.span(),
        ));
    }

    fn handle_input(&mut self, ev: InputEvent) {
        ev.prevent_default();
        web_sys::console::log_1(
            &format!(
                "got: {} + {}",
                ev.input_type(),
                ev.data().unwrap_or_default()
            )
            .into(),
        );
    }
}

impl Component for Editor {
    fn audit(&self) {
        self.0.borrow().audit()
    }

    fn node(&self) -> NodeRef {
        self.0.borrow().into()
    }
}

impl ElementComponent<HtmlDivElement> for Editor {
    fn element(&self) -> ElementRef<HtmlDivElement> {
        self.0.borrow().into()
    }
}

impl Component for _Editor {
    fn audit(&self) {
        self.component.audit()
    }

    fn node(&self) -> NodeRef {
        self.component.node()
    }
}

impl ElementComponent<HtmlDivElement> for _Editor {
    fn element(&self) -> ElementRef<HtmlDivElement> {
        self.component.element()
    }
}
