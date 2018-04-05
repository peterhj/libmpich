use ::ffi::*;

pub mod ffi;

pub struct MPIComm {
  raw:      i32,
  owned:    bool,
}

impl MPIComm {
  pub fn world() -> Self {
    MPIComm{
      raw: MPI_COMM_WORLD,
      owned: false,
    }
  }

  pub fn self_() -> Self {
    MPIComm{
      raw: MPI_COMM_SELF,
      owned: false,
    }
  }
}
