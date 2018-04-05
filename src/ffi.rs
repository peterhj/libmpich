#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

use std::os::raw::{c_int, c_char};

// FIXME: bindgen doesnt support "mpi.h" macro defines,
// so we have to manually write bindings for constants.
//include!(concat!(env!("OUT_DIR"), "/mpi_bind.rs"));

pub const MPI_THREAD_SINGLE:        c_int = 0;
pub const MPI_THREAD_FUNNELED:      c_int = 1;
pub const MPI_THREAD_SERIALIZED:    c_int = 2;
pub const MPI_THREAD_MULTIPLE:      c_int = 3;

extern "C" {
  pub fn MPI_Init(argc: *mut c_int, argv: *mut *mut *mut c_char) -> c_int;
  pub fn MPI_Init_thread(argc: *mut c_int, argv: *mut *mut *mut c_char, required: c_int, provided: *mut c_int) -> c_int;
}

pub type MPI_Datatype = c_int;

pub const MPI_CHAR:   MPI_Datatype = 0x4c000101;
pub const MPI_BYTE:   MPI_Datatype = 0x4c00010d;
pub const MPI_FLOAT:  MPI_Datatype = 0x4c00040a;
pub const MPI_DOUBLE: MPI_Datatype = 0x4c00080b;

pub type MPI_Comm = c_int;

pub const MPI_COMM_WORLD: MPI_Comm = 0x44000000;
pub const MPI_COMM_SELF:  MPI_Comm = 0x44000001;

pub type MPI_Op = c_int;

pub const MPI_MAX:    MPI_Op = 0x58000001;
pub const MPI_MIN:    MPI_Op = 0x58000002;
pub const MPI_SUM:    MPI_Op = 0x58000003;
pub const MPI_PROD:   MPI_Op = 0x58000004;
