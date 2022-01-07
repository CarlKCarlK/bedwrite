use dpc_pariter::IteratorExt;
use ndarray as nd;

use std::{
    fs::File,
    io::{BufWriter, Write},
};

use thiserror::Error;

pub fn write(
    filename: &str,
    iid_count: usize,
    sid_count: usize,
    high: f64,
) -> Result<(), BedError> {
    assert!(iid_count % 4 == 0, "iid_count must be a multiple of 4");
    let iid_count_div4 = iid_count / 4;
    let val = nd::Array::from_elem((iid_count, sid_count), high - 0.01);

    let mut writer = BufWriter::new(File::create(filename)?);

    val.axis_iter(nd::Axis(1))
        .parallel_map(|column| {
            // Covert each column into a bytes_vector
            let mut bytes_vector: Vec<u8> = vec![0; iid_count_div4]; // inits to 0
            for (iid_i, &v0) in column.iter().enumerate() {
                let byte = (v0 / 4.0f64).floor() as u8;
                let i_div_4 = iid_i / 4;
                let i_mod_4 = iid_i % 4;
                bytes_vector[i_div_4] |= byte << (i_mod_4 * 2);
            }
            bytes_vector
        })
        .for_each(|bytes_vector| {
            writer.write_all(&bytes_vector).unwrap();
        });
    return Ok(());
}

// val.axis_iter(nd::Axis(1))
//     .parallel_map(|column| {
//         // Covert each column into a bytes_vector
//         let mut bytes_vector: Vec<u8> = vec![0; iid_count_div4]; // inits to 0
//         for (iid_i, &v0) in column.iter().enumerate() {
//             let byte = if v0 < 4.0 {
//                 (v0 / 4.0f64).floor() as u8
//             } else {
//                 return Err(BedError::BadValue(filename.to_string()).into());
//             };
//             let i_div_4 = iid_i / 4;
//             let i_mod_4 = iid_i % 4;
//             bytes_vector[i_div_4] |= byte << (i_mod_4 * 2);
//         }
//         Ok(bytes_vector)
//     })
//     .try_for_each(|result: Result<Vec<u8>, BedError>| match result {
//         Ok(bytes_vector) => {
//             for byte in bytes_vector {
//                 writer.write_all(&[byte])?;
//             }
//             Ok(())
//         }
//         Err(e) => Err(e),
//     })?;
// .try_for_each(|result| match result {
//     Ok(bytes_vector) => {
//         writer.write_all(&bytes_vector)?;
//     }
//     Err(e) => return Err(e),
// });

// for column in val.axis_iter(nd::Axis(1)) {
//     // Covert each column into a bytes_vector
//     let mut bytes_vector: Vec<u8> = vec![0; iid_count_div4]; // inits to 0
//     for (iid_i, &v0) in column.iter().enumerate() {
//         let byte = if v0 < 4.0 {
//             (v0 / 4.0f64).floor() as u8
//         } else {
//             return Err(BedError::BadValue(filename.to_string()).into());
//         };
//         let i_div_4 = iid_i / 4;
//         let i_mod_4 = iid_i % 4;
//         bytes_vector[i_div_4] |= byte << (i_mod_4 * 2);
//     }
//     // Write the bytes vector, they must be in order.
//     writer.write_all(&bytes_vector)?;
// }
// return Ok(());
//}

#[derive(Error, Debug)]
pub enum BedError {
    #[error("Attempt to write illegal value to BED file. Only 0,1,2,missing allowed. '{0}'")]
    BadValue(String),

    #[error(transparent)]
    IOError(#[from] std::io::Error),
}

#[test]
fn test1() {
    write("test1.bed", 12, 10, 4.0).unwrap();
}

#[test]
fn test2() {
    let result = write("test2.bed", 12, 10, 5.0);
    assert!(result.is_err());
}

#[test]
fn big() {
    // too slow:
    write("big.bed", 1_000_000, 100, 4.0).unwrap();
}
