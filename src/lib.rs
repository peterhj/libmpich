use ::ffi::*;

use std::env;
use std::ffi::{CString};
use std::mem::{zeroed};
use std::os::raw::{c_void};
use std::os::unix::ffi::{OsStrExt};
use std::path::{Path};
use std::ptr::{null_mut};

pub mod ffi;

fn sz2int(sz: usize) -> i32 {
  assert!(sz <= i32::max_value() as _);
  sz as _
}

// TODO: dedicated error type.
pub type MPIResult<T> = Result<T, u32>;

pub fn mpi_init_multithreaded() -> MPIResult<()> {
  let args: Vec<_> = env::args().collect();
  let mut raw_args = vec![];
  for arg in args.iter() {
    let raw_arg = CString::new(arg.clone()).unwrap();
    raw_args.push(raw_arg);
  }
  let mut provided: i32 = -1;
  let res = {
    let mut argc: i32 = args.len() as _;
    let mut argv_buf: Vec<*mut i8> = vec![];
    for raw_arg in raw_args.iter() {
      argv_buf.push(raw_arg.as_ptr() as *mut _);
    }
    let mut argv = argv_buf.as_mut_ptr();
    unsafe { MPI_Init_thread(
        &mut argc as *mut _,
        &mut argv as *mut *mut *mut _,
        MPI_THREAD_SERIALIZED as _,
        &mut provided as *mut _,
    ) }
  };
  match res as u32 {
    MPI_SUCCESS => {
      assert_eq!(provided, MPI_THREAD_SERIALIZED as _);
      Ok(())
    }
    e => Err(e),
  }
}

pub fn mpi_finalize() -> MPIResult<()> {
  let res = unsafe { MPI_Finalize() };
  match res as u32 {
    MPI_SUCCESS => Ok(()),
    e => Err(e),
  }
}

pub struct MPIComm {
  raw:      MPI_Comm,
  owned:    bool,
}

impl Drop for MPIComm {
  fn drop(&mut self) {
    if self.owned {
      let status = unsafe { MPI_Comm_disconnect(&mut self.raw as *mut _) };
      assert_eq!(status, MPI_SUCCESS as _);
      assert_eq!(self.raw, MPI_COMM_NULL);
    }
  }
}

impl MPIComm {
  pub fn world() -> Self {
    MPIComm{
      raw:      MPI_COMM_WORLD,
      owned:    false,
    }
  }

  pub fn self_() -> Self {
    MPIComm{
      raw:      MPI_COMM_SELF,
      owned:    false,
    }
  }

  pub fn rank(&self) -> i32 {
    let mut rank: i32 = 0;
    let status = unsafe { MPI_Comm_rank(self.raw, &mut rank as *mut _) };
    assert_eq!(status, MPI_SUCCESS as _);
    rank
  }

  pub fn num_ranks(&self) -> i32 {
    let mut nranks: i32 = 0;
    let status = unsafe { MPI_Comm_size(self.raw, &mut nranks as *mut _) };
    assert_eq!(status, MPI_SUCCESS as _);
    nranks
  }
}

#[derive(Clone, Copy, Debug)]
pub enum MPIReduceOp {
  Max,
  Min,
  Sum,
  Prod,
}

impl MPIReduceOp {
  pub fn to_raw_op(&self) -> MPI_Op {
    match *self {
      MPIReduceOp::Max => MPI_MAX,
      MPIReduceOp::Min => MPI_MIN,
      MPIReduceOp::Sum => MPI_SUM,
      MPIReduceOp::Prod => MPI_PROD,
    }
  }
}

pub trait MPIDataTypeExt: Copy {
  fn mpi_data_type() -> MPI_Datatype;
}

impl MPIDataTypeExt for u8  { fn mpi_data_type() -> MPI_Datatype { MPI_BYTE } }
impl MPIDataTypeExt for f32 { fn mpi_data_type() -> MPI_Datatype { MPI_FLOAT } }
impl MPIDataTypeExt for f64 { fn mpi_data_type() -> MPI_Datatype { MPI_DOUBLE } }

pub fn mpi_barrier(comm: &mut MPIComm) -> MPIResult<()> {
  let res = unsafe { MPI_Barrier(comm.raw) };
  match res as u32 {
    MPI_SUCCESS => Ok(()),
    e => Err(e),
  }
}

pub unsafe fn mpi_bcast<T: MPIDataTypeExt>(buf: *mut T, len: usize, root: i32, comm: &mut MPIComm) -> MPIResult<()> {
  let count = sz2int(len);
  let res = unsafe { MPI_Bcast(
      buf as *mut c_void,
      count,
      T::mpi_data_type(),
      root,
      comm.raw,
  ) };
  match res as u32 {
    MPI_SUCCESS => Ok(()),
    e => Err(e),
  }
}

pub unsafe fn mpi_reduce<T: MPIDataTypeExt>(src: *const T, src_len: usize, dst: *mut T, dst_len: usize, op: MPIReduceOp, root: i32, comm: &mut MPIComm) -> MPIResult<()> {
  assert_eq!(src_len, dst_len);
  let count = sz2int(src_len);
  let res = unsafe { MPI_Reduce(
      src as *const c_void,
      dst as *mut c_void,
      count,
      T::mpi_data_type(),
      op.to_raw_op(),
      root,
      comm.raw,
  ) };
  match res as u32 {
    MPI_SUCCESS => Ok(()),
    e => Err(e),
  }
}

pub unsafe fn mpi_allreduce<T: MPIDataTypeExt>(src: *const T, src_len: usize, dst: *mut T, dst_len: usize, op: MPIReduceOp, comm: &mut MPIComm) -> MPIResult<()> {
  assert_eq!(src_len, dst_len);
  let count = sz2int(src_len);
  let res = unsafe { MPI_Allreduce(
      src as *const c_void,
      dst as *mut c_void,
      count,
      T::mpi_data_type(),
      op.to_raw_op(),
      comm.raw,
  ) };
  match res as u32 {
    MPI_SUCCESS => Ok(()),
    e => Err(e),
  }
}

pub struct MPIFile {
  raw:  MPI_File,
}

impl Drop for MPIFile {
  fn drop(&mut self) {
    let status = unsafe { MPI_File_close(&mut self.raw as *mut _) };
    assert_eq!(status, MPI_SUCCESS as _);
    self.raw = null_mut();
  }
}

impl MPIFile {
  pub fn open<P>(path: P, comm: &mut MPIComm) -> MPIResult<MPIFile> where P: AsRef<Path> {
    let path_cstr = CString::new(path.as_ref().as_os_str().as_bytes()).unwrap();
    let mut raw: MPI_File = null_mut();
    let status = unsafe { MPI_File_open(
        comm.raw,
        path_cstr.as_ptr(),
        MPI_MODE_RDONLY as _,
        MPI_INFO_NULL,
        &mut raw as *mut _,
    ) };
    match status as u32 {
      MPI_SUCCESS => Ok(MPIFile{raw: raw}),
      e => Err(e),
    }
  }

  pub fn close(self) {
  }

  pub fn read_at(&mut self, offset: i64, buf: &mut [u8]) -> MPIResult<usize> {
    assert!(buf.len() <= i32::max_value() as usize);
    let count = buf.len() as i32;
    let mut status: MPI_Status = unsafe { zeroed() };
    let res = unsafe { MPI_File_read_at(
        self.raw,
        offset,
        buf.as_mut_ptr() as *mut _,
        count,
        MPI_BYTE,
        &mut status as *mut _,
    ) };
    match res as u32 {
      MPI_SUCCESS => {
        // TODO: `MPI_Get_count` vs `MPI_Get_elements`?
        let mut recv_count: i32 = -1;
        let res2 = unsafe { MPI_Get_count(
            &status as *const _,
            MPI_BYTE,
            &mut recv_count as *mut _,
        ) };
        match res2 as u32 {
          MPI_SUCCESS => {
            assert!(recv_count >= 0);
            assert!(recv_count <= count);
            Ok(recv_count as _)
          }
          e2 => Err(e2),
        }
      }
      e => Err(e),
    }
  }
}
