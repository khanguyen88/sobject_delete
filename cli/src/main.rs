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
    let args: CliArg = CliArg::from_args();
    println!("{:#?}", args);

    let mut sobjects: Vec<SObject> = vec![];
    for path in &args.paths {
        sobjects.extend(SObject::parse(path));
    }
    println!("{:#?}", sobjects);
}
