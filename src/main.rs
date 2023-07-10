use std::env;
use std::error::Error;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::io::Read;
use std::mem;
use std::path::Path;
use std::slice;
use std::str;

use simple_endian::*;

use vtfx::VTFXHEADER;

mod vtfx;

fn main() {
    let mut args: Vec<String> = env::args().collect();

    args.push(String::from("E:\\Rust Projects\\vtfx_reader\\00000290.vtf"));
    
    if args.len() == 1 {
        println!("VTFX Reader");
        println!("Enter path of file:");
        let mut buffer = String::new();
        let stdin = io::stdin();
        match stdin.read_line(&mut buffer) {
            Err(e) => println!("Input not valid"),
            Ok(_) => (),
        }
        //Remove trailing new line chars
        if buffer.ends_with('\n') {
            buffer.pop();
            if buffer.ends_with('\r') {
                buffer.pop();
            }
        }
        buffer = buffer.replace("\"", "");
        //Check path is file
        let path = Path::new(&buffer);

        if path.is_file() {
            args.push(buffer);
        } else {
            println!("Provided path '{}' is not a file!", buffer);
            return;
        }
    }

    match read_vtfx(Path::new(&args[1])) {
        Ok(_) => {println!("Success")},
        Err(e) => {println!("Failed to open file: {e}")},
    };
}

fn read_vtfx(path: &Path) -> Result<VTFXHEADER, Box<dyn Error>> {
    let f = File::open(path)?;
    let mut reader = BufReader::new(f);
    let mut buffer = Vec::new();

    reader.read_to_end(&mut buffer)?;

    let type_str_range = &buffer[0..4];
    let type_str = str::from_utf8(&type_str_range)?;
    if type_str != "VTFX"
    {
        let err = io::Error::new(io::ErrorKind::Other, "File is not VTFX file!");
        return Err(Box::new(err));
    }
    else
    {
        println!("File is VTFX file!");
    }

    let mut vtfx: VTFXHEADER = unsafe { mem::zeroed() };

    println!("{:?}", vtfx);

    Ok(vtfx)
}
