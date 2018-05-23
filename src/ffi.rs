#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

use std::os::raw::{c_int};

// FIXME: bindgen doesnt support "mpi.h" macro defines,
// so we have to manually write bindings for constants.
include!(concat!(env!("OUT_DIR"), "/mpi_bind.rs"));

pub const MPI_THREAD_SINGLE:        c_int = 0;
pub const MPI_THREAD_FUNNELED:      c_int = 1;
pub const MPI_THREAD_SERIALIZED:    c_int = 2;
pub const MPI_THREAD_MULTIPLE:      c_int = 3;

pub const MPI_CHAR:   MPI_Datatype = 0x4c000101;
pub const MPI_BYTE:   MPI_Datatype = 0x4c00010d;
pub const MPI_FLOAT:  MPI_Datatype = 0x4c00040a;
pub const MPI_DOUBLE: MPI_Datatype = 0x4c00080b;

pub const MPI_COMM_NULL:  MPI_Comm = 0x04000000;
pub const MPI_COMM_WORLD: MPI_Comm = 0x44000000;
pub const MPI_COMM_SELF:  MPI_Comm = 0x44000001;

pub const MPI_MAX:    MPI_Op = 0x58000001;
pub const MPI_MIN:    MPI_Op = 0x58000002;
pub const MPI_SUM:    MPI_Op = 0x58000003;
pub const MPI_PROD:   MPI_Op = 0x58000004;

pub const MPI_INFO_NULL:  MPI_Info = 0x1c000000;
pub const MPI_INFO_ENV:   MPI_Info = 0x5c000001;
