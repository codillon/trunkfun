// Codillon "web support" structs to be used by Components.
// These wrap web_sys types to prevent "unsafe" access to the underlying DOM object.
// The goal is to enforce modularity between Components, and prevent a Component
// from modifying a DOM object belonging to another. This means that Components
// cannot directly access the children or parents of a DOM node.

use delegate::delegate;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use wasm_bindgen::closure::Closure;
use web_sys::wasm_bindgen::JsCast;

// Bare wrappers for a DOM Node and Element.
pub struct NodeRef<'a>(&'a web_sys::Node);
pub struct ElementRef<'a, T: AsRef<web_sys::HtmlElement>>(&'a T);

impl<'a, T: AsRef<web_sys::HtmlElement>> From<ElementRef<'a, T>> for NodeRef<'a> {
    fn from(val: ElementRef<'a, T>) -> Self {
        NodeRef(val.0.as_ref())
    }
}
pub struct NodeReader<T: AsRef<web_sys::Node>>(T);

pub struct ElementReader<T: AsRef<web_sys::Element>>(T);

impl<T: AsRef<web_sys::Node>> NodeReader<T> {
    pub fn parent_node(&self) -> Option<NodeReader<web_sys::Node>> {
        self.0.as_ref().parent_node().map(|node| NodeReader(node))
    }

    pub fn parent_lement(&self) -> Option<ElementReader<web_sys::Element>> {
        self.0.as_ref().parent_element().map(|elem| ElementReader(elem))
    }
}

impl<T: AsRef<web_sys::Node> + JsCast, U: AsRef<web_sys::Element> + JsCast + Clone>
    TryFrom<NodeReader<T>> for ElementReader<U>
{
    type Error = NodeReader<T>;
    fn try_from(value: NodeReader<T>) -> Result<Self, Self::Error> {
        let node_ref: &web_sys::Node = value.0.as_ref();
        if let Some(elem) = node_ref.dyn_ref::<U>() {
            std::result::Result::Ok(ElementReader(elem.clone()))
        } else {
            std::result::Result::Err(value)
        }
    }
}

impl<T: AsRef<web_sys::Element>> ElementReader<T> {
    pub fn parent_node(&self) -> Option<NodeReader<web_sys::Node>> {
        self.0.as_ref().parent_node().map(|node| NodeReader(node))
    }

    pub fn parent_lement(&self) -> Option<ElementReader<web_sys::Element>> {
        self.0.as_ref().parent_element().map(|elem| ElementReader(elem))
    }

    pub fn get_attr(&self, attr: &str) -> Option<String> {
        self.0.as_ref().get_attribute(attr)
    }
}

// Wrapper for a Node or Element that removes it from its parent when dropped
struct AutoRemove<T: AsRef<web_sys::Node>>(T);

impl<T: AsRef<web_sys::Node>> AutoRemove<T> {}

impl<T: AsRef<web_sys::Node>> From<T> for AutoRemove<T> {
    fn from(t: T) -> Self {
        Self(t)
    }
}

impl<T: AsRef<web_sys::Node>> Deref for AutoRemove<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T: AsRef<web_sys::Node>> DerefMut for AutoRemove<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T: AsRef<web_sys::Node>> Drop for AutoRemove<T> {
    fn drop(&mut self) {
        if let Some(parent) = self.0.as_ref().parent_node() {
            parent.remove_child(self.0.as_ref()).expect("remove_child");
        }
    }
}

// Wrapper for a DOM Text node, allowing access to and modification of its CharacterData's data.
// Access to the underlying Node is only via a NodeRef.
pub struct TextHandle(AutoRemove<web_sys::Text>);

impl Default for TextHandle {
    fn default() -> Self {
        Self(web_sys::Text::new().expect("Text::new()").into())
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

pub trait AnyElement: AsRef<web_sys::HtmlElement> + AsRef<web_sys::Node> {
    fn element(&self) -> &web_sys::HtmlElement {
        self.as_ref()
    }
}
impl<T: AsRef<web_sys::HtmlElement> + AsRef<web_sys::Node>> AnyElement for T {}

#[derive(Default)]
struct Handlers {
    beforeinput: Option<Closure<dyn Fn(web_sys::InputEvent)>>,
}

// Wrapper for a DOM Element, allowing access to and modification of its attributes
// and event handlers, and the ability to set and append to its child nodes (as NodeRefs
// or a ArrayHandle).
pub struct ElementHandle<T: AnyElement> {
    elem: AutoRemove<T>,
    attributes: HashMap<String, String>,
    event_handlers: Handlers,
}

impl<T: AnyElement> ElementHandle<T> {
    fn new(elem: T) -> Self {
        Self {
            elem: elem.into(),
            attributes: HashMap::default(),
            event_handlers: Handlers::default(),
        }
    }

    pub fn append_node(&self, child: NodeRef) {
        self.elem.element().append_with_node_1(child.0).unwrap() // no return value anyway
    }

    pub fn insert_node(&self, index: usize, child: NodeRef) {
        self.elem
            .element()
            .insert_before(
                child.0,
                self.elem
                    .element()
                    .child_nodes()
                    .item(index as u32)
                    .as_ref(),
            )
            .expect("insert node");
        // self.elem.element().append_with_node_1(child.0).unwrap()
    }

    pub fn attach_node(&self, child: NodeRef) {
        self.elem.element().replace_children_with_node_1(child.0);
    }

    pub fn attach_nodes(&self, children: ArrayHandle) {
        self.elem.element().replace_children_with_node(&children.0);
    }

    pub fn set_attribute(&mut self, name: &str, value: &str) {
        self.attributes.insert(name.to_string(), value.to_string());
        self.elem.element().set_attribute(name, value).unwrap();
    }

    pub fn audit(&self) {
        for (key, value) in &self.attributes {
            if let Some(dom_value) = self.elem.element().get_attribute(key) {
                assert_eq!(dom_value, *value);
            } else {
                panic!("missing {key} (expected value {value})");
            }
        }

        for dom_key in self.elem.element().get_attribute_names() {
            assert!(self.attributes.contains_key(&dom_key.as_string().unwrap()));
        }

        match (
            &self.event_handlers.beforeinput,
            self.elem.element().onbeforeinput(),
        ) {
            (Some(expect), Some(actual)) => assert_eq!(actual, *expect.as_ref().unchecked_ref()),
            (Some(_), None) => panic!("missing event handler"),
            (None, Some(_)) => panic!("unexpected event handler"),
            (None, None) => (),
        }
    }

    pub fn element<'a>(&'a self) -> ElementRef<'a, T> {
        ElementRef(&self.elem)
    }

    pub fn get_child_node_list(&self) -> NodeListHandle {
        NodeListHandle(self.elem.element().child_nodes())
    }

    pub fn set_onbeforeinput<F: Fn(web_sys::InputEvent) + 'static>(&mut self, handler: F) {
        self.event_handlers.beforeinput = Some(Closure::new(handler));
        self.elem.element().set_onbeforeinput(Some(
            self.event_handlers
                .beforeinput
                .as_ref()
                .unwrap()
                .as_ref()
                .unchecked_ref(),
        ));
    }
}

#[derive(Debug)]
pub struct SelectionHandle(web_sys::Selection);

impl SelectionHandle {
    pub fn get_focus_node(&self) -> Option<NodeReader<web_sys::Node>> {
        self.0.focus_node().map(|node| NodeReader(node))
    }

    pub fn get_anchor_node(&self) -> Option<NodeReader<web_sys::Node>> {
        self.0.anchor_node().map(|node| NodeReader(node))
    }

    pub fn get_focus_offset(&self) -> usize {
        self.0.focus_offset() as usize
    }

    pub fn get_anchor_offset(&self) -> usize {
        self.0.anchor_offset() as usize
    }
}

// Wrapper for a DOM Document, allowing modification of the body and
// the ability to create Elements (as ElementHandles).
pub struct DocumentHandle<BodyType: ElementComponent<web_sys::HtmlBodyElement>> {
    document: web_sys::Document,
    body: Option<BodyType>,
}

impl<BodyType: ElementComponent<web_sys::HtmlBodyElement>> Default for DocumentHandle<BodyType> {
    fn default() -> Self {
        Self {
            document: web_sys::window().unwrap().document().unwrap(),
            body: None,
        }
    }
}

#[derive(Clone)]
pub struct ElementFactory(web_sys::Document);

impl<BodyType: ElementComponent<web_sys::HtmlBodyElement>> DocumentHandle<BodyType> {
    pub fn body(&self) -> Option<&BodyType> {
        self.body.as_ref()
    }

    pub fn body_mut(&mut self) -> Option<&mut BodyType> {
        self.body.as_mut()
    }

    pub fn set_body(&mut self, body: BodyType) {
        self.body = Some(body);
        self.document
            .set_body(Some(self.body.as_ref().unwrap().element().0));
    }

    pub fn element_factory(&self) -> ElementFactory {
        ElementFactory(self.document.clone())
    }

    pub fn get_selection(&self) -> Option<SelectionHandle> {
        self.document
            .get_selection()
            .expect("Get Document Selection")
            .map(|dom_selection| SelectionHandle(dom_selection))
    }

    pub fn audit(&self) {
        match (&self.body, self.document.body()) {
            (Some(body), Some(dom_body)) => {
                assert!(dom_body.is_same_node(Some(body.node().0)));
                body.audit();
            }
            (Some(_), None) => panic!("missing body"),
            (None, Some(_)) => panic!("unexpected body"),
            (None, None) => (),
        }
    }
}

impl ElementFactory {
    fn create_element<T: JsCast>(&self, t: &str) -> T {
        self.0
            .create_element(t)
            .unwrap()
            .dyn_into::<T>()
            .unwrap_or_else(|_| panic!("expecting {t} element"))
    }

    pub fn div(&self) -> ElementHandle<web_sys::HtmlDivElement> {
        ElementHandle::new(self.create_element("div"))
    }

    pub fn span(&self) -> ElementHandle<web_sys::HtmlSpanElement> {
        ElementHandle::new(self.create_element("span"))
    }

    pub fn p(&self) -> ElementHandle<web_sys::HtmlParagraphElement> {
        ElementHandle::new(self.create_element("p"))
    }

    pub fn br(&self) -> ElementHandle<web_sys::HtmlBrElement> {
        ElementHandle::new(self.create_element("br"))
    }

    pub fn body(&self) -> ElementHandle<web_sys::HtmlBodyElement> {
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
pub trait ElementComponent<T: AnyElement>: Component {
    fn element<'a>(&'a self) -> ElementRef<'a, T>;
}
