extern crate strum;
extern crate strum_macros;

use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::str::FromStr;

use quick_xml::events::Event;
use quick_xml::Reader;
use strum_macros::EnumString;

use graph;
use graph::DirectedGraph;

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

pub fn is_sobject(object_name: &str) -> bool {
    !object_name.ends_with("__e")
}

pub fn delete_order(sobjects: &[SObject]) -> Option<Vec<&SObject>> {
    let delete_dependency = to_graph(sobjects);
    delete_dependency
        .topological_sort().
        map(|sorted_indices| {
            let mut result: Vec<&SObject> = Vec::with_capacity(sorted_indices.len());
            for index in sorted_indices {
                result.push(sobjects.get(index).unwrap());
            }
            result
        })
}

fn map_name_to_index(sobjects: &[SObject]) -> HashMap<&str, usize> {
    let mut index_by_sobject_name: HashMap<&str, usize> = HashMap::with_capacity(sobjects.len());
    for (index, sobject) in sobjects.iter().enumerate() {
        index_by_sobject_name.insert(sobject.name(), index);
    }
    index_by_sobject_name
}

fn to_graph(sobjects: &[SObject]) -> DirectedGraph {
    let index_by_sobject_name = map_name_to_index(sobjects);

    let mut graph = graph::DirectedGraph::new(sobjects.len());
    for (index, sobject) in sobjects.iter().enumerate() {
        let mut edges: Vec<usize> = Vec::with_capacity(sobject.lookup_fields.len());

        for field in &sobject.lookup_fields {
            if field.lookup_type == LookupType::Lookup && field.delete_constraint == DeleteConstraint::Restrict {
                edges.push(*index_by_sobject_name.get(field.target_sobject.as_str()).unwrap());
            }
        }
        graph.add_edges(index, edges.as_slice());
    }
    graph
}

impl SObject {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn parse(file_path: &PathBuf) -> Vec<SObject> {
        if file_path.is_dir() {
            let mut paths: Vec<PathBuf> = vec![];
            for entry in file_path.read_dir().expect("read_dir call failed") {
                if let Ok(entry) = entry {
                    paths.push(entry.path());
                };
            };
            paths.iter()
                .filter(|path| path.is_file() && is_sobject(path.file_stem().and_then(std::ffi::OsStr::to_str).unwrap()))
                .map(SObject::parse_sobject_file)
                .collect()
        } else {
            vec![SObject::parse_sobject_file(file_path)]
        }
    }

    fn parse_sobject_file(file_path: &PathBuf) -> SObject {
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

    use crate::{delete_order, is_sobject, LookupField, SObject};
    use crate::DeleteConstraint::{Cascade, Restrict, SetNull};
    use crate::LookupType::Lookup;

    #[test]
    fn parse_definition_correctly() {
        let account = SObject::parse(&Path::new("../salesforce/tests/objects/Account.object").to_path_buf());

        assert_eq!(1, account.len());

        let account = account.get(0).unwrap();

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

    #[test]
    fn parse_folder_correctly() {
        let sobjects = SObject::parse(&Path::new("../salesforce/tests/objects").to_path_buf());

        assert_eq!(2, sobjects.len());

        {
            let account = sobjects.get(0).unwrap();
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

        {
            let contact = sobjects.get(1).unwrap();
            assert_eq!("Contact", contact.name());
            assert_eq!(0, contact.lookup_fields.len());
        }
    }

    #[test]
    fn test_sobject_detection() {
        assert_eq!(true, is_sobject("Account"));
        assert_eq!(true, is_sobject("Custom_Object__c"));
        assert_eq!(false, is_sobject("Custom_Event__e"));
    }

    #[test]
    fn test_delete_sort() {
        let sobjects = SObject::parse(&Path::new("../salesforce/tests/objects").to_path_buf());

        let sorted_sobjects = delete_order(&sobjects).unwrap();

        assert_eq!(2, sorted_sobjects.len());
        assert_eq!("Account", sorted_sobjects[0].name);
        assert_eq!("Contact", sorted_sobjects[1].name);
    }
}
