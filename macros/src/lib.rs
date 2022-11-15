use proc_macro::TokenStream;
use syn::{parse_macro_input, AttributeArgs, ItemFn, Type};
use quote::quote;

macro_rules! error {
    ($($x:tt)*) => {
        return quote! {
            compile_error!($($x)*);
        }.into()
    };
}

#[proc_macro_attribute]
pub fn update(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    if !args.is_empty() {
        error!("Unexpected macro args");
    }
    let func = parse_macro_input!(input as ItemFn);
    let func_name = &func.sig.ident;

    let out = quote! {
        #[no_mangle]
        pub extern "C" fn update() {
            // The user-implemented update function
            #func
            fn shorten<'a, T: 'static, U>(x: &'static mut T, _y: &'a mut U) -> &'a mut T {
                x
            }
            unsafe {
                // there's a initialization flag at memory addres 0x0001
                // so that in the very unlikely case `start` isn't run first,
                // there won't be UB reading the user state
                if *(1 as *mut u8) != 1 {
                    return;
                }
                let state_v = &mut *(4 as *mut ::sw4::Wasm4 );
                let user_state_v = (&mut *(SW4_USER_STATE.get())).assume_init_mut();

                // The lifetimes have to be shortened, as giving the user a
                // 'static lifetime would allow them to store it between frames
                // which could break aliasing    
                let mut state = ();
                let state = shorten(state_v, &mut state);
                let mut user_state = ();
                let user_state = shorten(user_state_v, &mut user_state);
                (#func_name)(state, user_state)
            }
        }
    };

    out.into()
}

#[proc_macro_attribute]
pub fn start(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    if !args.is_empty() {
        error!("Unexpected macro args");
    }
    let func = parse_macro_input!(input as ItemFn);
    let func_name = &func.sig.ident;
    let user_data_type = match &func.sig.output {
        syn::ReturnType::Default => Type::Verbatim(quote!(())),
        syn::ReturnType::Type(_, ty) => (**ty).clone(),
    };

    let out = quote! {
        #[allow(deprecated)]
        static SW4_USER_STATE: sw4::SyncUnsafeCell<core::mem::MaybeUninit<#user_data_type>> = 
            sw4::SyncUnsafeCell::new(core::mem::MaybeUninit::uninit());
        #[no_mangle]
        pub extern "C" fn start() {
            // The user-implemented start function
            #func
            fn shorten<'a, T: 'static, U>(x: &'static mut T, _y: &'a mut U) -> &'a mut T {
                x
            }
            unsafe {
                let state_v = &mut *(4 as *mut ::sw4::Wasm4 );
                let mut state = ();
                let state = shorten(state_v, &mut state);
                SW4_USER_STATE.get().cast::<#user_data_type>().write((#func_name)(state));
                // Set an initialization flag at memory addres 0x0001
                (1 as *mut u8).write(1)
            }
        }
    };

    out.into()
}