#![feature(proc_macro_quote)]

extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro2::{self, Span};
use quote::quote;
use syn::{
    parse_macro_input, punctuated::Punctuated, token::Comma, FnArg, Ident, ItemFn, LitByteStr,
    LitStr, PatType, Signature,
};

fn arg_initialization_impl(arg: &FnArg, index: usize) -> proc_macro2::TokenStream {
    let arg_ident = Ident::new(&format!("arg_{index}"), Span::call_site());
    match arg {
        FnArg::Typed(PatType { ty, .. }) => quote! {
            let #arg_ident = unsafe { <#ty as vpi_export::FromVpiHandle>::from_vpi_handle(args_iter.next().unwrap()) }?;
        },
        _ => panic!("Only functions supported"),
    }
}

fn port_arg_initialization_impl(arg: &FnArg, index: usize) -> proc_macro2::TokenStream {
    let arg_ident = Ident::new(&format!("arg_{index}"), Span::call_site());
    match arg {
        FnArg::Typed(PatType { ty, .. }) => quote! {
            let port = args_iter.next().expect("Arg is null");
            let name = unsafe {
                vpi_get_str(vpi_export::vpi_user::vpiName as i32, port.as_ptr())
            };
            let net = unsafe {
                vpi_export::vpi_user::vpi_handle_by_name(name, module.as_ptr())
            };
            let net = core::ptr::NonNull::new(net).expect("Low connection is null");
            let #arg_ident = unsafe { <#ty as vpi_export::FromVpiHandle>::from_vpi_handle(net) }?;
        },
        _ => panic!("Only functions supported"),
    }
}

fn args_impl(args: &Punctuated<FnArg, Comma>) -> proc_macro2::TokenStream {
    let result: Punctuated<Ident, Comma> = (0..args.len())
        .map(|i| Ident::new(&format!("arg_{i}"), Span::call_site()))
        .collect();
    quote! { #result }
}

/// Drive signals of a module
#[proc_macro_attribute]
pub fn vpi_top(_: TokenStream, item: TokenStream) -> TokenStream {
    let function = parse_macro_input!(item as ItemFn);
    let ItemFn { sig, .. } = function.clone();
    let Signature {
        ident: fn_ident,
        inputs,
        ..
    } = sig;
    let args = args_impl(&inputs);
    let args_initialization = inputs
        .iter()
        .enumerate()
        .map(|(i, e)| port_arg_initialization_impl(e, i))
        .collect::<Vec<_>>();

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
                VPI_FUNCTION_COLLECTION.push(unsafe {  &mut *::core::ptr::addr_of_mut!(VPI_FUNCTION_NODE) });
            }

            pub fn init() {
                let mut cb_data = t_cb_data {
                        reason: cbStartOfSimulation as i32,
                        cb_rtn: Some(raw_wrapper),
                        obj: core::ptr::null_mut(),
                        user_data: core::ptr::null_mut(),
                        ..Default::default()
                    };

                //SAFETY: correct usage of function
                unsafe {
                    vpi_export::vpi_user::vpi_register_cb(&mut cb_data);
                }
            }

            fn wrapper() -> vpi_export::Result<()> {
                let module = unsafe { vpi_export::VpiIter::new(vpiModule as PLI_INT32, core::ptr::null_mut()) }.next().unwrap();
                //Safety: systfref is not null or dangling
                let mut args_iter = unsafe { vpi_export::VpiIter::new(vpiPort as PLI_INT32, module.as_ptr()) };
                {
                    #(#args_initialization)*
                    let res = #fn_ident(#args);
                    vpi_export::VpiTaskResult::into_vpi_result(res)?
                }
                Ok(())
            }

            unsafe extern "C" fn raw_wrapper(_user_data: *mut vpi_export::vpi_user::t_cb_data) -> vpi_export::vpi_user::PLI_INT32 {
                wrapper().unwrap();
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

/// Drive signals of a module
#[proc_macro_attribute]
pub fn vpi_module(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = attr.to_string();
    let function = parse_macro_input!(item as ItemFn);
    let ItemFn { sig, .. } = function.clone();
    let Signature {
        ident: fn_ident,
        inputs,
        ..
    } = sig;
    let module_name_literal = LitByteStr::new(
        format!("{}\0", attr).as_bytes(),
        proc_macro2::Span::call_site(),
    );
    let args = args_impl(&inputs);
    let args_initialization = inputs
        .iter()
        .enumerate()
        .map(|(i, e)| port_arg_initialization_impl(e, i))
        .collect::<Vec<_>>();

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
                VPI_FUNCTION_COLLECTION.push(unsafe {  &mut *::core::ptr::addr_of_mut!(VPI_FUNCTION_NODE) });
            }

            pub fn init() {
                let module_name_ptr = #module_name_literal.as_ptr() as *mut ::core::ffi::c_char;
                let mut cb_data = t_cb_data {
                        reason: cbStartOfSimulation as i32,
                        cb_rtn: Some(raw_wrapper),
                        obj: core::ptr::null_mut(),
                        user_data: core::ptr::null_mut(),
                        ..Default::default()
                    };

                //SAFETY: correct usage of function
                unsafe {
                    vpi_export::vpi_user::vpi_register_cb(&mut cb_data);
                }
            }

            fn wrapper() -> vpi_export::Result<()> {
                let module = unsafe { vpi_handle_by_name(#module_name_literal.as_ptr() as *mut ::core::ffi::c_char, ::core::ptr::null_mut()) };
                let Some(module) = core::ptr::NonNull::new(module) else {
                    return Err(vpi_export::VpiError::NoModule(core::ffi::CStr::from_bytes_with_nul(#module_name_literal).unwrap()))
                };
                //Safety: systfref is not null or dangling
                let mut args_iter = unsafe { vpi_export::VpiIter::new(vpiPort as PLI_INT32, module.as_ptr()) };
                {
                    #(#args_initialization)*
                    let res = #fn_ident(#args);
                    vpi_export::VpiTaskResult::into_vpi_result(res)?
                }
                Ok(())
            }

            unsafe extern "C" fn raw_wrapper(_user_data: *mut vpi_export::vpi_user::t_cb_data) -> vpi_export::vpi_user::PLI_INT32 {
                wrapper().unwrap();
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
    let args_initialization = inputs
        .iter()
        .enumerate()
        .map(|(i, e)| arg_initialization_impl(e, i))
        .collect::<Vec<_>>();

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
                VPI_FUNCTION_COLLECTION.push(unsafe {  &mut *::core::ptr::addr_of_mut!(VPI_FUNCTION_NODE) });
            }

            pub fn init() {
                let func_name_ptr = #func_name_literal.as_ptr() as *mut ::core::ffi::c_char;
                let mut task_data_p = s_vpi_systf_data {
                    type_: vpiSysTask as PLI_INT32,
                    tfname: func_name_ptr,
                    calltf: Some(raw_wrapper),
                    ..Default::default()
                };
                //SAFETY: correct usage of function
                unsafe {
                    vpi_register_systf(&mut task_data_p);
                }
            }

            fn wrapper() -> vpi_export::Result<()> {
                let systfref = unsafe { vpi_handle(vpiSysTfCall as PLI_INT32, ::core::ptr::null_mut()) };
                //Safety: systfref is not null or dangling
                let mut args_iter = unsafe { vpi_export::VpiIter::new(vpiArgument as PLI_INT32, systfref) };
                {
                    #(#args_initialization)*
                    let res = #fn_ident(#args);
                    vpi_export::VpiTaskResult::into_vpi_result(res)?
                }
                Ok(())
            }

            unsafe extern "C" fn raw_wrapper(_user_data: *mut vpi_export::vpi_user::PLI_BYTE8) -> vpi_export::vpi_user::PLI_INT32 {
                wrapper().unwrap();
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
