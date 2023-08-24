use std::error::Error;
use std::fs;
use std::io::ErrorKind;

use libivf_rs::{IvfReader, IvfWriter};

fn check_ivf(filename: &str) -> Result<(), Box<dyn Error>> {
    let file = fs::File::open(filename)?;
    let mut reader = IvfReader::init(file)?;

    assert_eq!(&reader.header.fourcc, b"VP80");
    assert_eq!(reader.header.width, 160);
    assert_eq!(reader.header.height, 120);
    assert_eq!(reader.header.framerate_num, 15);
    assert_eq!(reader.header.framerate_den, 1);
    assert_eq!(reader.header.num_frames, 75);

    let mut frame_index = 0;
    loop {
        match reader.read_ivf_frame_header() {
            Ok(ivf_frame_header) => {
                assert_eq!(ivf_frame_header.timestamp, frame_index as _);
                reader.skip_frame(ivf_frame_header.frame_size)?;
            }
            Err(e) if e.kind() == ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(Box::new(e)),
        }
        frame_index += 1;
    }
    Ok(())
}

#[test]
fn ivf_reader() {
    check_ivf("testfiles/sample01_vp8.ivf").unwrap();
}

fn copy_ivf(filename: &str, outfilename: &str) -> Result<(), Box<dyn Error>> {
    let file = fs::File::open(filename)?;
    let outfile = fs::File::create(outfilename)?;
    let mut reader = IvfReader::init(file)?;

    assert_eq!(&reader.header.fourcc, b"VP80");
    assert_eq!(reader.header.width, 160);
    assert_eq!(reader.header.height, 120);
    assert_eq!(reader.header.framerate_num, 15);
    assert_eq!(reader.header.framerate_den, 1);
    assert_eq!(reader.header.num_frames, 75);

    let mut writer = IvfWriter::init(outfile, &reader.header)?;

    let mut frame_index = 0;
    let mut buf = vec![0u8; 64 * 1024];
    loop {
        match reader.read_ivf_frame_header() {
            Ok(ivf_frame_header) => {
                assert_eq!(ivf_frame_header.timestamp, frame_index as _);
                reader.read_frame(&mut buf[..ivf_frame_header.frame_size as _])?;
                writer.write_ivf_frame(&buf[..ivf_frame_header.frame_size as _], ivf_frame_header.timestamp)?;
            }
            Err(e) if e.kind() == ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(Box::new(e)),
        }
        frame_index += 1;
    }
    Ok(())
}

#[test]
fn ivf_writer() {
    copy_ivf("testfiles/sample01_vp8.ivf", "testfiles/out.ivf").unwrap();
    check_ivf("testfiles/out.ivf").unwrap();
}
