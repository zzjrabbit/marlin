// Copyright (C) 2024 Ethan Uppal.
//
// This project is free software: you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License as published by the Free
// Software Foundation, version 3 of the License only.
//
// This project is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS
// FOR A PARTICULAR PURPOSE. See the GNU Lesser General Public License for more
// details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with this project. If not, see <https://www.gnu.org/licenses/>.

use proc_macro::TokenStream;

use verilog_macro_builder::{build_verilated_struct, MacroArgs};

#[proc_macro_attribute]
pub fn verilog(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(args as MacroArgs);
    build_verilated_struct(args, item.into(), "verilog").into()
}
