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
    // Datatypes.
    .whitelist_type("MPI_Datatype")
    // Communicators.
    .whitelist_type("MPI_Comm")
    // Groups.
    .whitelist_type("MPI_Group")
    // RMA and windows.
    .whitelist_type("MPI_Win")
    // File and I/O.
    //.whitelist_type("MPI_File")
    // Collective operations.
    .whitelist_type("MPI_Op")
    // Begin prototypes.
    .whitelist_function("MPI_Send")
    .whitelist_function("MPI_Recv")
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
    .generate()
    .expect("bindgen failed to generate mpi bindings");
  fs::remove_file(out_dir.join("mpi_bind.rs")).ok();
  mpi_bindings
    .write_to_file(out_dir.join("mpi_bind.rs"))
    .expect("bindgen failed to write mpi bindings");
}
