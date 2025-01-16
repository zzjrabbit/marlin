use std::env;

use camino::Utf8PathBuf;
use proc_macro::TokenStream;
use verilog_macro_builder::{build_verilated_struct, MacroArgs};

fn search_for_swim_toml(mut start: Utf8PathBuf) -> Option<Utf8PathBuf> {
    while !start.as_str().is_empty() {
        if start.join("swim.toml").is_file() {
            return Some(start.join("swim.toml"));
        }
        start.pop();
    }
    None
}

#[proc_macro_attribute]
pub fn spade(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(args as MacroArgs);

    let manifest_directory = Utf8PathBuf::from(
        env::var("CARGO_MANIFEST_DIR").expect("Please use CARGO"),
    );
    let Some(swim_toml) = search_for_swim_toml(manifest_directory) else {
        return syn::Error::new_spanned(
            args.source_path,
            "Could not find swim.toml",
        )
        .into_compile_error()
        .into();
    };

    let verilog_args = MacroArgs {
        source_path: {
            let mut source_path = swim_toml.clone();
            source_path.pop();
            source_path.push("build/spade.sv");

            // TODO: parse spade file directory and remove this
            if !source_path.is_file() {
                return syn::Error::new_spanned(
                    args.source_path,
                    "Please run swim build or similar",
                )
                .into_compile_error()
                .into();
            }

            syn::LitStr::new(source_path.as_str(), args.source_path.span())
        },
        name: args.name,
        clock_port: args.clock_port,
        reset_port: args.reset_port,
    };

    build_verilated_struct(verilog_args, item.into(), "spade").into()
}
