
use std::io::{BufRead, Seek, SeekFrom};

use byteorder::{ReadBytesExt, LittleEndian};

use crate::errors::{ImageError, ImageResult, ImageResultU};
use crate::types::{Dimensions, Format, ImageMeta};



const GIF87A: [u8; 6] = *b"GIF87a";
const GIF89A: [u8; 6] = *b"GIF89a";


#[derive(Default)]
struct BlockReader {
    frames: usize,
}


pub fn load<R: BufRead + Seek>(image: &mut R) -> ImageResult<ImageMeta> {
    read_signature(image)?;
    let dimensions = read_header(image)?;

    let mut reader = BlockReader::default();
    reader.read(image)?;

    Ok(ImageMeta {
        animation_frames: if 1 < reader.frames { Some(reader.frames) } else { None },
        dimensions,
        format: Format::Gif,
    })
}

fn read_signature<R: BufRead + Seek>(image: &mut R) -> ImageResultU {
    let mut signature = [0u8;6];
    image.read_exact(&mut signature)?;
    match signature {
        GIF87A | GIF89A => Ok(()),
        _ => Err(ImageError::InvalidSignature),
    }
}

fn read_header<R: BufRead + Seek>(image: &mut R) -> ImageResult<Dimensions> {
    let width = image.read_u16::<LittleEndian>().map(u32::from)?;
    let height = image.read_u16::<LittleEndian>().map(u32::from)?;

    let table_bytes = read_table_bits(image)?;

    // 1 Background color index
    // 1 Aspect Ratio

    image.seek(SeekFrom::Current(table_bytes + 2))?;

    Ok(Dimensions { width, height })
}

impl BlockReader {
    fn read<R: BufRead + Seek>(&mut self, image: &mut R) -> ImageResultU {
        loop {
            let b = image.read_u8()?;
            match b {
                0x21 => self.read_extension(image)?,
                0x2c => self.read_image_data(image)?,
                0x3b => return Ok(()),
                x => return Err(ImageError::CorruptImage(format!("Unknown block: {:x}", x).into())),
            };
        }
    }

    fn read_extension<R: BufRead + Seek>(&mut self, image: &mut R) -> ImageResultU {
        match image.read_u8()? {
            0x01 | 0xf9 | 0xfe | 0xff => (),
            x => return Err(ImageError::CorruptImage(format!("Unknown extension: {:x}", x).into())),
        };
        loop {
            let size = image.read_u8()?;
            if size == 0 {
                return Ok(());
            }
            image.seek(SeekFrom::Current(i64::from(size)))?;
        }
    }

    fn read_image_data<R: BufRead + Seek>(&mut self, image: &mut R) -> ImageResultU {
        // 2 Left
        // 2 Top
        // 2 Width
        // 2 Height
        image.seek(SeekFrom::Current(8))?;

        let table_bytes = read_table_bits(image)?;
        image.seek(SeekFrom::Current(table_bytes + 1))?; // `+ 1` means LZW minimum code size

        loop {
            let size = image.read_u8()?;
            if size == 0 {
                break;
            }
            image.seek(SeekFrom::Current(i64::from(size)))?;
        }

        self.frames += 1;
        Ok(())
    }
}

/// Returns the bytes to skip
fn read_table_bits<R: BufRead>(image: &mut R) -> ImageResult<i64> {
    let bits = image.read_u8()?;
    let has_table = (bits & 0b1000_0000) > 0;
    let table_size = 2 << (bits & 0b0000_0111);
    if has_table {
        Ok(i64::from(table_size) * 3)
    } else {
        Ok(0)
    }
}
