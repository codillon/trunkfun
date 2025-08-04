use crate::dom_br::DomBr;
use crate::dom_struct::*;
use crate::dom_text::*;
use crate::dom_vec::*;
use crate::web_support::Component;
use crate::web_support::ElementFactory;

type DomEditLine = DomStruct<(DomText, (DomBr, ())), web_sys::HtmlDivElement>;

struct EditLine {
    dom_editline: DomEditLine,
    id: usize,
}

impl EditLine {
    pub fn new(factory: &ElementFactory, id: usize) -> Self {
        let dom_editline = DomEditLine::new(
            (
                DomText::new(&format!("Hello World, id {}", id)),
                (DomBr::new((), factory.br()), ()),
            ),
            factory.div(),
        );
        EditLine { dom_editline, id }
    }
}

impl Component for EditLine {
    delegate::delegate! {
        to self.dom_editline {
            fn audit(&self);
            fn node<'a>(&'a self) -> crate::web_support::NodeRef<'a>;
        }
    }
}

type DomEditor = DomVec<EditLine, web_sys::HtmlDivElement>;
pub struct Editor {
    dom_editor: DomEditor,
    factory: ElementFactory,
    next_id: usize,
}

impl Component for Editor {
    delegate::delegate! {
        to self.dom_editor {
            fn audit(&self);
            fn node<'a>(&'a self) -> crate::web_support::NodeRef<'a>;
        }
    }
}

impl Editor {
    pub fn new(factory: ElementFactory) -> Editor {
        let mut div = factory.div();
        div.set_attribute("class", "textentry");
        div.set_attribute("contenteditable", "true");
        div.set_attribute("spellcheck", "false");
        let dom_editor = DomVec::new(div);
        Editor {
            dom_editor,
            factory,
            next_id: 0,
        }
    }

    pub fn insert(&mut self, index: usize) {
        self.dom_editor
            .insert(index, EditLine::new(&self.factory, self.next_id));
        self.next_id += 1;
    }
}
