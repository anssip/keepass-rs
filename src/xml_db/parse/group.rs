use crate::{
    db::{iconid::IconId, node::NodePtr, CustomData, Entry, Group, Times},
    rc_refcell_node,
    xml_db::parse::{bad_event, FromXml, IgnoreSubfield, SimpleTag, SimpleXmlEvent, XmlParseError},
};
use std::convert::TryFrom;
use uuid::Uuid;

impl FromXml for Group {
    type Parses = Self;

    fn from_xml<I: Iterator<Item = super::SimpleXmlEvent>>(
        iterator: &mut std::iter::Peekable<I>,
        inner_cipher: &mut dyn crate::crypt::ciphers::Cipher,
    ) -> Result<Self::Parses, super::XmlParseError> {
        let open_tag = iterator.next().ok_or(XmlParseError::Eof)?;
        if !matches!(open_tag, SimpleXmlEvent::Start(ref tag, _) if tag == "Group") {
            return Err(bad_event("Open Group tag", open_tag));
        }

        let mut out = Group::default();

        while let Some(event) = iterator.peek() {
            match event {
                SimpleXmlEvent::Start(name, _) => match &name[..] {
                    "UUID" => {
                        out.uuid = SimpleTag::<Uuid>::from_xml(iterator, inner_cipher)?.value;
                    }
                    "Name" => {
                        out.name = SimpleTag::<String>::from_xml(iterator, inner_cipher)?.value;
                    }
                    "Notes" => {
                        out.notes = SimpleTag::<Option<String>>::from_xml(iterator, inner_cipher)?.value;
                    }
                    "IconID" => {
                        out.icon_id = SimpleTag::<Option<usize>>::from_xml(iterator, inner_cipher)?
                            .value
                            .map(|v| IconId::try_from(v).unwrap_or(IconId::FOLDER));
                    }
                    "CustomIconUUID" => {
                        out.custom_icon_uuid = SimpleTag::<Option<Uuid>>::from_xml(iterator, inner_cipher)?.value;
                    }
                    "Times" => {
                        out.times = Times::from_xml(iterator, inner_cipher)?;
                    }
                    "IsExpanded" => {
                        out.is_expanded = SimpleTag::<bool>::from_xml(iterator, inner_cipher)?.value;
                    }
                    "DefaultAutoTypeSequence" => {
                        out.default_autotype_sequence = SimpleTag::<Option<String>>::from_xml(iterator, inner_cipher)?.value;
                    }
                    "EnableAutoType" => {
                        out.enable_autotype = SimpleTag::<Option<String>>::from_xml(iterator, inner_cipher)?.value;
                    }
                    "EnableSearching" => {
                        out.enable_searching = SimpleTag::<Option<String>>::from_xml(iterator, inner_cipher)?.value;
                    }
                    "LastTopVisibleEntry" => {
                        out.last_top_visible_entry = SimpleTag::<Option<Uuid>>::from_xml(iterator, inner_cipher)?.value;
                    }
                    "Entry" => {
                        let entry = rc_refcell_node!(Entry::from_xml(iterator, inner_cipher)?);
                        out.children.push(entry.into());
                    }
                    "Group" => {
                        let group = rc_refcell_node!(Group::from_xml(iterator, inner_cipher)?);
                        out.children.push(group.into());
                    }
                    "CustomData" => {
                        out.custom_data = CustomData::from_xml(iterator, inner_cipher)?;
                    }
                    _ => IgnoreSubfield::from_xml(iterator, inner_cipher)?,
                },
                SimpleXmlEvent::End(name) if name == "Group" => break,
                _ => return Err(bad_event("start tag or close Group", event.clone())),
            }
        }

        out.children.iter().for_each(|child| {
            child.borrow_mut().set_parent(Some(out.uuid));
        });

        // no need to check for the correct closing tag - checked by XmlReader
        let _close_tag = iterator.next().ok_or(XmlParseError::Eof)?;

        Ok(out)
    }
}

#[cfg(test)]
mod parse_group_test {

    use crate::{
        db::Group,
        xml_db::parse::{parse_test::parse_test_xml, XmlParseError},
    };

    use uuid::uuid;

    #[test]
    fn test_group() -> Result<(), XmlParseError> {
        let _value = parse_test_xml::<Group>("<Group></Group>")?;

        let value = parse_test_xml::<Group>("<Group><Notes>ASDF</Notes></Group>")?;
        assert_eq!(value.notes, Some("ASDF".to_string()));

        let value = parse_test_xml::<Group>("<Group><CustomIconUUID>oaKjpLGywcLR0tPU1dbX2A==</CustomIconUUID></Group>")?;
        assert_eq!(value.custom_icon_uuid, Some(uuid!("a1a2a3a4b1b2c1c2d1d2d3d4d5d6d7d8")));

        let value = parse_test_xml::<Group>("");
        assert!(matches!(value, Err(XmlParseError::BadEvent { .. })));

        let value = parse_test_xml::<Group>("<TestTag>SomeData</TestTag>");
        assert!(matches!(value, Err(XmlParseError::BadEvent { .. })));

        let value = parse_test_xml::<Group>("<Group></TestTag>");
        assert!(matches!(value, Err(XmlParseError::BadEvent { .. })));

        let value = parse_test_xml::<Group>("<Group>No-Characters-Allowed</Group>");
        assert!(matches!(value, Err(XmlParseError::BadEvent { .. })));

        let _value = parse_test_xml::<Group>("<Group><UnkownChildTag/></Group>")?;

        Ok(())
    }
}
