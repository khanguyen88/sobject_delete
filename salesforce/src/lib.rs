pub mod salesforce {
    extern crate strum;
    extern crate strum_macros;

    use std::fs::File;
    use std::io::BufReader;
    use std::path::Path;
    use std::str::FromStr;

    use quick_xml::events::Event;
    use quick_xml::Reader;
    use strum_macros::EnumString;

    #[derive(Debug, PartialEq, Eq)]
    pub struct SObject {
        name: String,
        lookup_fields: Vec<LookupField>,
    }

    #[derive(Debug, PartialEq, Eq)]
    pub struct LookupField {
        full_name: String,
        target_sobject: String,
        lookup_type: LookupType,
        delete_constraint: DeleteConstraint,
    }

    #[derive(EnumString, Debug, PartialEq, Eq)]
    pub enum LookupType {
        Lookup,
        MasterDetail,
    }

    #[derive(EnumString, Debug, PartialEq, Eq)]
    pub enum DeleteConstraint {
        SetNull,
        Restrict,
        Cascade,
    }

    #[derive(Debug)]
    struct FieldDefinition {
        full_name: String,
        field_type: String,
        reference_to: Option<String>,
        delete_constraint: Option<String>,
    }

    impl SObject {
        pub fn name(&self) -> &str {
            &self.name
        }

        pub fn parse(file_path: &Path) -> SObject {
            let file_name_without_extension: String = file_path.file_stem()
                .and_then(std::ffi::OsStr::to_str)
                .map(std::borrow::ToOwned::to_owned)
                .unwrap();

            let mut sobject = SObject {
                name: file_name_without_extension,
                lookup_fields: Vec::new(),
            };

            let mut reader: Reader<BufReader<File>> = Reader::from_file(file_path).unwrap();
            reader.trim_text(true);
            let mut buffer: Vec<u8> = Vec::with_capacity(1024);

            let mut field_definition: Option<FieldDefinition> = None;
            let mut current_tag: String = "".to_owned();
            loop {
                match reader.read_event(&mut buffer) {
                    Ok(Event::Start(ref element)) if b"fields" == element.name() => {
                        field_definition.get_or_insert(FieldDefinition {
                            full_name: "".to_owned(),
                            field_type: "".to_owned(),
                            reference_to: None,
                            delete_constraint: None,
                        });
                    }
                    Ok(Event::Start(ref element)) => {
                        if field_definition.is_some() {
                            current_tag = String::from_utf8(element.name().to_owned()).unwrap();
                        };
                    }
                    Ok(Event::Text(text)) => {
                        if field_definition.is_none() {
                            continue;
                        };
                        if current_tag == "fullName" {
                            field_definition = field_definition.map(|mut f| {
                                f.full_name = text.unescape_and_decode(&reader).unwrap();
                                f
                            });
                        }
                        if current_tag == "type" {
                            field_definition = field_definition.map(|mut f| {
                                f.field_type = text.unescape_and_decode(&reader).unwrap();
                                f
                            });
                        }
                        if current_tag == "referenceTo" {
                            field_definition = field_definition.map(|mut f| {
                                f.reference_to = Some(text.unescape_and_decode(&reader).unwrap());
                                f
                            });
                        }
                        if current_tag == "deleteConstraint" {
                            field_definition = field_definition.map(|mut f| {
                                f.delete_constraint = Some(text.unescape_and_decode(&reader).unwrap());
                                f
                            });
                        }
                    }
                    Ok(Event::End(ref element)) if b"fields" == element.name() => {
                        let lookup_field = field_definition
                            .filter(|field| LookupType::from_str(&field.field_type).is_ok())
                            .map(|mut field| LookupField {
                                full_name: field.full_name.clone(),
                                target_sobject: field.reference_to.take().unwrap(),
                                lookup_type: LookupType::from_str(&field.field_type).unwrap(),
                                delete_constraint: DeleteConstraint::from_str(&field.delete_constraint.take().unwrap()).unwrap(),
                            });
                        if let Some(lookup_field) = lookup_field {
                            sobject.lookup_fields.push(lookup_field);
                        }

                        field_definition = None;
                    }
                    Ok(Event::Eof) => {
                        println!("Finished reading {:?}", file_path.file_name());
                        break;
                    }
                    Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
                    _ => (),
                };
                buffer.clear();
            };
            sobject
        }
    }

    #[cfg(test)]
    mod tests {
        use std::path::Path;

        use crate::salesforce::{LookupField, SObject};
        use crate::salesforce::DeleteConstraint::{SetNull, Restrict, Cascade};
        use crate::salesforce::LookupType::Lookup;

        #[test]
        fn parse_definition_correctly() {
            let account = SObject::parse(Path::new("../salesforce/tests/objects/Account.object"));

            assert_eq!("Account", account.name());

            assert_eq!(3, account.lookup_fields.len());

            assert_eq!(Some(&LookupField {
                full_name: "SetNull_Contact__c".to_owned(),
                target_sobject: "Contact".to_owned(),
                delete_constraint: SetNull,
                lookup_type: Lookup,
            }), account.lookup_fields.get(0));
            assert_eq!(Some(&LookupField {
                full_name: "Restrict_Contact__c".to_owned(),
                target_sobject: "Contact".to_owned(),
                delete_constraint: Restrict,
                lookup_type: Lookup,
            }), account.lookup_fields.get(1));
            assert_eq!(Some(&LookupField {
                full_name: "Cascade_Contact__c".to_owned(),
                target_sobject: "Contact".to_owned(),
                delete_constraint: Cascade,
                lookup_type: Lookup,
            }), account.lookup_fields.get(2));
        }
    }
}
