//! parser utilities

use crate::{
    error::Error,
    parser::phrases::*,
    xhtml::{Element, XhtmlBuilder, XhtmlNode},
};
use ego_tree::NodeRef;
use scraper::{ElementRef, Node};

/// Look for copyright indicators to decide whether to filter this text.
pub fn filter(text: &str) -> bool {
    if text.len() > COPYRIGHT_TRIGGER_LENGTH {
        return false;
    }
    let text = text.to_lowercase();
    let mut total_weight = 0;
    let mut count = 0;
    for (trigger, weight) in COPYRIGHT_TRIGGERS {
        if text.contains(trigger) {
            total_weight = total_weight + weight;
            count = count + 1;
        }
    }
    total_weight >= COPYRIGHT_TRIGGER_THRESHOLD || count >= COPYRIGHT_TRIGGER_COUNT
}

/// Generic parse_content function. Should be good enough for most scenarios.
pub fn parse_content<'a>(title: &str, content: ElementRef<'a>) -> Result<String, Error> {
    let xhtml = XhtmlBuilder::with_header(title);
    let article = xhtml.article();
    parse_inner_content(&xhtml, article, content)?;
    Ok(xhtml.build())
}

fn parse_inner_content<'a>(
    xhtml: &XhtmlBuilder,
    parent: XhtmlNode,
    content: ElementRef<'a>,
) -> Result<(), Error> {
    log::debug!("found: {:?}", content);
    let el = content.value();
    let tag = el.name();
    let node = match tag {
        "br" => {
            let linebreak = append_element(xhtml, parent, Element::P)?;
            append_text(xhtml, linebreak, ".".to_string())?;
            parent
        }
        "table" => append_element(xhtml, parent, Element::Table)?,
        "tr" => append_element(xhtml, parent, Element::Tr)?,
        "th" => append_element(xhtml, parent, Element::Th)?,
        "td" => append_element(xhtml, parent, Element::Td)?,
        "p" => append_element(xhtml, parent, Element::P)?,
        "i" | "em" => append_element(xhtml, parent, Element::Em)?,
        "b" | "strong" => append_element(xhtml, parent, Element::Strong)?,
        "hr" => {
            let attrs = el.attrs().collect::<Vec<_>>();
            append_element_with_attrs(xhtml, parent, Element::Hr, attrs.as_ref())?
        }
        "span" => match el.attr("style") {
            Some(style) => {
                // Filter out font-family and font-size. Too unpredictable.
                let mut stylelist = Vec::new();
                for st in style.split(";") {
                    if !st.contains("font-family") && !st.contains("font-size") {
                        stylelist.push(st.trim());
                    }
                }
                if stylelist.is_empty() {
                    parent
                } else {
                    append_element_with_attrs(
                        xhtml,
                        parent,
                        Element::Span,
                        &[("style", &stylelist.join("; "))],
                    )?
                }
            }
            None => parent,
        },
        // else, inherit the parent
        _ => parent,
    };
    for child in content.children() {
        parse_node(xhtml, node, child)?;
    }
    Ok(())
}

fn parse_node<'a>(
    xhtml: &XhtmlBuilder,
    parent: XhtmlNode,
    content: NodeRef<'a, Node>,
) -> Result<(), Error> {
    match content.value() {
        Node::Text(text) => {
            let text = text.to_string();
            if !text.trim().is_empty() {
                if filter(&text) {
                    log::debug!("filtering: {}", text);
                } else {
                    let len = if text.len() > 100 { 100 } else { text.len() };
                    log::debug!("appending {:?} to {:?}", &text[0..len], parent);
                    append_text(xhtml, parent, text)?;
                }
            }
            Ok(())
        }
        Node::Element(_) => parse_inner_content(
            xhtml,
            parent,
            ElementRef::wrap(content).ok_or(Error::html("should be element", true))?,
        ),
        _ => Ok(()),
    }
}

/// I want to require <p></p> for stylized tags and spans. It presents itself better in ePub
/// formats.
fn append_element_with_attrs<N, V>(
    xhtml: &XhtmlBuilder,
    parent: XhtmlNode,
    element: Element,
    attrs: &[(N, V)],
) -> Result<XhtmlNode, Error>
where
    N: AsRef<str>,
    V: AsRef<str>,
{
    let node = match element {
        Element::Em | Element::Strong => match parent.element() {
            Element::P | Element::Span | Element::Em | Element::Strong => {
                xhtml.append_element_with_attrs(parent, element, attrs)?
            }
            _ => {
                let p = xhtml.append_element(parent, Element::P)?;
                log::debug!("created: {:?}", p);
                xhtml.append_element_with_attrs(p, element, attrs)?
            }
        },
        Element::Span => match parent.element() {
            Element::P | Element::Span | Element::Em | Element::Strong => {
                xhtml.append_element_with_attrs(parent, element, attrs)?
            }
            _ => {
                let p = xhtml.append_element(parent, Element::P)?;
                log::debug!("created: {:?}", p);
                xhtml.append_element_with_attrs(p, element, attrs)?
            }
        },
        _ => xhtml.append_element_with_attrs(parent, element, attrs)?,
    };
    log::debug!("created: {:?}", node);
    Ok(node)
}

fn append_element(
    xhtml: &XhtmlBuilder,
    parent: XhtmlNode,
    element: Element,
) -> Result<XhtmlNode, Error> {
    append_element_with_attrs::<&str, &str>(xhtml, parent, element, &[])
}

/// Similar to paragraph styles, only allow text under certain tags
fn append_text(xhtml: &XhtmlBuilder, parent: XhtmlNode, text: String) -> Result<(), Error> {
    match parent.element() {
        Element::P | Element::Span | Element::Em | Element::Strong | Element::Th | Element::Td => {
            Ok(xhtml.append_text(parent, text))
        }
        Element::Table | Element::Tr => Err(Error::html("no text in table or tr", true)),
        _ => {
            let p = xhtml.append_element(parent, Element::P)?;
            Ok(xhtml.append_text(p, text))
        }
    }
}

#[allow(unused)]
/// Breaks a paragraph but keeps its formatting
fn break_paragraph(xhtml: &XhtmlBuilder, node: XhtmlNode) -> Result<XhtmlNode, Error> {
    log::debug!("[br] old parent: {:?}", node);
    let mut stack = Vec::new();
    let mut node = node;
    loop {
        match node.element() {
            // Save these elements
            Element::P | Element::Span | Element::Em | Element::Strong => {
                log::debug!("[br] saving: <{:?}>", node.element());
                stack.push(node);
                node = node
                    .parent(xhtml)
                    .ok_or(Error::html("[br] missing parent", true))?;
            }
            _ => {
                log::debug!("[br] stopping at: <{:?}>", node.element());
                break;
            }
        }
    }
    while let Some(child) = stack.pop() {
        let attrs = child.attrs(xhtml);
        log::debug!(
            "[br] attaching <{:?}> to <{:?}>",
            child.element(),
            node.element()
        );
        node = append_element_with_attrs(xhtml, node, child.element(), attrs.as_slice())?;
    }
    log::debug!("[br] new parent: {:?}", node);
    Ok(node)
}
