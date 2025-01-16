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

use std::{collections::HashMap, env, path::PathBuf};

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use sv_parser::{self as sv, unwrap_node, Locate, RefNode};

struct MacroArgs {
    source_path: syn::LitStr,
    name: syn::LitStr,

    clock_port: Option<syn::Ident>,
    reset_port: Option<syn::Ident>,
}

impl syn::parse::Parse for MacroArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        syn::custom_keyword!(src);
        syn::custom_keyword!(name);

        input.parse::<src>()?;
        input.parse::<syn::Token![=]>()?;
        let source_path = input.parse::<syn::LitStr>()?;

        input.parse::<syn::Token![,]>()?;

        input.parse::<name>()?;
        input.parse::<syn::Token![=]>()?;
        let name = input.parse::<syn::LitStr>()?;

        Ok(Self {
            source_path,
            name,
            clock_port: None,
            reset_port: None,
        })
    }
}

#[proc_macro_attribute]
pub fn verilog(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(args as MacroArgs);
    let item = syn::parse_macro_input!(item as syn::ItemStruct);

    let manifest_directory = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("Please compile using `cargo` or set the `CARGO_MANIFEST_DIR` environment variable"));
    let source_path = manifest_directory.join(args.source_path.value());

    //let source_bytes = match fs::read(source_path) {
    //    Ok(bytes) => bytes,
    //    Err(error) => {
    //    }
    //};
    //let source_contents = match String::from_utf8(source_bytes) {
    //
    //};

    let defines = HashMap::new();
    let (ast, _) =
        match sv::parse_sv(source_path, &defines, &["."], false, false) {
            Ok(result) => result,
            Err(error) => {
                return syn::Error::new_spanned(
                    args.source_path,
                    error.to_string(),
                )
                .into_compile_error()
                .into();
            }
        };

    let Some(module) = (&ast).into_iter().find_map(|node| match node {
        RefNode::ModuleDeclarationAnsi(module) => {
            // taken from https://github.com/dalance/sv-parser/blob/master/README.md
            fn get_identifier(node: RefNode) -> Option<Locate> {
                match unwrap_node!(node, SimpleIdentifier, EscapedIdentifier) {
                    Some(RefNode::SimpleIdentifier(x)) => Some(x.nodes.0),
                    Some(RefNode::EscapedIdentifier(x)) => Some(x.nodes.0),
                    _ => None,
                }
            }

            let id = unwrap_node!(module, ModuleIdentifier).unwrap();
            let id = get_identifier(id).unwrap();
            let id = ast.get_str(&id).unwrap();
            if id == args.name.value().as_str() {
                Some(module)
            } else {
                None
            }
        }
        _ => None,
    }) else {
        return syn::Error::new_spanned(
            &args.name,
            format!(
                "Could not find module declaration for `{}` in {}",
                args.name.value(),
                args.source_path.value()
            ),
        )
        .into_compile_error()
        .into();
    };

    let Some(port_declarations_list) = module
        .nodes
        .0
        .nodes
        .6
        .as_ref()
        .and_then(|list| list.nodes.0.nodes.1.as_ref())
    else {
        return syn::Error::new_spanned(
            &args.name,
            format!(
                "Module `{}` is missing a list of ports",
                args.name.value()
            ),
        )
        .into_compile_error()
        .into();
    };

    let mut struct_members = vec![];
    let mut preeval_impl = vec![];
    let mut posteval_impl = vec![];
    let mut other_impl = vec![];
    for (_, port) in port_declarations_list.contents() {
        match port {
            sv::AnsiPortDeclaration::Net(net) => {
                let port_name = ast.get_str_trim(&net.nodes.1.nodes.0).expect(
                    "Port identifier could not be traced back to source code",
                );

                let Some(port_direction )= net.nodes.0.as_ref().and_then(|maybe_net_header| match maybe_net_header {
                    sv::NetPortHeaderOrInterfacePortHeader::NetPortHeader(net_port_header) => {
                        net_port_header.nodes.0.as_ref()
                    },
                    _ => todo!("Other port header")
                }) else {
                    return syn::Error::new_spanned(
                        args.source_path,
                        format!(
                            "Port `{}` has no supported direction (`input` or `output`)",
                            port_name
                        ),
                    )
                    .into_compile_error()
                    .into();
                };

                let port_dimensions = &net.nodes.2;
                let port_width = match port_dimensions.len() {
                    0 => 1,
                    1 => match &port_dimensions[0] {
                        sv::UnpackedDimension::Range(
                            unpacked_dimension_range,
                        ) => {
                            let range =
                                &unpacked_dimension_range.nodes.0.nodes.1.nodes;
                            evaluate_numeric_constant_expression(&ast, &range.0)
                                - evaluate_numeric_constant_expression(
                                    &ast, &range.2,
                                )
                        }
                        sv::UnpackedDimension::Expression(
                            unpacked_dimension_expression,
                        ) => todo!("Other type of dimension"),
                    },
                    _ => todo!("Don't support multidimensional ports yet"),
                };

                let port_type = if port_width <= 8 {
                    quote! { verilog::__reexports::verilator::types::CData }
                } else if port_width <= 16 {
                    quote! { verilog::__reexports::verilator::types::SData }
                } else if port_width <= 32 {
                    quote! { verilog::__reexports::verilator::types::IData }
                } else if port_width <= 64 {
                    quote! { verilog::__reexports::verilator::types::QData }
                } else {
                    return syn::Error::new_spanned(
                        args.source_path,
                        format!(
                            "Port `{}` is wider than supported right now",
                            port_name
                        ),
                    )
                    .into_compile_error()
                    .into();
                };

                let port_name_ident = format_ident!("{}", port_name);
                struct_members.push(quote! {
                    pub #port_name_ident: #port_type
                });

                match port_direction {
                    sv::PortDirection::Input(_) => {
                        let setter = format_ident!("pin_{}", port_name);
                        struct_members.push(quote! {
                            #setter: extern "C" fn(*mut verilog::__reexports::libc::c_void, #port_type)
                        });
                        preeval_impl.push(quote! {
                            (self.#setter)(self.model, self.#port_name_ident);
                        });

                        if let Some(clock_port) = &args.clock_port {
                            if clock_port.to_string().as_str() == port_name {
                                other_impl.push(quote! {
                                    pub fn tick(&mut self) {
                                        self.#port_name = 1 as _;
                                        self.eval();
                                        self.#port_name = 0 as _;
                                        self.eval();
                                    }
                                });
                            }
                        }
                    }
                    sv::PortDirection::Output(_) => {
                        let getter = format_ident!("read_{}", port_name);
                        struct_members.push(quote! {
                            #getter: extern "C" fn(*mut verilog::__reexports::libc::c_void) -> #port_type
                        });
                        posteval_impl.push(quote! {
                            self.#port_name_ident = (self.#getter)(self.model);
                        });
                    }
                    sv::PortDirection::Inout(keyword) => todo!(),
                    sv::PortDirection::Ref(keyword) => todo!(),
                }
            }
            _ => todo!("Other types of ports"),
        }
    }

    struct_members.push(quote! {
        drop_model: extern "C" fn(*mut verilog::__reexports::libc::c_void),
        eval_model: extern "C" fn(*mut verilog::__reexports::libc::c_void)
    });

    let struct_name = item.ident;
    quote! {
        struct #struct_name<'ctx> {
            #(#struct_members),*,
            #[doc = "# Safety\nThe Rust binding to the model will not outlive the dynamic library context (with lifetime `'ctx`) and is dropped when this struct is."]
            model: *mut verilog::__reexports::libc::c_void,
            _phantom: std::marker::PhantomData<&'ctx ()>
        }

        impl #struct_name<'_> {
            pub fn eval(&mut self) {
                #(#preeval_impl)*
                (self.eval_model)(self.model);
                #(#posteval_impl)*
            }

            #(#other_impl)*
        }

        impl<'ctx> std::ops::Drop for #struct_name<'ctx> {
            fn drop(&mut self) {
                (self.drop_model)(self.model);
                self.model = std::ptr::null_mut();
            }
        }
    }
    .into()
}

fn evaluate_numeric_constant_expression(
    ast: &sv::SyntaxTree,
    expression: &sv::ConstantExpression,
) -> usize {
    match expression {
        sv::ConstantExpression::ConstantPrimary(constant_primary) => {
            match &**constant_primary {
                sv::ConstantPrimary::PrimaryLiteral(primary_literal) => {
                    match &**primary_literal {
                        sv::PrimaryLiteral::Number(number) => match &**number {
                            sv::Number::IntegralNumber(integral_number) => {
                                match &**integral_number {
                                    sv::IntegralNumber::DecimalNumber(
                                        decimal_number,
                                    ) => match &**decimal_number {
                                        sv::DecimalNumber::UnsignedNumber(
                                            unsigned_number,
                                        ) => ast
                                            .get_str_trim(
                                                &unsigned_number.nodes.0,
                                            )
                                            .unwrap()
                                            .parse()
                                            .unwrap(),
                                        sv::DecimalNumber::BaseUnsigned(
                                            decimal_number_base_unsigned,
                                        ) => todo!(),
                                        sv::DecimalNumber::BaseXNumber(
                                            decimal_number_base_xnumber,
                                        ) => todo!(),
                                        sv::DecimalNumber::BaseZNumber(
                                            decimal_number_base_znumber,
                                        ) => todo!(),
                                    },
                                    sv::IntegralNumber::OctalNumber(
                                        octal_number,
                                    ) => todo!(),
                                    sv::IntegralNumber::BinaryNumber(
                                        binary_number,
                                    ) => todo!(),
                                    sv::IntegralNumber::HexNumber(
                                        hex_number,
                                    ) => todo!(),
                                }
                            }
                            sv::Number::RealNumber(real_number) => {
                                panic!("Real number")
                            }
                        },
                        _ => todo!("Other constant primary literals"),
                    }
                }
                _ => panic!("Not a number"),
            }
        }
        sv::ConstantExpression::Unary(constant_expression_unary) => {
            todo!("Constant unary expressions")
        }
        sv::ConstantExpression::Binary(constant_expression_binary) => {
            todo!("Constant binary expressions")
        }
        sv::ConstantExpression::Ternary(constant_expression_ternary) => {
            todo!("Constant ternary expressions")
        }
    }
}
