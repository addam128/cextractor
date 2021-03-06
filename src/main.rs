
mod reader;
mod analyzers;
mod serialization;
mod models;
mod utils;

use std::env;
use std::process;
use std::collections::HashMap;

use analyzers::{revisions_finder::RevisionsFinder, traits::Analyzer};
use analyzers::version_finder::VersionFinder;
use analyzers::bibliography_finder::BibliographyFinder;
use analyzers::title_finder::TitleFinder;
use serialization::JsonSerializer;
use crate::analyzers::toc_finder::ToCFinder;


macro_rules! report_error {
    ($info: expr) => {
        eprintln!("\tWhoops: {}, skipping!", $info);  
    };
}


fn process(
    path: &str,
    analyzers: &mut HashMap< String, Box<dyn Analyzer> >)
    -> Result<(), utils::Error> {

    let infile = reader::open_file(path)?;
    let mut serializer = JsonSerializer::new(path)?;
    if infile.metadata()?.is_dir() {
        return Err(utils::Error::IsADirectory);
    }

    /* Do for the previous run, if this was at the end of the previous run,
    it could be omitted due to an Error, and would cause inconsistencies*/
    analyzers.values_mut().for_each(|a| a.as_mut().clear());

    reader::read_and_process_chunks(infile, analyzers)?;

    serializer.serialize(analyzers)?;

    Ok(())
}

fn main() {

    let mut analyzers = HashMap::< String, Box<dyn Analyzer> >::new();
    analyzers.insert(String::from("versions"), Box::new(VersionFinder::new().expect("Could not compile regex."))); 
    analyzers.insert(String::from("title"), Box::new(TitleFinder::new().expect("Could not compile regex.")));
    analyzers.insert(String::from("bibliography"), Box::new(BibliographyFinder::new().expect("Could not compile regex.")));
    analyzers.insert(String::from("revisions"), Box::new(RevisionsFinder::new().expect("Could not compile regex.")));
    analyzers.insert(String::from("table_of_contents"), Box::new(ToCFinder::new().expect("Could not compile regex.")));

    let retval = 
        env::args()
        .skip(1)
        .map(
            |arg: String| -> i32{
                match process(arg.as_str(), &mut analyzers) {
                    Ok(_) => {
                        println!("{} \u{2714}", arg.as_str()); 
                        0 
                    }
                    Err(err) => {
                        println!("{} \u{274c}", arg.as_str());
                        match err {
                            utils::Error::Io(err) => { report_error!(err); }
                            utils::Error::Utf8Conversion(err) => { report_error!(err); }
                            utils::Error::IsADirectory => { report_error!("this is a directory"); }
                            utils::Error::Regex(_) => {}
                            utils::Error::FancyRegex(_) => {}
                            utils::Error::BadRead => {report_error!("could not get more bytes from Read")}
                            utils::Error::UserChoice => {report_error!("user chose not to overwrite existing file")}
                        }
                        1 
                    }
                }
            }
        )
        .fold(0,
             |accumulator, elem| -> i32 { accumulator | elem });

    process::exit(retval);
}
