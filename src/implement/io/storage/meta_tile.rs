use crate::binding::meta_tile::{
    META_MAGIC, META_MAGIC_COMPRESSED, entry, meta_layout
};
use crate::schema::apache2::config::ModuleConfig;
use crate::schema::http::encoding::ContentEncoding;
use crate::schema::tile::error::{
    InvalidMetaTileError, InvalidCompressionError, TileOffsetOutOfBoundsError,
};
use crate::schema::tile::identity::TileIdentity;
use crate::schema::tile::tile_ref::TileRef;

use mime::Mime;

use std::cmp::min;
use std::ffi::CStr;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;
use std::result::Result;


const META_TILE_WIDTH: i32 = 8;
const META_TILE_MASK: i32 = META_TILE_WIDTH - 1;

pub struct TilePath {
    pub meta_tile_path: PathBuf,
    pub tile_offset: u32,
}

pub struct MetaTile {
    raw_bytes: Rc<Vec<u8>>,
    tile_count: u32,
    media_type: Mime,
    encoding: ContentEncoding,
}

impl MetaTile {
    pub fn read(
        path: &PathBuf
    ) -> Result<MetaTile, InvalidMetaTileError> {
        let raw_bytes = Rc::new(fs::read(path)?);
        let layout = MetaTile::get_layout(&raw_bytes);
        let encoding = MetaTile::detect_compression(layout)?;
        let tile_count = MetaTile::detect_tile_count(layout)?;
        // TODO - verify tile media type
        let result = MetaTile {
            raw_bytes,
            tile_count,
            media_type: mime::IMAGE_PNG,
            encoding,
        };
        result.verify_tile_lengths()?;
        return Ok(result);
    }

    pub fn identity_to_path(
        config: &ModuleConfig,
        id: &TileIdentity,
    ) -> TilePath {
        let directory_hash = Self::calc_directory_hash(id);
        let mut path_buf = PathBuf::new();
        path_buf.push(&config.renderd.store_uri);
        path_buf.push(id.layer.as_str());
        path_buf.push(id.z.to_string());
        path_buf.push(directory_hash[4].to_string());
        path_buf.push(directory_hash[3].to_string());
        path_buf.push(directory_hash[2].to_string());
        path_buf.push(directory_hash[1].to_string());
        path_buf.push(format!("{}.meta", directory_hash[0]));
        let offset = Self::calc_offset(id);
        let path = TilePath {
            meta_tile_path: path_buf,
            tile_offset: offset,
        };
        return path;
    }

    pub fn select(
        &self,
        tile_offset: u32
    ) -> Result<TileRef, TileOffsetOutOfBoundsError> {
        let entry = self.get_entry(tile_offset)?;
        let selected_tile_start = entry.offset as usize;
        let next_tile_start= (entry.offset + entry.size) as usize;
        return Ok(
            TileRef {
                raw_bytes: Rc::downgrade(&self.raw_bytes),
                begin: selected_tile_start,
                end: next_tile_start,
                media_type: self.media_type.clone(),
                encoding: self.encoding.clone(),
            }
        );
    }

    fn calc_directory_hash(
        id: &TileIdentity,
    ) -> [i32; 5] {
        let mut x_dir = id.x & !META_TILE_MASK;
        let mut y_dir = id.y & !META_TILE_MASK;
        let mut dir_hash: [i32; 5] = [0; 5];
        for hash_part in &mut dir_hash {
            // In original C code in mod_tile, the type of hash_part is u8, but
            // conversion from higher precision integer to lower precision is
            // implementation defined; making a behavior assumption here
            *hash_part = ((x_dir & 0x0f) << 4) | (y_dir & 0x0f);
            x_dir = x_dir >> 4;
            y_dir = y_dir >> 4;
        }
        return dir_hash;
    }

    fn calc_offset(
        id: &TileIdentity,
    ) -> u32 {
        ((id.x & META_TILE_MASK) * META_TILE_WIDTH + (id.y & META_TILE_MASK)) as u32
    }

    fn get_layout(
        raw_bytes: &Vec<u8>
    ) -> &meta_layout {
        unsafe {
            (raw_bytes.as_ptr() as *const meta_layout).as_ref().unwrap()
        }
    }

    fn detect_compression(
        layout: &meta_layout
    ) -> Result<ContentEncoding, InvalidCompressionError> {
        let raw_tag = unsafe {
            CStr::from_ptr(layout.magic.as_ptr()).to_str()?
        };
        let expected_uncompressed_tag = std::str::from_utf8(META_MAGIC.strip_suffix(&[0]).unwrap())?;
        let uncompressed_tag_length = min(expected_uncompressed_tag.len(), raw_tag.len());
        let actual_uncompressed_tag = &(raw_tag[..uncompressed_tag_length]);

        let expected_gzip_tag = std::str::from_utf8(META_MAGIC_COMPRESSED.strip_suffix(&[0]).unwrap())?;
        let gzip_tag_length = min(expected_gzip_tag.len(), raw_tag.len());
        let actual_gzip_tag = &(raw_tag[..gzip_tag_length]);

        if actual_uncompressed_tag == expected_uncompressed_tag {
            Ok(ContentEncoding::NotCompressed)
        } else if actual_gzip_tag == expected_gzip_tag {
            Ok(ContentEncoding::Gzip)
        } else {
            Err(InvalidCompressionError::InvalidTag(raw_tag.to_string()))
        }
    }

    fn detect_tile_count(
        layout: &meta_layout
    ) -> Result<u32, InvalidMetaTileError> {
        let expected_tile_count = META_TILE_WIDTH * META_TILE_WIDTH;
        let actual_tile_count = layout.count;
        if actual_tile_count > 0 && actual_tile_count == expected_tile_count {
            Ok(actual_tile_count as u32)
        } else {
            Err(InvalidMetaTileError::InvalidTileCount(actual_tile_count))
        }
    }

    fn get_entry(
        &self,
        tile_offset: u32,
    ) -> Result<&entry, TileOffsetOutOfBoundsError> {
        if tile_offset >= self.tile_count {
            return Err(
                TileOffsetOutOfBoundsError {
                    tile_offset
                }
            );
        }
        let layout = MetaTile::get_layout(&self.raw_bytes);
        let tile_index = unsafe {
            layout.index.as_slice(self.tile_count as usize)
        };
        return Ok(&tile_index[tile_offset as usize]);
    }

    fn verify_tile_lengths(
        &self,
    ) -> Result<(), InvalidMetaTileError> {
        for tile_offset in 0..self.tile_count {
            let entry = self.get_entry(tile_offset).unwrap();
            let end_of_tile_position= (entry.offset + entry.size - 1) as usize;
            let file_size = self.raw_bytes.len();
            if end_of_tile_position > file_size {
                return Err(
                    InvalidMetaTileError::InvalidTileLength(tile_offset)
                );
            }
        }
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::tile::identity::LayerName;
    use std::boxed::Box;
    use std::error::Error;
    use std::env;
    use std::result::Result;
    use std::string::String;

    #[test]
    fn test_calc_offset_x_positive_y_positive_small() -> Result<(), Box<dyn Error>> {
        let id1 = TileIdentity {
            x: 0 + 0 + 1,
            y: 4 + 2 + 1,
            z: 5,
            layer: LayerName::from("default"),
        };
        let offset1 = MetaTile::calc_offset(&id1);
        assert_eq!((8 * 1) + 7, offset1, "Incorrect offset calculation");

        let id2 = TileIdentity {
            x: 0 + 2 + 1,
            y: 4 + 2 + 1,
            z: 5,
            layer: LayerName::from("default"),
        };
        let offset2 = MetaTile::calc_offset(&id2);
        assert_eq!((8 * 3) + 7, offset2, "Incorrect offset calculation");

        let id3 = TileIdentity {
            x: 0 + 2 + 1,
            y: 4 + 2 + 0,
            z: 5,
            layer: LayerName::from("default"),
        };
        let offset3 = MetaTile::calc_offset(&id3);
        assert_eq!((8 * 3) + 6, offset3, "Incorrect offset calculation");
        Ok(())
    }

    #[test]
    fn test_calc_offset_x_positive_y_positive_large() -> Result<(), Box<dyn Error>> {
        let id1 = TileIdentity {
            x: 32 + 0 + 0 + 4 + 0 + 1,
            y: 0 + 16 + 0 + 0 + 2 + 1,
            z: 3,
            layer: LayerName::from("default"),
        };
        let offset1 = MetaTile::calc_offset(&id1);
        assert_eq!((8 * 5) + 3, offset1, "Incorrect offset calculation");

        let id2 = TileIdentity {
            x: 32 + 0 + 8 + 4 + 0 + 1,
            y: 0 + 16 + 0 + 0 + 2 + 1,
            z: 3,
            layer: LayerName::from("default"),
        };
        let offset2 = MetaTile::calc_offset(&id2);
        assert_eq!((8 * 5) + 3, offset2, "Incorrect offset calculation");

        let id3 = TileIdentity {
            x: 32 + 0 + 8 + 4 + 0 + 1,
            y: 32 + 16 + 0 + 0 + 2 + 1,
            z: 3,
            layer: LayerName::from("default"),
        };
        let offset3 = MetaTile::calc_offset(&id3);
        assert_eq!((8 * 5) + 3, offset3, "Incorrect offset calculation");
        Ok(())
    }

    #[test]
    fn test_calc_dir_hash_x_positive_y_positive() -> Result<(), Box<dyn Error>> {
        let id1 = TileIdentity {
            x: 000000 + 000000 + 000000 + 65536 + 00000 + 00000 + 0000 + 4096 + 0000 + 0000 + 000 + 256 + 000 + 00 + 00 + 16 + 0 + 4 + 2 + 1,
            y: 000000 + 000000 + 000000 + 00000 + 00000 + 00000 + 0000 + 0000 + 0000 + 0000 + 000 + 000 + 000 + 00 + 00 + 00 + 0 + 4 + 2 + 1,
            z: 5,
            layer: LayerName::from("default"),
        };
        let hash1 = MetaTile::calc_directory_hash(&id1);
        assert_eq!(0, hash1[0], "Incorrect directory hash calculation");
        assert_eq!(16, hash1[1], "Incorrect directory hash calculation");
        assert_eq!(16, hash1[2], "Incorrect directory hash calculation");
        assert_eq!(16, hash1[3], "Incorrect directory hash calculation");
        assert_eq!(16, hash1[4], "Incorrect directory hash calculation");

        let id2 = TileIdentity {
            x: 524288 + 000000 + 000000 + 00000 + 32768 + 00000 + 0000 + 0000 + 2048 + 0000 + 000 + 000 + 128 + 00 + 00 + 00 + 8 + 0 + 0 + 1,
            y: 000000 + 000000 + 000000 + 00000 + 00000 + 00000 + 0000 + 0000 + 0000 + 0000 + 000 + 000 + 000 + 00 + 00 + 00 + 0 + 4 + 2 + 1,
            z: 5,
            layer: LayerName::from("default"),
        };
        let hash2 = MetaTile::calc_directory_hash(&id2);
        assert_eq!(128, hash2[0], "Incorrect directory hash calculation");
        assert_eq!(128, hash2[1], "Incorrect directory hash calculation");
        assert_eq!(128, hash2[2], "Incorrect directory hash calculation");
        assert_eq!(128, hash2[3], "Incorrect directory hash calculation");
        assert_eq!(128, hash2[4], "Incorrect directory hash calculation");

        let id3 = TileIdentity {
            x: 524288 + 000000 + 000000 + 00000 + 32768 + 00000 + 0000 + 0000 + 2048 + 0000 + 000 + 000 + 128 + 00 + 00 + 00 + 8 + 0 + 0 + 1,
            y: 000000 + 000000 + 000000 + 65536 + 00000 + 00000 + 0000 + 4096 + 0000 + 0000 + 000 + 256 + 000 + 00 + 00 + 16 + 8 + 4 + 2 + 1,
            z: 5,
            layer: LayerName::from("default"),
        };
        let hash3 = MetaTile::calc_directory_hash(&id3);
        assert_eq!(136, hash3[0], "Incorrect directory hash calculation");
        assert_eq!(129, hash3[1], "Incorrect directory hash calculation");
        assert_eq!(129, hash3[2], "Incorrect directory hash calculation");
        assert_eq!(129, hash3[3], "Incorrect directory hash calculation");
        assert_eq!(129, hash3[4], "Incorrect directory hash calculation");

        let id4 = TileIdentity {
            x: 524288 + 000000 + 000000 + 00000 + 32768 + 00000 + 0000 + 0000 + 2048 + 0000 + 000 + 000 + 128 + 00 + 00 + 00 + 8 + 0 + 0 + 1,
            y: 000000 + 262144 + 000000 + 00000 + 00000 + 16384 + 0000 + 0000 + 0000 + 1024 + 000 + 000 + 000 + 64 + 00 + 00 + 8 + 4 + 2 + 1,
            z: 5,
            layer: LayerName::from("default"),
        };
        let hash4 = MetaTile::calc_directory_hash(&id4);
        assert_eq!(136, hash4[0], "Incorrect directory hash calculation");
        assert_eq!(132, hash4[1], "Incorrect directory hash calculation");
        assert_eq!(132, hash4[2], "Incorrect directory hash calculation");
        assert_eq!(132, hash4[3], "Incorrect directory hash calculation");
        assert_eq!(132, hash4[4], "Incorrect directory hash calculation");
        Ok(())
    }

    #[test]
    fn test_read_valid_basic_meta_tile() -> Result<(), InvalidMetaTileError> {
        let mut test_store_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        test_store_path.push("resources");
        test_store_path.push("test");
        test_store_path.push("meta_tile");
        let mut config = ModuleConfig::new();
        config.renderd.store_uri = String::from(test_store_path.to_str().unwrap());
        let id = TileIdentity {
            x: 000000 + 000000 + 000000 + 00000 + 00000 + 00000 + 0000 + 0000 + 0000 + 0000 + 512 + 256 + 128 + 00 + 32 + 16 + 0 + 0 + 0 + 0,
            y: 000000 + 000000 + 000000 + 00000 + 00000 + 00000 + 0000 + 0000 + 0000 + 0000 + 512 + 000 + 000 + 64 + 32 + 00 + 8 + 0 + 0 + 0,
            z: 10,
            layer: LayerName::from("default"),
        };
        let hash = MetaTile::calc_directory_hash(&id);
        assert_eq!(8, hash[0], "Incorrect directory hash calculation");
        assert_eq!(182, hash[1], "Incorrect directory hash calculation");
        assert_eq!(50, hash[2], "Incorrect directory hash calculation");
        assert_eq!(0, hash[3], "Incorrect directory hash calculation");
        assert_eq!(0, hash[4], "Incorrect directory hash calculation");
        let path = MetaTile::identity_to_path(&config, &id);
        let meta_tile = MetaTile::read(&path.meta_tile_path)?;
        for tile_offset in 0..meta_tile.tile_count {
            let mut path = env::temp_dir();
            path.push(format!("basic-{}.png", tile_offset));
            let tile_ref = meta_tile.select(tile_offset).unwrap();
            tile_ref.with_tile(|raw_bytes| {
                std::fs::write(path, raw_bytes).expect("Tile write failed");
            });
        }
        Ok(())
    }

    #[test]
    fn test_read_valid_complex_meta_tile() -> Result<(), InvalidMetaTileError> {
        let mut test_store_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        test_store_path.push("resources");
        test_store_path.push("test");
        test_store_path.push("meta_tile");
        let mut config = ModuleConfig::new();
        config.renderd.store_uri = String::from(test_store_path.to_str().unwrap());
        let id = TileIdentity {
            x: 000000 + 000000 + 000000 + 00000 + 00000 + 00000 + 0000 + 0000 + 0000 + 0000 + 000 + 000 + 000 + 00 + 32 + 16 + 8 + 0 + 0 + 0,
            y: 000000 + 000000 + 000000 + 00000 + 00000 + 00000 + 0000 + 0000 + 0000 + 0000 + 000 + 000 + 000 + 00 + 32 + 00 + 0 + 0 + 0 + 0,
            z: 6,
            layer: LayerName::from("default"),
        };
        let hash = MetaTile::calc_directory_hash(&id);
        assert_eq!(128, hash[0], "Incorrect directory hash calculation");
        assert_eq!(50, hash[1], "Incorrect directory hash calculation");
        assert_eq!(0, hash[2], "Incorrect directory hash calculation");
        assert_eq!(0, hash[3], "Incorrect directory hash calculation");
        assert_eq!(0, hash[4], "Incorrect directory hash calculation");
        let path = MetaTile::identity_to_path(&config, &id);
        let meta_tile = MetaTile::read(&path.meta_tile_path)?;
        for tile_offset in 0..meta_tile.tile_count {
            let mut path = env::temp_dir();
            path.push(format!("complex-{}.png", tile_offset));
            let tile_ref = meta_tile.select(tile_offset).unwrap();
            tile_ref.with_tile(|raw_bytes| {
                std::fs::write(path, raw_bytes).expect("Tile write failed");
            });
        }
        Ok(())
    }
}
