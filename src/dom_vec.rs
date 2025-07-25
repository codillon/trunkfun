// A Codillon DOM "vector": a variable-length collection of Components of the same type

use crate::web_support::{Component, ElementComponent, ElementHandle, ElementRef, NodeRef};
use delegate::delegate;

pub struct DomVec<Child: Component, Element: AsRef<web_sys::HtmlElement>> {
    contents: Vec<Child>,
    elem: ElementHandle<Element>,
}

impl<Child: Component, Element: AsRef<web_sys::HtmlElement>> DomVec<Child, Element> {
    pub fn new(elem: ElementHandle<Element>) -> Self {
        Self {
            contents: Vec::new(),
            elem,
        }
    }

    pub fn push(&mut self, elem: Child) {
        self.contents.push(elem);
        self.elem.append_node(self.contents.last().unwrap().node());
    }

    pub fn set_contents(&mut self, elem: Child) {
        self.contents = vec![elem];
        self.elem.attach_node(self.contents.last().unwrap().node());
    }

    delegate! {
    to self.contents {
        pub fn get(&self, index: usize) -> Option<&Child>;
        pub fn get_mut(&mut self, index: usize) -> Option<&mut Child>;
    }
    to self.elem {
        pub fn set_attribute(&mut self, name: &str, value: &str);
    }
    }
}

impl<Child: Component, Element: AsRef<web_sys::HtmlElement>> Component for DomVec<Child, Element> {
    fn audit(&self) {
        self.elem.audit_attributes();
        let dom_children = self.elem.get_child_node_list();
        assert!(dom_children.length() == self.contents.len());
        for (index, elem) in self.contents.iter().enumerate() {
            elem.audit();
            dom_children.audit_node(index, elem.node());
        }
    }

    fn node(&self) -> NodeRef {
        self.elem.element().into()
    }
}

impl<Child: Component, Element: AsRef<web_sys::HtmlElement>> ElementComponent<Element>
    for DomVec<Child, Element>
{
    fn element(&self) -> ElementRef<Element> {
        self.elem.element()
    }
}
