use bindgen::callbacks::{MacroParsingBehavior, ParseCallbacks};
use bindgen::RustTarget;
use std::env;
use std::path::PathBuf;

const INCLUDES: &str = "
#include <openssl/crypto.h>
#include <openssl/stack.h>
#include <openssl/x509v3.h>
";

pub fn run(include_dirs: &[PathBuf]) {
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    let mut builder = bindgen::builder()
        .parse_callbacks(Box::new(OpensslCallbacks))
        .rust_target(RustTarget::Stable_1_47)
        .ctypes_prefix("::libc")
        .raw_line("use libc::*;")
        .allowlist_file(".*/openssl/[^/]+.h")
        .allowlist_recursively(false)
        // libc is missing pthread_once_t on macOS
        .blocklist_type("CRYPTO_ONCE")
        .blocklist_function("CRYPTO_THREAD_run_once")
        // we don't want to mess with va_list
        .blocklist_function("BIO_vprintf")
        .blocklist_function("BIO_vsnprintf")
        // Maintain compatibility for existing enum definitions
        .rustified_enum("point_conversion_form_t")
        // Maintain compatibility for pre-union definitions
        .blocklist_type("GENERAL_NAME")
        .blocklist_type("EVP_PKEY")
        .layout_tests(false)
        .header_contents("includes.h", INCLUDES);

    for include_dir in include_dirs {
        builder = builder
            .clang_arg("-I")
            .clang_arg(include_dir.display().to_string());
    }

    builder
        .generate()
        .unwrap()
        .write_to_file(out_dir.join("bindgen.rs"))
        .unwrap();
}

#[derive(Debug)]
struct OpensslCallbacks;

impl ParseCallbacks for OpensslCallbacks {
    // for now we'll continue hand-writing constants
    fn will_parse_macro(&self, _name: &str) -> MacroParsingBehavior {
        MacroParsingBehavior::Ignore
    }

    fn item_name(&self, original_item_name: &str) -> Option<String> {
        match original_item_name {
            // Our original definitions of these are wrong, so rename to avoid breakage
            "CRYPTO_EX_new"
            | "CRYPTO_EX_dup"
            | "CRYPTO_EX_free"
            | "BIO_meth_set_write"
            | "BIO_meth_set_read"
            | "BIO_meth_set_puts"
            | "BIO_meth_set_ctrl"
            | "BIO_meth_set_create"
            | "BIO_meth_set_destroy" => Some(format!("{}__fixed_rust", original_item_name)),
            _ => None,
        }
    }
}
