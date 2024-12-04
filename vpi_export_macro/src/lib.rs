#![feature(proc_macro_quote)]

extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro2::{self, Span};
use quote::quote;
use syn::{
    parse_macro_input, punctuated::Punctuated, token::Comma, FnArg, Ident, ItemFn, LitByteStr,
    LitStr, PatType, Signature,
};

fn arg_instantiation_impl(arg: &FnArg, arg_ident: Ident) -> proc_macro2::TokenStream {
    match arg {
        FnArg::Typed(PatType { ty, .. }) => quote! {
            let handle = vpi_scan(args_iter);
            if handle == ::core::ptr::null_mut() {
                panic!("not enough arguments");
            }
            let #arg_ident: #ty = vpi_export::FromVpiHandle::from_vpi_handle(handle).unwrap();
        },
        _ => panic!("Only functions supported"),
    }
}

fn args_instantiation_impl(args: &Punctuated<FnArg, Comma>) -> proc_macro2::TokenStream {
    let args = args
        .iter()
        .enumerate()
        .map(|(i, e)| arg_instantiation_impl(e, Ident::new(&format!("arg_{i}"), Span::call_site())))
        .collect::<Vec<_>>();
    quote! { #(#args)* }
}

fn args_impl(args: &Punctuated<FnArg, Comma>) -> proc_macro2::TokenStream {
    let result: Punctuated<Ident, Comma> = (0..args.len())
        .map(|i| Ident::new(&format!("arg_{i}"), Span::call_site()))
        .collect();
    quote! { #result }
}

/// Export function as a vpi task
#[proc_macro_attribute]
pub fn vpi_task(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let function = parse_macro_input!(item as ItemFn);
    let ItemFn { sig, .. } = function.clone();
    let Signature {
        ident: fn_ident,
        inputs,
        ..
    } = sig;
    let func_name_literal = LitByteStr::new(
        format!("${}\0", fn_ident).as_bytes(),
        proc_macro2::Span::call_site(),
    );
    let args = args_impl(&inputs);
    let args_instantiation = args_instantiation_impl(&inputs);

    let register_fm = quote! {
        const _: () = {
            use ::vpi_export::__hidden__::{
                ctor, VpiFunctionNode, VPI_FUNCTION_COLLECTION,
            };
            use ::vpi_export::vpi_user::*;

            #[ctor]
            fn ctor() {
                static mut VPI_FUNCTION_NODE: VpiFunctionNode = VpiFunctionNode::new(init);
                //SAFETY: this ctor function is called only once
                unsafe {  VPI_FUNCTION_COLLECTION.push(::core::ptr::addr_of_mut!(VPI_FUNCTION_NODE)) };
            }

            pub fn init() {
                let func_name_ptr = #func_name_literal.as_ptr() as *mut ::core::ffi::c_char;
                let mut task_data_p = s_vpi_systf_data {
                    type_: vpiSysTask as PLI_INT32,
                    tfname: func_name_ptr,
                    calltf: Some(wrapper),
                    ..Default::default()
                };
                //SAFETY: correct usage of function
                unsafe {
                    vpi_register_systf(&mut task_data_p);
                }
            }

            unsafe extern "C" fn wrapper(_user_data: *mut vpi_export::vpi_user::PLI_BYTE8) -> vpi_export::vpi_user::PLI_INT32 {
                let systfref = vpi_handle(vpiSysTfCall as PLI_INT32, ::core::ptr::null_mut());
                let args_iter = vpi_iterate(vpiArgument as PLI_INT32, systfref);
                {
                    #args_instantiation
                    #fn_ident(#args);
                }
                if args_iter != ::core::ptr::null_mut(){
                    vpi_free_object(args_iter);
                }
                0
            }
        };
    };
    quote! {
        #register_fm
        #function
    }
    .into()
}

#[proc_macro]
pub fn bitvec(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as LitStr);
    let value = input.value();

    use regex::Regex;

    let re = Regex::new(r"^(\d*)'([b|d|o|h])([0-9|a|b|c|d|e|f]*)$").unwrap();
    let Some(caps) = re.captures(&value) else {
        panic!("Literal \"{}\" is not a valid verilog vector", value);
    };

    let len = caps[1].parse::<usize>().ok();
    let encoding = &caps[2];
    let data = &caps[3];

    let (bits, filled) = match encoding {
        "b" => {
            let mut bits = Vec::<u32>::new();
            let mut b = 0;
            let mut filled = 0;
            for c in data.chars() {
                filled += 1;
                if filled % 32 == 0 {
                    bits.push(b);
                    b = 0;
                }
                match c {
                    '1' => b = (b << 1) | 1,
                    '0' => b <<= 1,
                    _ => panic!("invalid input"),
                }
            }
            bits.push(b);
            (bits, filled)
        }
        "d" => todo!(),
        "o" => todo!(),
        "h" => todo!(),
        e => unreachable!("{e}"),
    };

    let len = len.unwrap_or(filled);

    // generate code, include `str_value` variable (automatically encodes
    // `String` as a string literal in the generated code)
    quote! {
        vpi_export::BitVector::<#len>::from_raw(&[#(#bits),*])
    }
    .into()
}
