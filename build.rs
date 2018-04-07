extern crate bindgen;

use std::env;
use std::fs;
use std::path::{PathBuf};

fn main() {
  let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

  println!("cargo:rustc-link-lib=mpich");

  let mpi_bindings = bindgen::Builder::default()
    .header("wrapped.h")
    .whitelist_recursively(false)
    .whitelist_type("MPI_Status")
    .whitelist_type("MPI_Request")
    .whitelist_type("MPI_Offset")
    .whitelist_type("MPI_Info")
    .whitelist_var("MPI_SUCCESS")
    // Datatypes.
    .whitelist_type("MPI_Datatype")
    // Communicators.
    .whitelist_type("MPI_Comm")
    // Groups.
    .whitelist_type("MPI_Group")
    // RMA and windows.
    .whitelist_type("MPI_Win")
    // File and I/O.
    .whitelist_type("ADIOI_FileD")
    .whitelist_type("MPI_File")
    .whitelist_type("ADIOI_RequestD")
    .whitelist_type("MPIO_Request")
    .whitelist_var("MPI_MODE_RDONLY")
    // Collective operations.
    .whitelist_type("MPI_Op")
    // Begin prototypes.
    .whitelist_function("MPI_Send")
    .whitelist_function("MPI_Recv")
    .whitelist_function("MPI_Get_count")
    .whitelist_function("MPI_Get_elements")
    .whitelist_function("MPI_Isend")
    .whitelist_function("MPI_Irecv")
    .whitelist_function("MPI_Wait")
    .whitelist_function("MPI_Test")
    .whitelist_function("MPI_Request_free")
    .whitelist_function("MPI_Iprobe")
    .whitelist_function("MPI_Probe")
    .whitelist_function("MPI_Barrier")
    .whitelist_function("MPI_Bcast")
    .whitelist_function("MPI_Reduce")
    .whitelist_function("MPI_Allreduce")
    .whitelist_function("MPI_Init")
    .whitelist_function("MPI_Finalize")
    .whitelist_function("MPI_Initialized")
    // External interfaces.
    .whitelist_function("MPI_Init_thread")
    .whitelist_function("MPI_Query_thread")
    // Nonblocking collectives.
    .whitelist_function("MPI_Ibarrier")
    .whitelist_function("MPI_Ibcast")
    .whitelist_function("MPI_Ireduce")
    .whitelist_function("MPI_Iallreduce")
    // MPI-IO function prototypes.
    .whitelist_function("MPI_File_open")
    .whitelist_function("MPI_File_close")
    .whitelist_function("MPI_File_get_size")
    .whitelist_function("MPI_File_read_at")
    .whitelist_function("MPI_File_iread_at")
    .whitelist_function("MPIO_Test")
    .whitelist_function("MPIO_Wait")
    .generate()
    .expect("bindgen failed to generate mpi bindings");
  fs::remove_file(out_dir.join("mpi_bind.rs")).ok();
  mpi_bindings
    .write_to_file(out_dir.join("mpi_bind.rs"))
    .expect("bindgen failed to write mpi bindings");
}
