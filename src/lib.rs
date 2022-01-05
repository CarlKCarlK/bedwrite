use ndarray as nd;

use ndarray_rand::rand_distr::Uniform;
use ndarray_rand::RandomExt;

use std::{
    fs::File,
    io::{BufWriter, Write},
};

use thiserror::Error;

const BED_FILE_MAGIC1: u8 = 0x6C; // 0b01101100 or 'l' (lowercase 'L')
const BED_FILE_MAGIC2: u8 = 0x1B; // 0b00011011 or <esc>

/// BedErrorPlus enumerates all possible errors returned by this library.
/// Based on https://nick.groenen.me/posts/rust-error-handling/#the-library-error-type
#[derive(Error, Debug)]
pub enum BedErrorPlus {
    #[error(transparent)]
    IOError(#[from] std::io::Error),

    #[error(transparent)]
    BedError(#[from] BedError),
}
// https://docs.rs/thiserror/1.0.23/thiserror/
#[derive(Error, Debug, Clone)]
pub enum BedError {
    #[error("Attempt to write illegal value to BED file. Only 0,1,2,missing allowed. '{0}'")]
    BadValue(String),
}

pub fn write(
    filename: &str,
    iid_count: usize,
    sid_count: usize,
    //cmk seed: usize,
) -> Result<(), BedErrorPlus> {
    assert!(iid_count % 4 == 0, "iid_count must be a multiple of 4");
    let iid_count_div4 = iid_count / 4;

    let val = nd::Array::random((iid_count, sid_count), Uniform::new(0., 10.));

    let mut writer = BufWriter::new(File::create(filename)?);
    writer.write_all(&[BED_FILE_MAGIC1, BED_FILE_MAGIC2, 0x01])?;

    for column in val.axis_iter(nd::Axis(1)) {
        let mut bytes_vector: Vec<u8> = vec![0; iid_count_div4]; // inits to 0
        for (iid_i, &v0) in column.iter().enumerate() {
            let genotype_byte = if v0 == 0.0 {
                0
            } else if v0 == 1.0 {
                1
            } else if v0 == 2.0 {
                2
            } else if v0 == 3.0 {
                3
            } else {
                return Err(BedError::BadValue(filename.to_string()).into());
            };
            // Possible optimization: We could pre-compute the conversion, the division, the mod, and the multiply*2
            let i_div_4 = iid_i / 4;
            let i_mod_4 = iid_i % 4;
            bytes_vector[i_div_4] |= genotype_byte << (i_mod_4 * 2);
        }
        writer.write_all(&bytes_vector)?;
    }
    return Ok(());
}
mod tests;
