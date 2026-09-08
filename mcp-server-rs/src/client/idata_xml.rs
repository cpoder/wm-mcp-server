//! Decoder for the XML produced by IS's `IDataXMLCoder` (`Accept: text/xml`).
//!
//! Why this exists: the JSON encoder IS uses for `/invoke` answers can only
//! render IData, strings and primitives. A flow service's `flow.nodes` holds
//! `FlowElement` objects, which the JSON encoder flattens into a string such
//! as `"[INVOKE]"` -- the step tree is lost. The XML encoder walks
//! `ValuesCodable` objects, so the same `wm.server.ns:getNode` call answered
//! as XML carries the complete tree (`<record>` / `<list name="nodes">`).
//! This module turns that XML back into `serde_json::Value` with the same
//! shape the JSON encoder would have produced, had it been able to.
//!
//! Element vocabulary (all observed on IS 12.1):
//! `<Values>` root, `<record name javaclass>`, `<value name>text</value>`,
//! `<null name/>`, `<number name type="Integer">`, `<Boolean name>`,
//! `<list name>` (array of records), `<array name type="value|record"
//! depth="1">` (children are `<value>`/`<record>` without names; depth 2
//! nests `<array>` elements), `<Date>`. Unknown elements degrade to a string
//! (text only) or an object (children).

use quick_xml::events::{BytesStart, Event};
use quick_xml::{Reader, XmlVersion};
use serde_json::{Map, Value};

const XML_1_0: XmlVersion = XmlVersion::Implicit1_0;

/// Resolve an entity or character reference (`lt`, `#10`, `#x0A`, ...) the
/// way an XML parser would; unknown names are kept verbatim as `&name;`.
fn resolve_reference(name: &str) -> String {
    match name {
        "lt" => "<".into(),
        "gt" => ">".into(),
        "amp" => "&".into(),
        "quot" => "\"".into(),
        "apos" => "'".into(),
        _ => {
            let code = name
                .strip_prefix("#x")
                .or_else(|| name.strip_prefix("#X"))
                .and_then(|h| u32::from_str_radix(h, 16).ok())
                .or_else(|| name.strip_prefix('#').and_then(|d| d.parse::<u32>().ok()));
            match code.and_then(char::from_u32) {
                Some(c) => c.to_string(),
                None => format!("&{name};"),
            }
        }
    }
}

#[derive(Debug)]
struct Frame {
    name: Option<String>,
    kind: String,
    number_type: Option<String>,
    map: Map<String, Value>,
    items: Vec<Value>,
    text: String,
    /// `true` once a child element has been attached (an element with
    /// children is a container even if its kind is unknown).
    has_children: bool,
}

impl Frame {
    fn new(e: &BytesStart<'_>) -> Result<Self, String> {
        let kind = e.name().as_ref().to_string();
        let mut name = None;
        let mut number_type = None;
        for attr in e.attributes() {
            let attr = attr.map_err(|e| format!("bad XML attribute: {e}"))?;
            let value = attr
                .normalized_value(XML_1_0)
                .map_err(|e| format!("bad XML attribute value: {e}"))?
                .into_owned();
            match attr.key.as_ref() {
                "name" => name = Some(value),
                "type" if kind == "number" => number_type = Some(value),
                _ => {}
            }
        }
        Ok(Self {
            name,
            kind,
            number_type,
            map: Map::new(),
            items: Vec::new(),
            text: String::new(),
            has_children: false,
        })
    }

    fn is_container(&self) -> bool {
        matches!(
            self.kind.as_str(),
            "Values" | "record" | "list" | "array" | "idatacodable"
        ) || self.has_children
    }

    fn is_array(&self) -> bool {
        matches!(self.kind.as_str(), "list" | "array")
    }

    fn attach(&mut self, name: Option<String>, value: Value) {
        self.has_children = true;
        if self.is_array() {
            self.items.push(value);
        } else {
            // Nameless children inside a record are unexpected; keep them
            // under a synthetic key rather than dropping them silently.
            let key = name.unwrap_or_else(|| format!("_{}", self.map.len()));
            self.map.insert(key, value);
        }
    }

    fn finish(self) -> (Option<String>, Value) {
        let value = match self.kind.as_str() {
            "null" => Value::Null,
            "list" | "array" => Value::Array(self.items),
            "Values" | "record" | "idatacodable" => Value::Object(self.map),
            "number" => parse_number(&self.text, self.number_type.as_deref()),
            "Boolean" => match self.text.trim() {
                "true" => Value::Bool(true),
                "false" => Value::Bool(false),
                other => Value::String(other.to_string()),
            },
            "value" | "Date" => Value::String(self.text),
            _ if self.has_children => Value::Object(self.map),
            _ => Value::String(self.text),
        };
        (self.name, value)
    }
}

fn parse_number(text: &str, number_type: Option<&str>) -> Value {
    let t = text.trim();
    let integral = matches!(
        number_type,
        Some("Integer") | Some("Long") | Some("Short") | Some("Byte") | Some("BigInteger")
    );
    if integral && let Ok(i) = t.parse::<i64>() {
        return Value::from(i);
    }
    if let Ok(f) = t.parse::<f64>()
        && let Some(n) = serde_json::Number::from_f64(f)
    {
        return Value::Number(n);
    }
    Value::String(t.to_string())
}

/// Decode an `IDataXMLCoder` document into JSON. The root `<Values>` element
/// becomes the top-level object.
pub fn idata_xml_to_json(xml: &str) -> Result<Value, String> {
    let mut reader = Reader::from_str(xml);
    let mut stack: Vec<Frame> = Vec::new();
    let mut root: Option<Value> = None;
    loop {
        let event = reader.read_event().map_err(|e| {
            format!(
                "invalid IData XML at byte {}: {e}",
                reader.buffer_position()
            )
        })?;
        match event {
            Event::Start(e) => stack.push(Frame::new(&e)?),
            Event::Empty(e) => {
                let (name, value) = Frame::new(&e)?.finish();
                match stack.last_mut() {
                    Some(parent) => parent.attach(name, value),
                    None => root = Some(value),
                }
            }
            Event::Text(t) => {
                if let Some(frame) = stack.last_mut()
                    && !frame.is_container()
                {
                    frame.text.push_str(&t.xml_content(XML_1_0));
                }
            }
            // quick-xml >= 0.38 reports `&lt;` / `&#10;` as separate events
            // instead of unescaping text itself.
            Event::GeneralRef(r) => {
                if let Some(frame) = stack.last_mut()
                    && !frame.is_container()
                {
                    frame
                        .text
                        .push_str(&resolve_reference(&r.xml_content(XML_1_0)));
                }
            }
            Event::CData(c) => {
                if let Some(frame) = stack.last_mut() {
                    frame.text.push_str(&c.into_inner());
                }
            }
            Event::End(_) => {
                let frame = stack.pop().ok_or_else(|| {
                    "unbalanced IData XML: closing tag without opener".to_string()
                })?;
                let (name, value) = frame.finish();
                match stack.last_mut() {
                    Some(parent) => parent.attach(name, value),
                    None => root = Some(value),
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }
    if !stack.is_empty() {
        return Err("unbalanced IData XML: document ended inside an element".to_string());
    }
    root.ok_or_else(|| "empty IData XML document".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const GET_NODE_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>

<Values version="2.0">
  <value name="name">mcpfeedbacktest:greetShell</value>
  <number name="LOCK_STATUS" type="Integer">3</number>
  <record name="node" javaclass="com.wm.util.Values">
    <value name="node_type">service</value>
    <null name="node_comment"/>
    <Boolean name="modifiable">true</Boolean>
    <record name="svc_sig" javaclass="com.wm.util.Values">
      <record name="sig_in" javaclass="com.wm.util.Values">
        <array name="rec_fields" type="record" depth="1">
          <record javaclass="com.wm.util.Values">
            <value name="field_name">name</value>
            <value name="field_type">string</value>
          </record>
        </array>
      </record>
    </record>
    <array name="where.inputField" type="value" depth="1">
      <value>id</value>
      <value>customer</value>
    </array>
    <record name="flow" javaclass="com.wm.util.Values">
      <value name="type">ROOT</value>
      <list name="nodes">
        <record javaclass="com.wm.util.Values">
          <value name="type">INVOKE</value>
          <value name="service">pub.string:concat</value>
          <list name="nodes">
            <record javaclass="com.wm.util.Values">
              <value name="type">MAP</value>
              <value name="mode">INPUT</value>
              <list name="nodes">
                <record javaclass="com.wm.util.Values">
                  <value name="type">MAPSET</value>
                  <value name="field">/inString1;1;0</value>
                  <value name="data">

&lt;Values version="2.0"&gt;
  &lt;value name="xml"&gt;Hello, &lt;/value&gt;
&lt;/Values&gt;
</value>
                </record>
                <record javaclass="com.wm.util.Values">
                  <value name="type">MAPCOPY</value>
                  <value name="from">/name;1;0</value>
                  <value name="to">/inString2;1;0</value>
                  <null name="condition"/>
                </record>
              </list>
            </record>
          </list>
        </record>
      </list>
    </record>
  </record>
</Values>
"#;

    #[test]
    fn decodes_scalars_records_lists_and_arrays() {
        let v = idata_xml_to_json(GET_NODE_XML).unwrap();
        assert_eq!(v["name"], json!("mcpfeedbacktest:greetShell"));
        assert_eq!(v["LOCK_STATUS"], json!(3));
        assert_eq!(v["node"]["node_type"], json!("service"));
        assert!(v["node"]["node_comment"].is_null());
        assert_eq!(v["node"]["modifiable"], json!(true));
        assert_eq!(
            v["node"]["svc_sig"]["sig_in"]["rec_fields"][0]["field_name"],
            json!("name")
        );
        assert_eq!(v["node"]["where.inputField"], json!(["id", "customer"]));
    }

    #[test]
    fn flow_tree_keeps_nested_steps_the_json_encoder_flattens() {
        let v = idata_xml_to_json(GET_NODE_XML).unwrap();
        let flow = &v["node"]["flow"];
        assert_eq!(flow["type"], json!("ROOT"));
        let invoke = &flow["nodes"][0];
        assert_eq!(invoke["type"], json!("INVOKE"));
        let map = &invoke["nodes"][0];
        assert_eq!(map["mode"], json!("INPUT"));
        assert_eq!(map["nodes"][0]["type"], json!("MAPSET"));
        assert_eq!(map["nodes"][1]["to"], json!("/inString2;1;0"));
        assert!(map["nodes"][1]["condition"].is_null());
    }

    #[test]
    fn entity_escaped_text_is_unescaped_and_whitespace_preserved() {
        let v = idata_xml_to_json(GET_NODE_XML).unwrap();
        let data = v["node"]["flow"]["nodes"][0]["nodes"][0]["nodes"][0]["data"]
            .as_str()
            .unwrap();
        assert!(data.starts_with("\n\n<Values version=\"2.0\">"));
        assert!(data.contains("<value name=\"xml\">Hello, </value>"));
    }

    #[test]
    fn character_and_entity_references_are_resolved() {
        let xml = r#"<Values version="2.0"><value name="v">a &amp; b &lt;c&gt; &#10;&#x41;&quot;&apos;&bogus;</value></Values>"#;
        let v = idata_xml_to_json(xml).unwrap();
        assert_eq!(v["v"], json!("a & b <c> \nA\"'&bogus;"));
    }

    #[test]
    fn missing_node_decodes_to_null() {
        let xml = r#"<Values version="2.0"><value name="name">a:b</value><number name="LOCK_STATUS" type="Integer">2</number><null name="node"/></Values>"#;
        let v = idata_xml_to_json(xml).unwrap();
        assert!(v["node"].is_null());
        assert_eq!(v["LOCK_STATUS"], json!(2));
    }

    #[test]
    fn empty_containers_and_unknown_elements_degrade_gracefully() {
        let xml = r#"<Values version="2.0"><array name="empty" type="value" depth="1"/><record name="r"/><Date name="d" type="java.util.Date">Tue Sep 08</Date><Other name="o">x</Other><number name="f" type="Double">1.5</number><number name="bad" type="Integer">abc</number></Values>"#;
        let v = idata_xml_to_json(xml).unwrap();
        assert_eq!(v["empty"], json!([]));
        assert_eq!(v["r"], json!({}));
        assert_eq!(v["d"], json!("Tue Sep 08"));
        assert_eq!(v["o"], json!("x"));
        assert_eq!(v["f"], json!(1.5));
        assert_eq!(v["bad"], json!("abc"));
    }

    #[test]
    fn malformed_xml_is_an_error_not_a_panic() {
        assert!(idata_xml_to_json("<Values><record name=\"x\"></Values>").is_err());
        assert!(idata_xml_to_json("").is_err());
    }
}
