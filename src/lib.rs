use ndarray as nd;

use ndarray_rand::rand::SeedableRng;
use ndarray_rand::RandomExt;
use ndarray_rand::{rand::prelude::StdRng, rand_distr::Uniform};

use std::{
    fs::File,
    io::{BufWriter, Write},
};

use thiserror::Error;

/// Based on https://nick.groenen.me/posts/rust-error-handling/#the-library-error-type
#[derive(Error, Debug)]
pub enum BedError {
    #[error("Attempt to write illegal value to BED file. Only 0,1,2,missing allowed. '{0}'")]
    BadValue(String),

    #[error(transparent)]
    IOError(#[from] std::io::Error),
}

pub fn write(
    filename: &str,
    iid_count: usize,
    sid_count: usize,
    high: f64,
    seed: u64,
) -> Result<(), BedError> {
    assert!(iid_count % 4 == 0, "iid_count must be a multiple of 4");
    let iid_count_div4 = iid_count / 4;
    let mut rng = StdRng::seed_from_u64(seed);

    let val = nd::Array::random_using((iid_count, sid_count), Uniform::new(0.0, high), &mut rng);

    let mut writer = BufWriter::new(File::create(filename)?);
    for column in val.axis_iter(nd::Axis(1)) {
        // Covert each column into a bytes_vector
        let mut bytes_vector: Vec<u8> = vec![0; iid_count_div4]; // inits to 0
        for (iid_i, &v0) in column.iter().enumerate() {
            let byte = if v0 < 4.0 {
                (v0 / 4.0f64).floor() as u8
            } else {
                return Err(BedError::BadValue(filename.to_string()).into());
            };
            let i_div_4 = iid_i / 4;
            let i_mod_4 = iid_i % 4;
            bytes_vector[i_div_4] |= byte << (i_mod_4 * 2);
        }
        // Write the bytes vector
        writer.write_all(&bytes_vector)?;
    }
    return Ok(());
}
mod tests;
