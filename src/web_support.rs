// Codillon "web support" structs to be used by Components.
// These wrap web_sys types to prevent "unsafe" access to the underlying DOM object.
// The goal is to enforce modularity between Components, and prevent a Component
// from modifying a DOM object belonging to another. This means that Components
// cannot directly access the children or parents of a DOM node.

use delegate::delegate;
use std::collections::HashMap;
use web_sys::wasm_bindgen::JsCast;

// Bare wrappers for a DOM Node and Element.
pub struct NodeRef<'a>(&'a web_sys::Node);
pub struct ElementRef<'a, T: AsRef<web_sys::HtmlElement>>(&'a T);

impl<'a, T: AsRef<web_sys::HtmlElement>> From<ElementRef<'a, T>> for NodeRef<'a> {
    fn from(val: ElementRef<'a, T>) -> Self {
        NodeRef(val.0.as_ref())
    }
}

// Wrapper for a DOM Text node, allowing access to and modification of its CharacterData's data.
// Access to the underlying Node is only via a NodeRef.
pub struct TextHandle(web_sys::Text);

impl Default for TextHandle {
    fn default() -> Self {
        Self(web_sys::Text::new().expect("Text::new()"))
    }
}

impl TextHandle {
    pub fn node<'a>(&'a self) -> NodeRef<'a> {
        NodeRef(&self.0)
    }

    delegate! {
    to self.0 {
        pub fn data(&self) -> String;
    pub fn set_data(&self, value: &str);
    #[unwrap] // no return value anyway
    pub fn append_data(&self, data: &str);
    #[unwrap] // no return value anyway
    pub fn insert_data(&self, offset: u32, data: &str);
    }
    }
}

// Wrapper for a DOM Element, allowing access to and modification of its attributes
// and the ability to set and append to its child nodes (as NodeRefs or a ArrayHandle).
pub struct ElementHandle<T: AsRef<web_sys::HtmlElement>> {
    elem: T,
    attributes: HashMap<String, String>,
}

impl<T: AsRef<web_sys::HtmlElement>> ElementHandle<T> {
    fn new(elem: T) -> Self {
        Self {
            elem,
            attributes: HashMap::default(),
        }
    }

    pub fn append_node(&self, child: NodeRef) {
        self.elem.as_ref().append_with_node_1(child.0).unwrap() // no return value anyway
    }

    pub fn attach_node(&self, child: NodeRef) {
        self.elem.as_ref().replace_children_with_node_1(child.0);
    }

    pub fn attach_nodes(&self, children: ArrayHandle) {
        self.elem.as_ref().replace_children_with_node(&children.0);
    }

    pub fn set_attribute(&mut self, name: &str, value: &str) {
        self.attributes.insert(name.to_string(), value.to_string());
        self.elem.as_ref().set_attribute(name, value).unwrap();
    }

    pub fn audit_attributes(&self) {
        for (key, value) in &self.attributes {
            if let Some(dom_value) = self.elem.as_ref().get_attribute(key) {
                assert!(dom_value == *value);
            } else {
                panic!("missing {key} (expected value {value})");
            }
        }

        for dom_key in self.elem.as_ref().get_attribute_names() {
            assert!(self.attributes.contains_key(&dom_key.as_string().unwrap()));
        }
    }

    pub fn element<'a>(&'a self) -> ElementRef<'a, T> {
        ElementRef(&self.elem)
    }

    pub fn get_child_node_list(&self) -> NodeListHandle {
        NodeListHandle(self.elem.as_ref().child_nodes())
    }
}

// Wrapper for a DOM Document, allowing modification of the body and
// the ability to create Elements (as ElementHandles).
pub struct DocumentHandle(web_sys::Document);

impl Default for DocumentHandle {
    fn default() -> Self {
        Self(web_sys::window().unwrap().document().unwrap())
    }
}

impl DocumentHandle {
    pub fn set_body(&self, body: ElementRef<web_sys::HtmlBodyElement>) {
        self.0.set_body(Some(body.0));
    }

    fn create_element<T: JsCast>(&self, t: &str) -> T {
        self.0.create_element(t).unwrap().dyn_into::<T>().unwrap()
    }

    pub fn create_div(&self) -> ElementHandle<web_sys::HtmlDivElement> {
        ElementHandle::new(self.create_element("div"))
    }

    pub fn create_span(&self) -> ElementHandle<web_sys::HtmlSpanElement> {
        ElementHandle::new(self.create_element("span"))
    }

    pub fn create_paragraph(&self) -> ElementHandle<web_sys::HtmlParagraphElement> {
        ElementHandle::new(self.create_element("div"))
    }

    pub fn create_br(&self) -> ElementHandle<web_sys::HtmlBrElement> {
        ElementHandle::new(self.create_element("br"))
    }

    pub fn create_body(&self) -> ElementHandle<web_sys::HtmlBodyElement> {
        ElementHandle::new(self.create_element("body"))
    }
}

// Wrapper for a DOM NodeList, allowing audit that each entry matches an expected node.
pub struct NodeListHandle(web_sys::NodeList);

impl NodeListHandle {
    pub fn length(&self) -> usize {
        self.0.length() as usize
    }

    pub fn audit_node(&self, index: usize, node: NodeRef) {
        if let Some(actual) = self.0.item(index.try_into().expect("index -> u32"))
            && actual.is_same_node(Some(node.0))
        {
            return;
        }
        panic!("node {} mismatch (#{}/{})", index, index + 1, self.length())
    }
}

// Wrapper for a DOM Array, allowing modification of its entries.
pub struct ArrayHandle(js_sys::Array);

impl ArrayHandle {
    pub fn length(&self) -> usize {
        self.0.length() as usize
    }

    pub fn new_with_length(len: usize) -> Self {
        Self(js_sys::Array::new_with_length(
            len.try_into().expect("len -> u32"),
        ))
    }

    pub fn set<'a>(&mut self, index: usize, node: NodeRef<'a>) {
        self.0
            .set(index.try_into().expect("index -> u32"), node.0.into())
    }
}

// A trait for a safe "Component", allowing wrapped access to its root Node and audit
// that the DOM subtree matches the Component's expectations.
pub trait Component {
    fn audit(&self);
    fn node<'a>(&'a self) -> NodeRef<'a>;
}

// A Component that is also an HTML Element (i.e. not Text).
pub trait ElementComponent<T: AsRef<web_sys::HtmlElement>>: Component {
    fn element<'a>(&'a self) -> ElementRef<'a, T>;
}
