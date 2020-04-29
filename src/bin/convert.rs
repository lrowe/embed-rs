extern crate flexbuffers;
extern crate serde_json;
use std::env;
use std::io::{self, Read, Write};
use std::time::Instant;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        3 => {
            let format_in = &args[1];
            let format_out = &args[2];
            let mut buf_in = Vec::new();
            let begin = Instant::now();
            io::stdin().lock().read_to_end(&mut buf_in)?;

            let begin_de = Instant::now();
            let builder: flexbuffers::Builder = match &format_in[..] {
                "json" => serde_json::from_slice(&buf_in)?,
                "flexbuffer" => flexbuffers::from_slice(&buf_in).unwrap(),
                _ => panic!("invalid arg format_in"),
            };

            let begin_ser = Instant::now();
            let buffer = builder.view();
            let reader = flexbuffers::Reader::get_root(buffer).unwrap();
            let buf_out = match &format_out[..] {
                "json" => serde_json::to_vec(&reader)?,
                "flexbuffer" => flexbuffers::to_vec(&reader).unwrap(),
                _ => panic!("invalid arg format_out"),
            };
            let end_ser = Instant::now();

            io::stdout().lock().write_all(&buf_out)?;
            let end = Instant::now();

            eprintln!(
                "total:{:?} de:{:?} output:{:?}",
                end.duration_since(begin),
                begin_ser.duration_since(begin_de),
                end_ser.duration_since(begin_ser)
            );
            Ok(())
        }
        _ => {
            eprintln!(
                "usage: {} <format_in> <format_out>\nformats: json, flexbuffer",
                args[0]
            );
            panic!("invalid args");
        }
    }
}
