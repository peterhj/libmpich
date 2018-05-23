use ::ffi::*;

use std::ffi::{CString};
use std::mem::{zeroed};
use std::os::unix::ffi::{OsStrExt};
use std::path::{Path};
use std::ptr::{null_mut};

pub mod ffi;

// TODO
pub type MPIResult<T> = Result<T, i32>;

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
      _ => Err(status),
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
          _ => Err(res2),
        }
      }
      _ => Err(res),
    }
  }
}
