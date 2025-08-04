use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use wasm_bindgen::JsCast;

use crate::dom_br::DomBr;
use crate::dom_struct::*;
use crate::dom_text::*;
use crate::dom_vec::*;
use crate::web_support::Component;
use crate::web_support::ElementFactory;
use crate::web_support::ElementReader;
use crate::web_support::NodeReader;
use crate::web_support::SelectionHandle;

type DomEditLine = DomStruct<(DomText, (DomBr, ())), web_sys::HtmlDivElement>;

/// LogicSelection is the selection area in logical model's view.
/// Usually it is dom selection restricted to editable area.
#[derive(Debug, Clone, Copy)]
struct LogicSelection {
    pub anchor: (usize, usize), // #Ln, #Col
    pub focus: (usize, usize),
}

impl LogicSelection {
    pub fn new_cursor(r: usize, c: usize) -> LogicSelection {
        LogicSelection {
            anchor: (r, c),
            focus: (r, c),
        }
    }

    pub fn new(anchor: (usize, usize), focus: (usize, usize)) -> LogicSelection {
        LogicSelection { anchor, focus }
    }

    pub fn is_cursor(&self) -> bool {
        self.anchor == self.focus
    }

    pub fn to_area(self) -> std::ops::Range<(usize, usize)> {
        std::cmp::min(self.anchor, self.focus)..std::cmp::max(self.anchor, self.focus)
    }
}
struct EditLine {
    dom_editline: DomEditLine,
    id: usize,
}

impl EditLine {
    pub const EDITLINE_ID_ATTR: &str = "data-editline-id";
    fn new(factory: &ElementFactory, id: usize) -> Self {
        let mut dom_editline = DomEditLine::new(
            (
                DomText::new(&format!("Hello World, id {}", id)),
                (DomBr::new((), factory.br()), ()),
            ),
            factory.div(),
        );
        dom_editline.set_attribute(Self::EDITLINE_ID_ATTR, &id.to_string());
        EditLine { dom_editline, id }
    }

    fn find_id_from_node<T: AsRef<web_sys::Node> + JsCast>(node: NodeReader<T>) -> Option<usize> {
        let mut elem = match ElementReader::<web_sys::Element>::try_from(node) {
            Ok(elem) => elem,
            Err(node) => node.parent_element().expect("Get a parent element parent"),
        };

        loop {
            if let Some(id) = elem.get_attr(Self::EDITLINE_ID_ATTR) {
                return Some(id.parse::<usize>().expect("parse id"));
            }
            elem = if let Some(parent_elem) = elem.parent_element() {
                parent_elem
            } else {
                return None;
            }
        }
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
struct _Editor {
    dom_editor: DomEditor,
    dom_selection: SelectionHandle,
    factory: ElementFactory,
    next_id: usize,
    id_map: HashMap<usize, usize>,
}

impl Component for _Editor {
    delegate::delegate! {
        to self.dom_editor {
            fn audit(&self);
            fn node<'a>(&'a self) -> crate::web_support::NodeRef<'a>;
        }
    }
}

impl _Editor {
    fn new(factory: ElementFactory, dom_selection: SelectionHandle) -> _Editor {
        let mut div = factory.div();
        div.set_attribute("class", "textentry");
        div.set_attribute("contenteditable", "true");
        div.set_attribute("spellcheck", "false");
        let dom_editor = DomVec::new(div);
        _Editor {
            dom_editor,
            dom_selection,
            factory,
            next_id: 0,
            id_map: HashMap::new(),
        }
    }

    fn insert(&mut self, index: usize) {
        self.dom_editor
            .insert(index, EditLine::new(&self.factory, self.next_id));
        self.id_map.values_mut().for_each(|line_no| {
            if *line_no >= index {
                *line_no += 1;
            }
        });
        self.id_map.insert(self.next_id, index);
        self.next_id += 1;
    }

    fn get_logic_seletion(&self) -> Option<LogicSelection> {
        let anchor = self.dom_selection.get_anchor_node()?;
        let focus = self.dom_selection.get_focus_node()?;

        let anchor_id = EditLine::find_id_from_node(anchor);
        let focus_id = EditLine::find_id_from_node(focus);

        match (anchor_id, focus_id) {
            (Some(anchor_id), Some(focus_id)) => {
                let anchor_idx: usize = *self.id_map.get(&anchor_id).expect("can't find line");
                let focus_idx: usize = *self.id_map.get(&focus_id).expect("can't find line");
                Some(LogicSelection::new(
                    (anchor_idx, self.dom_selection.get_anchor_offset()),
                    (focus_idx, self.dom_selection.get_focus_offset()),
                ))
            }
            (Some(anchor_id), None) => {
                let anchor_idx: usize = *self.id_map.get(&anchor_id).expect("can't find line");
                Some(LogicSelection::new(
                    (anchor_idx, self.dom_selection.get_anchor_offset()),
                    (
                        self.dom_editor.len() - 1,
                        self.dom_editor
                            .last()
                            .expect("Empty lines")
                            .dom_editline
                            .get()
                            .0
                            .get_data()
                            .chars()
                            .count(),
                    ),
                ))
            }
            (None, Some(focus_id)) => {
                let focus_idx: usize = *self.id_map.get(&focus_id).expect("can't find line");
                Some(LogicSelection::new(
                    (0, 0),
                    (focus_idx, self.dom_selection.get_focus_offset()),
                ))
            }
            (None, None) => None,
        }
    }
}

#[derive(Clone)]
pub struct Editor(Rc<RefCell<_Editor>>);
impl Editor {
    fn initialize(&mut self) {
        let editor_rc = self.clone();
        self.0.borrow_mut().dom_editor.set_onbeforeinput(move |ev| {
            ev.prevent_default();
            web_sys::console::log_1(
                &format!(
                    "input event {}, data {:?}, {:?}",
                    ev.input_type(),
                    ev.data(),
                    editor_rc.0.borrow().get_logic_seletion()
                )
                .into(),
            );
        });

        self.0.borrow_mut().insert(0);
        self.0.borrow_mut().insert(1);
        self.0.borrow_mut().insert(1);
    }

    pub fn new(factory: ElementFactory, dom_selection: SelectionHandle) -> Self {
        let mut editor = Editor(Rc::new(RefCell::new(_Editor::new(factory, dom_selection))));
        editor.initialize();
        editor
    }
}

impl<Rest: Sequence> Sequence for (Editor, Rest) {
    const LEN: usize = Rest::LEN + 1;
    fn install(&self, nodes: &mut crate::web_support::ArrayHandle, index: usize) {
        assert_eq!(index + Self::LEN, nodes.length());
        nodes.set(index, self.0.0.borrow().node());
        self.1.install(nodes, index + 1);
    }

    fn audit(&self, node_list: &crate::web_support::NodeListHandle, index: usize) {
        assert_eq!(index + Self::LEN, node_list.length());
        node_list.audit_node(index, self.0.0.borrow().node());
        self.0.0.borrow().audit();
        self.1.audit(node_list, index + 1);
    }
}
