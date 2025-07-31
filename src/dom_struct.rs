// A Codillon DOM "struct": a (possibly empty) collection
// of heterogeneous Components of (possibly) different types.

use crate::web_support::{
    AnyElement, ArrayHandle, Component, ElementComponent, ElementHandle, ElementRef,
    NodeListHandle, NodeRef,
};
use delegate::delegate;

pub struct DomStruct<Child: Sequence, Element: AnyElement> {
    contents: Child,
    elem: ElementHandle<Element>,
}

pub trait Sequence {
    const LEN: usize;
    fn install(&self, nodes: &mut ArrayHandle, index: usize);
    fn audit(&self, node_list: &NodeListHandle, index: usize);
}

impl Sequence for () {
    const LEN: usize = 0;
    fn install(&self, nodes: &mut ArrayHandle, index: usize) {
        assert_eq!(index, nodes.length());
    }
    fn audit(&self, node_list: &NodeListHandle, index: usize) {
        assert_eq!(index, node_list.length());
    }
}

impl<First: Component, Rest: Sequence> Sequence for (First, Rest) {
    const LEN: usize = Rest::LEN + 1;
    fn install(&self, nodes: &mut ArrayHandle, index: usize) {
        assert_eq!(index + Self::LEN, nodes.length());
        nodes.set(index, self.0.node());
        self.1.install(nodes, index + 1);
    }
    fn audit(&self, node_list: &NodeListHandle, index: usize) {
        assert_eq!(index + Self::LEN, node_list.length());
        node_list.audit_node(index, self.0.node());
        self.0.audit();
        self.1.audit(node_list, index + 1);
    }
}

impl<Child: Sequence, Element: AnyElement> DomStruct<Child, Element> {
    pub fn new(contents: Child, elem: ElementHandle<Element>) -> Self {
        let mut child_nodes = ArrayHandle::new_with_length(Child::LEN);
        contents.install(&mut child_nodes, 0);
        elem.attach_nodes(child_nodes);
        Self { contents, elem }
    }

    pub fn get(&self) -> &Child {
        &self.contents
    }

    pub fn get_mut(&mut self) -> &mut Child {
        &mut self.contents
    }

    pub fn set_contents(&mut self, new_contents: Child) {
        self.contents = new_contents;
    }

    delegate! {
    to self.elem {
    pub fn set_attribute(&mut self, name: &str, value: &str);
        pub fn set_onbeforeinput<F: Fn(web_sys::InputEvent) + 'static>(&mut self, handler: F);
    }
    }
}

impl<Child: Sequence, Element: AnyElement> Component for DomStruct<Child, Element> {
    fn audit(&self) {
        self.elem.audit();
        let dom_children = self.elem.get_child_node_list();
        assert_eq!(dom_children.length(), Child::LEN);
        self.contents.audit(&dom_children, 0);
    }

    fn node(&self) -> NodeRef {
        self.elem.element().into()
    }
}

impl<Child: Sequence, Element: AnyElement> ElementComponent<Element> for DomStruct<Child, Element> {
    fn element(&self) -> ElementRef<Element> {
        self.elem.element()
    }
}
