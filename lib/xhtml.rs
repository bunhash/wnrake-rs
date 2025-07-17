//! minimal epub xhtml builder

use crate::error::Error;
use ego_tree::NodeId;
use html5ever::{
    tendril::StrTendril,
    tree_builder::{ElementFlags, NodeOrText, TreeSink},
    Attribute, LocalName, Namespace, QualName,
};
use scraper::{ElementRef, Html, HtmlTreeSink, Node, Selector};

static XHTML_NAMESPACE: &str = "http://www.w3.org/1999/xhtml";
static XHTML_BASE: &str = r#"
<?xml version="1.0" encoding="UTF-8" standalone="no"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops">
<head></head>
<body>
<section>
<header></header>
<article></article>
</section>
</body>
</html>
"#;

#[derive(Clone, Debug, Copy, PartialOrd, PartialEq, Ord, Eq)]
pub enum Element {
    Article,
    Body,
    Br,
    Em,
    H1,
    H2,
    Head,
    Header,
    Html,
    Img,
    P,
    Section,
    Span,
    Strong,
    Table,
    Td,
    Th,
    Title,
    Tr,
}

impl Element {
    fn from_tag(tag: &str) -> Option<Self> {
        match tag {
            "article" => Some(Element::Article),
            "body" => Some(Element::Body),
            "br" => Some(Element::Br),
            "i" | "em" => Some(Element::Em),
            "h1" => Some(Element::H1),
            "h2" => Some(Element::H2),
            "head" => Some(Element::Head),
            "header" => Some(Element::Header),
            "html" => Some(Element::Html),
            "img" => Some(Element::Img),
            "p" => Some(Element::P),
            "section" => Some(Element::Section),
            "span" => Some(Element::Span),
            "b" | "strong" => Some(Element::Strong),
            "table" => Some(Element::Table),
            "td" => Some(Element::Td),
            "th" => Some(Element::Th),
            "title" => Some(Element::Title),
            "tr" => Some(Element::Tr),
            _ => None,
        }
    }

    pub fn qualname(&self) -> QualName {
        QualName::new(
            None,
            Namespace::from(XHTML_NAMESPACE),
            match self {
                Element::Article => LocalName::from("article"),
                Element::Body => LocalName::from("body"),
                Element::Br => LocalName::from("br"),
                Element::Em => LocalName::from("em"),
                Element::H1 => LocalName::from("h1"),
                Element::H2 => LocalName::from("h2"),
                Element::Head => LocalName::from("head"),
                Element::Header => LocalName::from("header"),
                Element::Html => LocalName::from("html"),
                Element::Img => LocalName::from("img"),
                Element::P => LocalName::from("p"),
                Element::Section => LocalName::from("section"),
                Element::Span => LocalName::from("span"),
                Element::Strong => LocalName::from("strong"),
                Element::Table => LocalName::from("table"),
                Element::Td => LocalName::from("td"),
                Element::Th => LocalName::from("th"),
                Element::Title => LocalName::from("title"),
                Element::Tr => LocalName::from("tr"),
            },
        )
    }

    fn append_to(self, sink: &HtmlTreeSink, node: XhtmlNode) -> XhtmlNode {
        let element_id = sink.create_element(self.qualname(), Vec::new(), ElementFlags::default());
        sink.append(&node.id, NodeOrText::AppendNode(element_id));
        XhtmlNode::new(element_id, self, Some(node.id))
    }

    fn append_to_with_attrs<N, V>(
        self,
        sink: &HtmlTreeSink,
        node: XhtmlNode,
        attrs: &[(N, V)],
    ) -> XhtmlNode
    where
        N: AsRef<str>,
        V: AsRef<str>,
    {
        let mut attributes = Vec::new();
        for (name, value) in attrs {
            attributes.push(Attribute {
                name: QualName::new(None, Namespace::from(""), LocalName::from(name.as_ref())),
                value: StrTendril::from(value.as_ref()),
            });
        }
        let element_id = sink.create_element(self.qualname(), attributes, ElementFlags::default());
        sink.append(&node.id, NodeOrText::AppendNode(element_id));
        XhtmlNode::new(element_id, self, Some(node.id))
    }
}

#[derive(Clone, Debug, Copy, PartialOrd, PartialEq, Ord, Eq)]
pub struct XhtmlNode {
    id: NodeId,
    element: Element,
    parent: Option<NodeId>,
}

impl XhtmlNode {
    fn new(id: NodeId, element: Element, parent: Option<NodeId>) -> Self {
        XhtmlNode {
            id,
            element,
            parent,
        }
    }

    fn root(id: NodeId) -> Self {
        XhtmlNode {
            id,
            element: Element::Html,
            parent: None,
        }
    }

    pub fn element(&self) -> Element {
        self.element
    }

    pub fn parent(&self, xhtml: &XhtmlBuilder) -> Option<XhtmlNode> {
        xhtml.get_xhtml_node(self.parent?)
    }

    pub fn attrs<'a>(&self, xhtml: &'a XhtmlBuilder) -> Vec<(String, String)> {
        xhtml.get_attrs(self.id)
    }
}

#[derive(Debug)]
pub struct XhtmlBuilder {
    sink: HtmlTreeSink,
    header: XhtmlNode,
    article: XhtmlNode,
}

impl XhtmlBuilder {
    pub fn new(title: &str) -> Self {
        Self::new_internal(title, false)
    }

    pub fn with_header(title: &str) -> Self {
        Self::new_internal(title, true)
    }

    fn new_internal(title: &str, build_header: bool) -> Self {
        // Parse template and get head/body
        let document = Html::parse_document(XHTML_BASE);
        let root = XhtmlNode::root(document.root_element().id());
        let head = XhtmlNode::new(
            document
                .select(&Selector::parse("head").unwrap())
                .next()
                .expect("head should exist")
                .id(),
            Element::Head,
            Some(root.id),
        );
        let header = XhtmlNode::new(
            document
                .select(&Selector::parse("header").unwrap())
                .next()
                .expect("header should exist")
                .id(),
            Element::Header,
            Some(root.id),
        );
        let article = XhtmlNode::new(
            document
                .select(&Selector::parse("article").unwrap())
                .next()
                .expect("article should exist")
                .id(),
            Element::Article,
            Some(root.id),
        );

        // Create sink and add the title
        let sink = HtmlTreeSink::new(document);
        let title_node = Element::Title.append_to(&sink, head);
        sink.append(
            &title_node.id,
            NodeOrText::AppendText(StrTendril::from(title)),
        );

        if build_header {
            let h1_node = Element::H1.append_to(&sink, header);
            sink.append(&h1_node.id, NodeOrText::AppendText(StrTendril::from(title)));
        }

        // Return object
        XhtmlBuilder {
            sink,
            header,
            article,
        }
    }

    pub fn append_element(&self, node: XhtmlNode, element: Element) -> Result<XhtmlNode, Error> {
        self.append_element_with_attrs::<&str, &str>(node, element, &[])
    }

    pub fn append_element_with_attrs<N, V>(
        &self,
        node: XhtmlNode,
        element: Element,
        attrs: &[(N, V)],
    ) -> Result<XhtmlNode, Error>
    where
        N: AsRef<str>,
        V: AsRef<str>,
    {
        match element {
            Element::Article
            | Element::Body
            | Element::Head
            | Element::Header
            | Element::Section
            | Element::Title => Err(Error::html("restricted xhtml tag", true)),
            _ => Ok(element.append_to_with_attrs(&self.sink, node, attrs)),
        }
    }

    pub fn append_image(&self, node: XhtmlNode, source: &str) {
        Element::Img.append_to_with_attrs(&self.sink, node, &[("src", source)]);
    }

    pub fn append_text(&self, node: XhtmlNode, text: String) {
        self.sink
            .append(&node.id, NodeOrText::AppendText(text.into()))
    }

    pub fn header(&self) -> XhtmlNode {
        self.header
    }

    pub fn article(&self) -> XhtmlNode {
        self.article
    }

    pub fn build(self) -> String {
        // purge empty paragraphs
        let mut ids_to_purge = Vec::new();
        {
            let html = self.sink.0.borrow();
            for p in html.select(&Selector::parse("p").unwrap()) {
                let text = p.text().collect::<Vec<_>>().join("").trim().to_string();
                if text.is_empty() {
                    ids_to_purge.push(p.id());
                }
            }
        }
        for id in ids_to_purge {
            self.sink.remove_from_parent(&id);
        }

        // build HTML
        self.sink.finish().html()
    }

    fn get_xhtml_node(&self, id: NodeId) -> Option<XhtmlNode> {
        let html = self.sink.0.borrow();
        let node_ref = html.tree.get(id)?;
        let id = node_ref.id();
        let element = Element::from_tag(ElementRef::wrap(node_ref)?.value().name())?;
        let parent = node_ref.parent().map(|n| n.id());
        Some(XhtmlNode::new(id, element, parent))
    }

    fn get_attrs(&self, id: NodeId) -> Vec<(String, String)> {
        let html = self.sink.0.borrow();
        let node = html.tree.get(id).map(|n| n.value());
        match node {
            Some(Node::Element(el)) => el
                .attrs()
                .map(|(n, v)| (n.into(), v.into()))
                .collect::<Vec<_>>(),
            _ => Vec::new(),
        }
    }
}
