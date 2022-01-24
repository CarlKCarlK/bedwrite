use dpc_pariter::{scope, IteratorExt};
use ndarray as nd;

use std::{
    fs::{self, File},
    io::{BufWriter, Write},
};

use thiserror::Error;

pub fn write(
    filename: &str,
    iid_count: usize, //e.g. 5000, 50_000, 500_000
    sid_count: usize, //e.g. 5 to 50_000 (but only to 5K when iid is 500K)
    high: i8,
    num_threads: usize,
) -> Result<(), BedError> {
    assert!(iid_count % 4 == 0, "iid_count must be a multiple of 4");
    let iid_count_div4 = iid_count / 4;
    let val: nd::Array2<i8> = nd::Array::from_elem((iid_count, sid_count), high);

    if let Err(e) = write_internal(filename, iid_count_div4, &val, num_threads) {
        // Clean up the file
        let _ = fs::remove_file(filename);
        Err(e)
    } else {
        Ok(())
    }
}

fn write_internal(
    filename: &str,
    iid_count_div4: usize,
    val: &nd::Array2<i8>,
    num_threads: usize,
) -> Result<(), BedError> {
    let mut writer = BufWriter::new(File::create(filename)?);

    // Return this expression to the caller.
    // The 'scope' guarantees that all threads end, thus
    // no thread  can continue borrowing 'val' after 'scope'.
    scope(|scope| {
        // for each column in the array
        val.axis_iter(nd::Axis(1))
            // '.parallel_map_scoped' works with 'scope' to avoid
            // a 'borrowed value does not live long enough' error.
            .parallel_map_scoped(scope, {
                |column| {
                    // Convert each column into a bytes_vector
                    let mut bytes_vector: Vec<u8> = vec![0; iid_count_div4]; // inits to 0
                    for (iid_i, &v0) in column.iter().enumerate() {
                        if v0 > 3 {
                            return Err(BedError::BadValue(filename.to_string()));
                        }
                        let byte = v0 as u8;
                        let i_div_4 = iid_i / 4;
                        let i_mod_4 = iid_i % 4;
                        bytes_vector[i_div_4] |= byte << (i_mod_4 * 2);
                    }
                    Ok(bytes_vector)
                }
            })
            // Set the number of threads to use
            .threads(num_threads)
            // Sequentially, write each column to the file.
            // If there is an error, stop early and return it.
            .try_for_each(|bytes_vector: Result<_, BedError>| {
                writer.write_all(&bytes_vector?)?;
                Ok(())
            })
    })
    // In the unlikely event of a scope error, return this error.
    .map_err(|_e| BedError::PanickedThread())?
}

#[derive(Error, Debug)]
pub enum BedError {
    #[error("Attempt to write illegal value to BED file. Only 0,1,2,missing allowed. '{0}'")]
    BadValue(String),

    #[error(transparent)]
    IOError(#[from] std::io::Error),

    #[error("Multithreading resulted in panic(s)")]
    PanickedThread(),
}

#[test]
fn test1() {
    write("test1.bed", 12, 10, 3, 12).unwrap();
}

#[test]
fn test2() {
    let result = write("test2.bed", 12, 10, 4, 1);
    assert!(result.is_err());
}

#[test]
fn big1() {
    // too slow:
    write("big1.bed", 50_000, 5000, 2, 1).unwrap();
}

#[test]
fn big12() {
    // too slow:
    write("big12.bed", 50_000, 5000, 2, 12).unwrap();
}
