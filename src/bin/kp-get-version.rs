/// utility to get the version of a `KeePass` database.
use std::fs::File;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Provide a .kdbx database
    in_kdbx: String,
}

pub fn main() -> Result<(), keepass_ng::BoxError> {
    let args = Args::parse();

    let mut source = File::open(args.in_kdbx)?;

    let version = keepass_ng::Database::get_version(&mut source)?;
    println!("{}", version);
    Ok(())
}
