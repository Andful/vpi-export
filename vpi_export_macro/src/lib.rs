#![feature(proc_macro_quote)]

extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro2;
use quote::quote;
use syn::{
    parse_macro_input, punctuated::Punctuated, token::Comma, FnArg, ItemFn, LitByteStr, Pat,
    PatType, Signature,
};

fn arg_instantiation_impl(arg: &FnArg) -> proc_macro2::TokenStream {
    match arg {
        FnArg::Typed(PatType { pat, ty, .. }) => quote! {
            let handle = vpi_scan(args_iter);
            let #pat: #ty = vpi_export::FromVpiHandle::from_vpi_handle(handle);
        },
        _ => panic!("Only functions supported"),
    }
}

fn args_instantiation_impl(args: &Punctuated<FnArg, Comma>) -> proc_macro2::TokenStream {
    let args = args.iter().map(arg_instantiation_impl).collect::<Vec<_>>();
    quote! { #(#args)* }
}

fn args_impl(args: &Punctuated<FnArg, Comma>) -> proc_macro2::TokenStream {
    let result: Punctuated<Box<Pat>, Comma> = args
        .iter()
        .map(|e| match e {
            FnArg::Receiver(_) => panic!(),
            FnArg::Typed(e) => e.pat.clone(),
        })
        .collect();
    quote! { #result }
}

#[proc_macro_attribute]
pub fn vpi_export(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let funtion = parse_macro_input!(item as ItemFn);
    let ItemFn { sig, .. } = funtion.clone();
    let Signature {
        ident: fn_ident,
        inputs,
        ..
    } = sig;
    let fn_name = proc_macro2::Ident::new(
        &format!("__hidden_{}_register", fn_ident),
        proc_macro2::Span::call_site(),
    );
    let func_name_literal = LitByteStr::new(
        format!("${}\0", fn_ident).as_bytes(),
        proc_macro2::Span::call_site(),
    );
    let test_name = proc_macro2::Ident::new(
        &format!("__ASSIGN_{}__", fn_ident).to_uppercase(),
        proc_macro2::Span::call_site(),
    );
    let args = args_impl(&inputs);
    let args_instantiation = args_instantiation_impl(&inputs);

    let register_fm = quote! {
        #[vpi_export::ctor]
        static #test_name: () = {
            vpi_export::__FUNCTION_COLLECTIONS__.values.lock().unwrap().push(#fn_name);
        };
        pub fn #fn_name() {
            //nsafe extern "C" fn(arg1: *mut PLI_BYTE8) -> PLI_INT32
            use vpi_export::vpi_user::*;
            unsafe extern "C" fn wrapper(_user_data: *mut vpi_export::vpi_user::PLI_BYTE8) -> vpi_export::vpi_user::PLI_INT32 {
                let systfref = vpi_handle(vpiSysTfCall as i32, std::ptr::null_mut());
                let args_iter = vpi_iterate(vpiArgument as i32, systfref);
                #args_instantiation
                #fn_ident(#args);
                0
            }
            let func_name_ptr = #func_name_literal.as_ptr() as *mut i8;

            let mut task_data_p = s_vpi_systf_data {
                type_: vpiSysTask as i32,
                tfname: func_name_ptr,
                calltf: Some(wrapper),
                ..Default::default()
            };
            unsafe {
                vpi_register_systf(&mut task_data_p);
            }
        }
    };
    quote! {
        #register_fm
        #funtion
    }
    .into()
}
