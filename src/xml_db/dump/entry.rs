use base64::{engine::general_purpose as base64_engine, Engine as _};
use xml::writer::{EventWriter, XmlEvent as WriterEvent};

use crate::{
    crypt::ciphers::Cipher,
    db::{AutoType, AutoTypeAssociation, Entry, History, Value},
    xml_db::dump::{DumpXml, SimpleTag},
};

fn escape_xml(input: &str) -> String {
    input
        .chars()
        .map(|c| match c {
            '<' => "&lt;".to_string(),
            '>' => "&gt;".to_string(),
            '&' => "&amp;".to_string(),
            '\'' => "&apos;".to_string(),
            '"' => "&quot;".to_string(),
            _ if c.is_control() && c != '\n' && c != '\r' && c != '\t' => {
                format!("&#x{:X};", c as u32)
            }
            _ => c.to_string(),
        })
        .collect()
}

impl DumpXml for Entry {
    fn dump_xml<E: std::io::Write>(&self, writer: &mut EventWriter<E>, inner_cipher: &mut dyn Cipher) -> Result<(), xml::writer::Error> {
        writer.write(WriterEvent::start_element("Entry"))?;

        SimpleTag("UUID", &self.uuid).dump_xml(writer, inner_cipher)?;

        SimpleTag("Tags", &escape_xml(&self.tags.join(";"))).dump_xml(writer, inner_cipher)?;

        for (field_name, field_value) in &self.fields {
            writer.write(WriterEvent::start_element("String"))?;

            SimpleTag("Key", &escape_xml(field_name)).dump_xml(writer, inner_cipher)?;
            field_value.dump_xml(writer, inner_cipher)?;

            writer.write(WriterEvent::end_element())?; // String
        }

        self.custom_data.dump_xml(writer, inner_cipher)?;

        if let Some(ref value) = self.autotype {
            value.dump_xml(writer, inner_cipher)?;
        }

        self.times.dump_xml(writer, inner_cipher)?;

        if let Some(value) = self.icon_id {
            SimpleTag("IconID", usize::from(value)).dump_xml(writer, inner_cipher)?;
        }

        if let Some(ref value) = self.custom_icon_uuid {
            SimpleTag("CustomIconUUID", value).dump_xml(writer, inner_cipher)?;
        }

        if let Some(ref value) = self.foreground_color {
            SimpleTag("ForegroundColor", &escape_xml(&value.to_string())).dump_xml(writer, inner_cipher)?;
        }

        if let Some(ref value) = self.background_color {
            SimpleTag("BackgroundColor", &escape_xml(&value.to_string())).dump_xml(writer, inner_cipher)?;
        }

        if let Some(ref value) = self.override_url {
            SimpleTag("OverrideURL", &escape_xml(value)).dump_xml(writer, inner_cipher)?;
        }

        if let Some(value) = self.quality_check {
            SimpleTag("QualityCheck", value).dump_xml(writer, inner_cipher)?;
        }

        if let Some(ref value) = self.history {
            value.dump_xml(writer, inner_cipher)?;
        }

        writer.write(WriterEvent::end_element())?; // Entry

        Ok(())
    }
}

impl DumpXml for Value {
    fn dump_xml<E: std::io::Write>(&self, writer: &mut EventWriter<E>, inner_cipher: &mut dyn Cipher) -> Result<(), xml::writer::Error> {
        match self {
            Value::Bytes(b) => SimpleTag("Value", std::str::from_utf8(b).expect("utf-8")).dump_xml(writer, inner_cipher),
            Value::Unprotected(s) => SimpleTag("Value", &escape_xml(s)).dump_xml(writer, inner_cipher),
            Value::Protected(p) => {
                writer.write(WriterEvent::start_element("Value").attr("Protected", "True"))?;

                let encrypted_value = inner_cipher.encrypt(p.unsecure()).expect("Encrypt with inner cipher");

                let protected_value = base64_engine::STANDARD.encode(encrypted_value);

                writer.write(WriterEvent::characters(&protected_value))?;

                writer.write(WriterEvent::end_element())?;
                Ok(())
            }
        }
    }
}

impl DumpXml for AutoType {
    fn dump_xml<E: std::io::Write>(&self, writer: &mut EventWriter<E>, inner_cipher: &mut dyn Cipher) -> Result<(), xml::writer::Error> {
        writer.write(WriterEvent::start_element("AutoType"))?;

        SimpleTag("Enabled", self.enabled).dump_xml(writer, inner_cipher)?;

        if let Some(ref value) = self.sequence {
            SimpleTag("DefaultSequence", &escape_xml(value)).dump_xml(writer, inner_cipher)?;
        }

        for assoc in &self.associations {
            assoc.dump_xml(writer, inner_cipher)?;
        }

        writer.write(WriterEvent::end_element())?;
        Ok(())
    }
}

impl DumpXml for AutoTypeAssociation {
    fn dump_xml<E: std::io::Write>(&self, writer: &mut EventWriter<E>, inner_cipher: &mut dyn Cipher) -> Result<(), xml::writer::Error> {
        writer.write(WriterEvent::start_element("Association"))?;

        if let Some(ref value) = self.window {
            SimpleTag("Window", &escape_xml(value)).dump_xml(writer, inner_cipher)?;
        }

        if let Some(ref value) = self.sequence {
            SimpleTag("KeystrokeSequence", &escape_xml(value)).dump_xml(writer, inner_cipher)?;
        }

        writer.write(WriterEvent::end_element())?;
        Ok(())
    }
}

impl DumpXml for History {
    fn dump_xml<E: std::io::Write>(&self, writer: &mut EventWriter<E>, inner_cipher: &mut dyn Cipher) -> Result<(), xml::writer::Error> {
        writer.write(WriterEvent::start_element("History"))?;

        for entry in &self.entries {
            entry.dump_xml(writer, inner_cipher)?;
        }

        writer.write(WriterEvent::end_element())?;
        Ok(())
    }
}
