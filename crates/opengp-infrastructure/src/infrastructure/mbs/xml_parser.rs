use std::collections::HashMap;
use std::io::BufRead;

use chrono::Utc;
use opengp_domain::domain::billing::MbsItem;
use quick_xml::events::Event;
use quick_xml::Reader;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MbsXmlParseError {
    #[error("XML parsing failed: {0}")]
    Xml(#[from] quick_xml::Error),
    #[error("Invalid MBS XML data: {0}")]
    InvalidData(String),
}

pub fn parse_mbs_xml_reader<R: BufRead>(reader: R) -> Result<Vec<MbsItem>, MbsXmlParseError> {
    let mut xml_reader = Reader::from_reader(reader);
    xml_reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut items = Vec::new();
    let mut in_data = false;
    let mut current_field: Option<String> = None;
    let mut current_data: HashMap<String, String> = HashMap::new();

    loop {
        match xml_reader.read_event_into(&mut buf)? {
            Event::Start(e) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == "Data" {
                    in_data = true;
                    current_field = None;
                    current_data.clear();
                } else if in_data {
                    current_field = Some(name);
                }
            }
            Event::Text(e) => {
                if in_data {
                    if let Some(field) = current_field.as_ref() {
                        let text = String::from_utf8_lossy(e.as_ref()).to_string();
                        current_data
                            .entry(field.clone())
                            .and_modify(|existing| existing.push_str(&text))
                            .or_insert(text);
                    }
                }
            }
            Event::CData(e) => {
                if in_data {
                    if let Some(field) = current_field.as_ref() {
                        let text = String::from_utf8_lossy(e.as_ref()).to_string();
                        current_data
                            .entry(field.clone())
                            .and_modify(|existing| existing.push_str(&text))
                            .or_insert(text);
                    }
                }
            }
            Event::End(e) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if name == "Data" {
                    in_data = false;
                    current_field = None;
                    items.push(map_data_to_item(&current_data)?);
                } else if in_data {
                    current_field = None;
                }
            }
            Event::Eof => break,
            _ => {}
        }

        buf.clear();
    }

    Ok(items)
}

fn map_data_to_item(data: &HashMap<String, String>) -> Result<MbsItem, MbsXmlParseError> {
    let item_num = required_i32(data, "ItemNum")?;

    Ok(MbsItem {
        item_num,
        sub_item_num: optional_i32(data, "SubItemNum")?,
        item_start_date: optional_string(data, "ItemStartDate"),
        item_end_date: optional_string(data, "ItemEndDate"),
        category: optional_string(data, "Category"),
        group_code: optional_string(data, "Group"),
        sub_group: optional_string(data, "SubGroup"),
        sub_heading: optional_string(data, "SubHeading"),
        item_type: optional_string(data, "ItemType"),
        fee_type: optional_string(data, "FeeType"),
        provider_type: optional_string(data, "ProviderType"),
        schedule_fee: optional_f64(data, "ScheduleFee")?,
        benefit_75: optional_f64(data, "Benefit75")?,
        benefit_85: optional_f64(data, "Benefit85")?,
        benefit_100: optional_f64(data, "Benefit100")?,
        derived_fee: optional_string(data, "DerivedFee"),
        description: optional_string(data, "Description"),
        description_start_date: optional_string(data, "DescriptionStartDate"),
        emsn_cap: optional_string(data, "EMSNCap"),
        emsn_maximum_cap: optional_f64(data, "EMSNMaximumCap")?,
        emsn_percentage_cap: optional_f64(data, "EMSNPercentageCap")?,
        is_gst_free: true,
        is_active: true,
        imported_at: Utc::now().to_rfc3339(),
    })
}

fn optional_string(data: &HashMap<String, String>, key: &str) -> Option<String> {
    data.get(key)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn required_i32(data: &HashMap<String, String>, key: &str) -> Result<i32, MbsXmlParseError> {
    let value = data
        .get(key)
        .map(|v| v.trim())
        .filter(|v| !v.is_empty())
        .ok_or_else(|| MbsXmlParseError::InvalidData(format!("Missing required field: {key}")))?;

    value.parse::<i32>().map_err(|err| {
        MbsXmlParseError::InvalidData(format!("Invalid integer for {key}: {value} ({err})"))
    })
}

fn optional_i32(
    data: &HashMap<String, String>,
    key: &str,
) -> Result<Option<i32>, MbsXmlParseError> {
    match data.get(key).map(|v| v.trim()).filter(|v| !v.is_empty()) {
        Some(value) => value.parse::<i32>().map(Some).map_err(|err| {
            MbsXmlParseError::InvalidData(format!("Invalid integer for {key}: {value} ({err})"))
        }),
        None => Ok(None),
    }
}

fn optional_f64(
    data: &HashMap<String, String>,
    key: &str,
) -> Result<Option<f64>, MbsXmlParseError> {
    match data.get(key).map(|v| v.trim()).filter(|v| !v.is_empty()) {
        Some(value) => value.parse::<f64>().map(Some).map_err(|err| {
            MbsXmlParseError::InvalidData(format!("Invalid decimal for {key}: {value} ({err})"))
        }),
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::parse_mbs_xml_reader;

    #[test]
    fn parses_single_data_item() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<MBS_XML>
  <Data>
    <ItemNum>23</ItemNum>
    <SubItemNum></SubItemNum>
    <ItemStartDate>01.12.1989</ItemStartDate>
    <ItemEndDate></ItemEndDate>
    <Category>1</Category>
    <Group>A1</Group>
    <SubGroup></SubGroup>
    <SubHeading>2</SubHeading>
    <ItemType>S</ItemType>
    <FeeType>N</FeeType>
    <ProviderType></ProviderType>
    <ScheduleFee>43.90</ScheduleFee>
    <Benefit75></Benefit75>
    <Benefit85></Benefit85>
    <Benefit100>43.90</Benefit100>
    <DerivedFee></DerivedFee>
    <Description>Example item</Description>
    <DescriptionStartDate>01.11.2023</DescriptionStartDate>
    <EMSNCap>P</EMSNCap>
    <EMSNMaximumCap>500.00</EMSNMaximumCap>
    <EMSNPercentageCap>300.00</EMSNPercentageCap>
  </Data>
</MBS_XML>"#;

        let items =
            parse_mbs_xml_reader(Cursor::new(xml.as_bytes())).expect("parser should succeed");

        assert_eq!(items.len(), 1);
        let item = &items[0];
        assert_eq!(item.item_num, 23);
        assert_eq!(item.sub_item_num, None);
        assert_eq!(item.group_code.as_deref(), Some("A1"));
        assert_eq!(item.schedule_fee, Some(43.90));
        assert_eq!(item.benefit_75, None);
        assert_eq!(item.emsn_maximum_cap, Some(500.0));
    }
}
