use ::ffi::*;

use std::env;
use std::ffi::{CString};
use std::marker::{PhantomData};
use std::mem::{size_of, zeroed};
use std::os::raw::{c_void};
use std::os::unix::ffi::{OsStrExt};
use std::path::{Path};
use std::ptr::{null_mut};
use std::slice::{from_raw_parts, from_raw_parts_mut};

pub mod ffi;

fn sz2int(sz: usize) -> i32 {
  assert!(sz <= i32::max_value() as _);
  sz as i32
}

fn sz2longint(sz: usize) -> i64 {
  assert!(sz <= i64::max_value() as _);
  sz as i64
}

fn u8s_to_typed<T: Copy>(buf: &[u8], new_len: usize) -> &[T] {
  let ptr = buf.as_ptr();
  let size = buf.len();
  assert!(size >= new_len * size_of::<T>());
  // TODO: check alignment.
  unsafe { from_raw_parts(ptr as *const T, new_len * size_of::<T>()) }
}

fn u8s_to_typed_mut<T: Copy>(buf: &mut [u8], new_len: usize) -> &mut [T] {
  let ptr = buf.as_mut_ptr();
  let size = buf.len();
  assert!(size >= new_len * size_of::<T>());
  // TODO: check alignment.
  unsafe { from_raw_parts_mut(ptr as *mut T, new_len * size_of::<T>()) }
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
impl MPIDataTypeExt for u64 { fn mpi_data_type() -> MPI_Datatype { MPI_UNSIGNED_LONG_LONG } }
impl MPIDataTypeExt for f32 { fn mpi_data_type() -> MPI_Datatype { MPI_FLOAT } }
impl MPIDataTypeExt for f64 { fn mpi_data_type() -> MPI_Datatype { MPI_DOUBLE } }

pub struct MPIStatus {
  raw:  MPI_Status,
}

impl MPIStatus {
  pub fn new() -> Self {
    MPIStatus{
      raw:  unsafe { zeroed() },
    }
  }

  pub fn as_mut_ptr(&mut self) -> *mut MPI_Status {
    &mut self.raw as *mut _
  }
}

pub struct MPIMem {
  base: *mut c_void,
  size: usize,
}

impl Drop for MPIMem {
  fn drop(&mut self) {
    assert!(!self.base.is_null());
    let res = unsafe { MPI_Free_mem(self.base) };
    assert_eq!(res as u32, MPI_SUCCESS);
  }
}

impl MPIMem {
  pub fn alloc(size: usize) -> Self {
    let mut base: *mut c_void = null_mut();
    let res = unsafe { MPI_Alloc_mem(
        sz2longint(size),
        MPI_INFO_NULL,
        (&mut base as *mut *mut c_void) as *mut c_void,
    ) };
    assert_eq!(res as u32, MPI_SUCCESS);
    MPIMem{
      base: base,
      size: size,
    }
  }

  pub fn as_slice(&self) -> &[u8] {
    unsafe { from_raw_parts(self.base as *const u8, self.size) }
  }

  pub fn as_mut_slice(&mut self) -> &mut [u8] {
    unsafe { from_raw_parts_mut(self.base as *mut u8, self.size) }
  }
}

pub struct MPIWindowLockSharedGuard<'a, T: 'static> {
  rank: i32,
  win:  &'a MPIWindow<T>,
}

impl<'a, T: 'static> Drop for MPIWindowLockSharedGuard<'a, T> {
  fn drop(&mut self) {
    let res = unsafe { MPI_Win_unlock(self.rank, self.win.raw) };
    assert_eq!(res as u32, MPI_SUCCESS);
  }
}

impl<'a, T: MPIDataTypeExt + 'static> MPIWindowLockSharedGuard<'a, T> {
  pub fn as_slice(&'a self) -> &'a [T] {
    unsafe { from_raw_parts(self.win.base, self.win.len) }
  }

  pub unsafe fn get_mem(&'a self, buf: *mut T, len: usize, tg_rank: i32, tg_offset: usize, tg_len: usize) -> MPIResult<()> {
    let res = unsafe { MPI_Get(
        buf as *mut c_void,
        sz2int(len),
        T::mpi_data_type(),
        tg_rank,
        sz2longint(tg_offset),
        sz2int(tg_len),
        T::mpi_data_type(),
        self.win.raw,
    ) };
    match res as u32 {
      MPI_SUCCESS => Ok(()),
      e => Err(e),
    }
  }
}

pub struct MPIWindowLockExclGuard<'a, T: 'static> {
  rank: i32,
  win:  &'a mut MPIWindow<T>,
}

impl<'a, T: 'static> Drop for MPIWindowLockExclGuard<'a, T> {
  fn drop(&mut self) {
    let res = unsafe { MPI_Win_unlock(self.rank, self.win.raw) };
    assert_eq!(res as u32, MPI_SUCCESS);
  }
}

impl<'a, T: MPIDataTypeExt + 'static> MPIWindowLockExclGuard<'a, T> {
  pub fn as_slice(&'a self) -> &'a [T] {
    //u8s_to_typed(self.win.mem.as_slice(), self.win.len)
    unsafe { from_raw_parts(self.win.base, self.win.len) }
  }

  pub fn as_mut_slice(&'a mut self) -> &'a mut [T] {
    //u8s_to_typed_mut(self.win.mem.as_mut_slice(), self.win.len)
    unsafe { from_raw_parts_mut(self.win.base, self.win.len) }
  }
}

pub struct MPIWindow<T> {
  rank: i32,
  //mem:  MPIMem,
  base: *mut T,
  len:  usize,
  size: usize,
  raw:  MPI_Win,
  _mrk: PhantomData<*mut T>,
}

impl<T> Drop for MPIWindow<T> {
  fn drop(&mut self) {
    let res = unsafe { MPI_Win_free(&mut self.raw as *mut _) };
    assert_eq!(res as u32, MPI_SUCCESS);
  }
}

impl<T: MPIDataTypeExt> MPIWindow<T> {
  /*pub fn new(len: usize, comm: &mut MPIComm) -> MPIResult<Self> {
    let rank = comm.rank();
    let size = size_of::<T>() * len;
    let mem = MPIMem::alloc(size);
    let mut raw_win: MPI_Win = 0;
    let res = unsafe { MPI_Win_create(
        mem.base,
        sz2longint(mem.size),
        sz2int(size_of::<T>()),
        MPI_INFO_NULL,
        comm.raw,
        &mut raw_win as *mut _,
    ) };
    match res as u32 {
      MPI_SUCCESS => Ok(MPIWindow{
        rank:   rank,
        len:    len,
        mem:    mem,
        raw:    raw_win,
        _mrk:   PhantomData,
      }),
      e => Err(e),
    }
  }*/

  pub unsafe fn new(base: *mut T, len: usize, comm: &mut MPIComm) -> MPIResult<Self> {
    let rank = comm.rank();
    let size = size_of::<T>() * len;
    assert!(sz2longint(size) >= 0);
    let mut raw_win: MPI_Win = 0;
    let res = unsafe { MPI_Win_create(
        base as *mut c_void,
        sz2longint(size),
        sz2int(size_of::<T>()),
        MPI_INFO_NULL,
        comm.raw,
        &mut raw_win as *mut _,
    ) };
    match res as u32 {
      MPI_SUCCESS => Ok(MPIWindow{
        rank:   rank,
        //mem:    mem,
        base:   base,
        len:    len,
        size:   size,
        raw:    raw_win,
        _mrk:   PhantomData,
      }),
      e => Err(e),
    }
  }

  pub fn lock_shared(&self) -> MPIWindowLockSharedGuard<T> {
    let res = unsafe { MPI_Win_lock(
        MPI_LOCK_SHARED as _,
        self.rank,
        0,
        self.raw,
    ) };
    assert_eq!(res as u32, MPI_SUCCESS);
    MPIWindowLockSharedGuard{
      rank: self.rank,
      win:  self,
    }
  }

  pub fn lock(&mut self) -> MPIWindowLockExclGuard<T> {
    let res = unsafe { MPI_Win_lock(
        MPI_LOCK_EXCLUSIVE as _,
        self.rank,
        0,
        self.raw,
    ) };
    assert_eq!(res as u32, MPI_SUCCESS);
    MPIWindowLockExclGuard{
      rank: self.rank,
      win:  self,
    }
  }
}

pub unsafe fn mpi_send<T: MPIDataTypeExt>(buf: *const T, len: usize, dst: i32, tag: i32, comm: &mut MPIComm) -> MPIResult<()> {
  let res = MPI_Send(
      buf as *const _,
      sz2int(len),
      T::mpi_data_type(),
      dst,
      tag,
      comm.raw,
  );
  match res as u32 {
    MPI_SUCCESS => Ok(()),
    e => Err(e),
  }
}

pub unsafe fn mpi_recv<T: MPIDataTypeExt>(buf: *mut T, len: usize, src: i32, tag: i32, comm: &mut MPIComm) -> MPIResult<()> {
  let mut status = MPIStatus::new();
  let res = MPI_Recv(
      buf as *mut _,
      sz2int(len),
      T::mpi_data_type(),
      src,
      tag,
      comm.raw,
      status.as_mut_ptr(),
  );
  match res as u32 {
    MPI_SUCCESS => Ok(()),
    e => Err(e),
  }
}

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
      if src as *mut T == dst { MPI_IN_PLACE } else { src as *const c_void },
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
