use crate::web_support::Component;
use crate::dom_struct::*;
use crate::dom_text::*;
use crate::dom_vec::*;
use crate::dom_br::*;


type DomLineNumber = DomStruct<(DomText, ()), web_sys::HtmlSpanElement>;
type DomEditLine = DomStruct<(DomLineNumber, (DomText, (DomBr, ()))), web_sys::HtmlSpanElement>;

struct EditLine
{
    dom_editline: DomEditLine,
}

impl Component for EditLine {
    fn audit(&self) {
        self.dom_editline.audit();
    }
    fn node<'a>(&'a self) -> crate::web_support::NodeRef<'a> {
        self.dom_editline.node()
    }
}


type DomEditor = DomVec<EditLine, web_sys::HtmlDivElement>;
pub struct Editor
{
    dom_editor: DomEditor
}

impl Component for Editor
{
    fn audit(&self) {
        self.dom_editor.audit();
    }

    fn node<'a>(&'a self) -> crate::web_support::NodeRef<'a> {
        self.dom_editor.node()
    }
}