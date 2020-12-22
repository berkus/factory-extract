//!
//! Parse .factory files and extract contained payload into separate files.
//!

use {
    byteorder::{BigEndian, ReadBytesExt},
    nom::{number::complete::be_u32, *},
    std::{fs::DirBuilder, io::Cursor, path::PathBuf},
};

struct NamedPayload {
    pub file_name: PathBuf,
    pub payload: Vec<u8>,
}

named!(fetch_payload, length_data!(be_u32));

fn file_path_name(i: &[u8]) -> IResult<&[u8], String> {
    let (i, payload) = fetch_payload(i)?;
    assert_eq!(payload.len() % 2, 0);
    let mut converted_path = Vec::<u16>::with_capacity(payload.len() / 2);
    let mut reader = Cursor::new(payload);
    for _ in 0..payload.len() / 2 {
        converted_path.push(reader.read_u16::<BigEndian>().unwrap());
    }
    Ok((i, String::from_utf16_lossy(converted_path.as_slice())))
}

fn block_payload(i: &[u8]) -> IResult<&[u8], Vec<u8>> {
    let (i, payload) = fetch_payload(i)?;
    Ok((i, Vec::from(payload)))
}

fn parse_block(i: &[u8]) -> IResult<&[u8], NamedPayload> {
    let (i, file_name) = file_path_name(i)?;
    let (i, payload) = block_payload(i)?;

    let file_name = PathBuf::from(file_name);

    Ok((i, NamedPayload { file_name, payload }))
}

named!(parse_file<&[u8], Vec<NamedPayload>>, many0!(parse_block));

fn main() -> std::io::Result<()> {
    let in_file_path = std::env::args()
        .nth(1)
        .expect("No .factory file path given");
    let dry_run = std::env::args().nth(2).unwrap_or("".into()) == "-d";

    let i = std::fs::read(in_file_path)?;

    let (_, blocks) = parse_file(i.as_bytes()).expect("Shall parse!");
    for (i, b) in blocks.iter().enumerate() {
        println!("Block {}: {:?} size {}", i, b.file_name, b.payload.len());
        let file_name = b
            .file_name
            .strip_prefix("/")
            .unwrap_or_else(|_| &b.file_name);
        if !dry_run {
            DirBuilder::new()
                .recursive(true)
                .create(file_name.parent().unwrap())?;
            std::fs::write(file_name, &b.payload)?;
        }
    }
    Ok(())
}
