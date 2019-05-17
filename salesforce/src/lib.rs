pub mod salesforce {
    use std::path::Path;

    use quick_xml::events::Event;
    use quick_xml::Reader;

    pub enum DeleteConstraint {
        SetNull,
        Restrict,
        Cascade,
    }

    pub enum ReferenceType {
        Lookup,
        MasterDetail,
    }

    pub struct Reference {
        to: String,
        reference_type: ReferenceType,
        delete_constraint: DeleteConstraint,
    }

    pub struct SObject {
        name: String,
        references: Vec<Reference>,
    }

    impl SObject {
        pub fn name(&self) -> &str {
            &self.name
        }

        pub fn references(&self) -> &[Reference] {
            &self.references[..]
        }

        pub fn parse(file_path: &Path) -> SObject {
            let file_name_without_extension: String = file_path.file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.to_owned())
                .unwrap();

            let mut sobject = SObject {
                name: file_name_without_extension,
                references: Vec::new(),
            };

            let mut buffer: Vec<u8> = Vec::with_capacity(1024);
            let mut reader = Reader::from_file(file_path).unwrap();
            loop {
                match reader.read_event(&mut buffer) {
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

        use crate::salesforce::SObject;

        #[test]
        fn parse_definition_correctly() {
            let account = SObject::parse(Path::new("../salesforce/tests/objects/Account.object"));

            assert_eq!("Account", account.name());
            assert_eq!(3, account.references().len());
        }
    }
}
