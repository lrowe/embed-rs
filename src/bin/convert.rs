extern crate clap;
extern crate flexbuffers;
extern crate serde_json;
extern crate serde_transcode;
use std::error::Error;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {
    let matches = clap::App::new("Convert")
        .arg_from_usage("--in [FILE] 'input file'")
        .arg_from_usage("--out [FILE] 'output file'")
        .arg(
            clap::Arg::from_usage("--format-in [TYPE] 'input type (json|bin)'")
                .possible_values(&["json", "bin"]),
        )
        .arg(
            clap::Arg::from_usage("--format-out [TYPE] 'output type (json|bin)'")
                .possible_values(&["json", "bin"]),
        )
        .get_matches();

    let format_in = match matches.value_of("format-in") {
        Some(format) => format,
        _ => match matches.value_of("in") {
            Some(path) => match Path::new(&path).extension().and_then(|ext| ext.to_str()) {
                Some("json") => "json",
                Some("bin") => "bin",
                _ => panic!("must specify format-in"),
            },
            _ => panic!("must specify format-in"),
        },
    };

    let format_out = match matches.value_of("format-out") {
        Some(format) => format,
        _ => match matches.value_of("out") {
            Some(path) => match Path::new(&path).extension().and_then(|ext| ext.to_str()) {
                Some("json") => "json",
                Some("bin") => "bin",
                _ => panic!("must specify format-out"),
            },
            _ => panic!("must specify format-out"),
        },
    };

    let mut reader: Box<dyn Read> = match matches.value_of("in") {
        Some(path) if path != "-" => Box::new(File::open(path)?),
        _ => Box::new(io::stdin()),
    };
    let mut buf_in = Vec::new();
    reader.read_to_end(&mut buf_in)?;

    let mut writer: Box<dyn Write> = match matches.value_of("out") {
        Some(path) if path != "-" => Box::new(File::create(path)?),
        _ => Box::new(io::stdout()),
    };

    // Each branch must be separate because of type specialization.
    // Cannot use Box<dyn Serializer> because serde::Serializer lacks Sized.
    match format_out {
        "json" => {
            let mut serializer = serde_json::Serializer::new(&mut writer);
            match format_in {
                "json" => {
                    let mut deserializer = serde_json::Deserializer::from_slice(&buf_in);
                    serde_transcode::transcode(&mut deserializer, &mut serializer)?;
                    deserializer.end()?;
                }
                "bin" => {
                    let deserializer = flexbuffers::Reader::get_root(&buf_in)?;
                    serde_transcode::transcode(deserializer, &mut serializer)?;
                }
                _ => panic!("unreachable"),
            }
            writer.write_all(b"\n")?;
        }
        "bin" => {
            let mut serializer = flexbuffers::FlexbufferSerializer::new();
            match format_in {
                "json" => {
                    let mut deserializer = serde_json::Deserializer::from_slice(&buf_in);
                    serde_transcode::transcode(&mut deserializer, &mut serializer)?;
                    deserializer.end()?;
                }
                "bin" => {
                    let deserializer = flexbuffers::Reader::get_root(&buf_in)?;
                    serde_transcode::transcode(deserializer, &mut serializer)?;
                }
                _ => panic!("unreachable"),
            }
            writer.write_all(serializer.view())?;
        }
        _ => panic!("unreachable"),
    }
    Ok(())
}
