extern crate structopt;

use std::path::PathBuf;

use structopt::StructOpt;

use salesforce;
use salesforce::SObject;

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
struct CliArg {
    #[structopt(short, long)]
    debug: bool,
    #[structopt(parse(from_os_str), value_delimiter = ";")]
    paths: Vec<PathBuf>,
    #[structopt(short = "a", long)]
    apex_template: Option<String>,
}

fn main() {
    let mut args: CliArg = CliArg::from_args();
    if args.apex_template == None {
        args.apex_template = Some("{}".to_owned());
    }
    println!("{:#?}", args);

    let mut sobjects: Vec<SObject> = vec![];
    for path in &args.paths {
        sobjects.extend(SObject::parse(path));
    }

    match SObject::delete_order(&sobjects) {
        None => println!("There is circular reference. Unable to generate delete query"),
        Some(sorted_objects) => {
            let delete_queries: Vec<String> = sorted_objects.into_iter()
                .map(|sobject| {
                    let cloned_query_template = &args.apex_template
                        .as_ref()
                        .unwrap()
                        .clone();
                    cloned_query_template.replace("{}", sobject.name())
                })
                .collect();
            println!("########## Output ##########");
            for query in delete_queries.iter() {
                println!("{}", query);
            }
        }
    };
}
