extern crate bindgen;

use std::env;
use std::path::{PathBuf};

fn main() {
  let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

  println!("cargo:rustc-link-lib=mpich");

  let mpi_bindings = bindgen::Builder::default()
    .clang_arg(format!("-I/opt/cray/pe/mpt/7.6.0/gni/mpich-gnu/5.1/include"))
    .header("wrapped.h")
    .whitelist_type("MPI_Request")
    .whitelist_type("MPI_Errhandler")
    .whitelist_type("MPI_Message")
    // Datatypes.
    .whitelist_type("MPI_Datatype")
    .whitelist_var("MPI_CHAR")
    .whitelist_var("MPI_SIGNED_CHAR")
    .whitelist_var("MPI_UNSIGNED_CHAR")
    .whitelist_var("MPI_BYTE")
    .whitelist_var("MPI_FLOAT")
    .whitelist_var("MPI_DOUBLE")
    // Communicators.
    .whitelist_type("MPI_Comm")
    .whitelist_var("MPI_COMM_WORLD")
    .whitelist_var("MPI_COMM_SELF")
    // Groups.
    .whitelist_type("MPI_Group")
    .whitelist_var("MPI_GROUP_EMPTY")
    // RMA and windows.
    .whitelist_type("MPI_Win")
    .whitelist_var("MPI_WIN_NULL")
    // File and I/O.
    .whitelist_type("MPI_File")
    // Collective operations.
    .whitelist_type("MPI_Op")
    .whitelist_var("MPI_MAX")
    .whitelist_var("MPI_MIN")
    .whitelist_var("MPI_SUM")
    .whitelist_var("MPI_PROD")
    .generate()
    .expect("bindgen failed to generate mpi bindings");
  mpi_bindings
    .write_to_file(out_dir.join("mpi_bind.rs"))
    .expect("bindgen failed to write mpi bindings");
}
