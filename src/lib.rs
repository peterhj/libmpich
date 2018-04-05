use ::ffi::*;

pub mod ffi;

pub struct MPIComm {
  raw:  i32,
}

impl MPIComm {
  pub fn world() -> Self {
    MPIComm{raw: MPI_COMM_WORLD}
  }
}
