use heck::ToShoutySnakeCase;
use heck::ToTitleCase;
use heck::ToUpperCamelCase;
use proc_macro::TokenStream;
use proc_macro2::Span as Span2;
use proc_macro_error2::abort;
use proc_macro_error2::emit_error;
use proc_macro_error2::proc_macro_error;
use quote::quote;
use syn::parse::Parse;
use syn::parse_macro_input;
use syn::parse_str;
use syn::spanned::Spanned;
use syn::Ident;

mod kv_attributes;
use kv_attributes::*;

mod instruction_implementation;
use instruction_implementation::*;

struct EngineDefinition {
    state_struct: syn::ItemStruct,
    instruction_implementations: Option<InstructionsImpl>,
}

impl Parse for EngineDefinition {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let state_struct = input.parse()?;

        let instruction_implementations = if input.is_empty() {
            None
        } else {
            Some(input.parse()?)
        };

        let r = Self {
            state_struct,
            instruction_implementations,
        };

        if !input.is_empty() {
            emit_error!(input.span(), "An engine implementation should only include a `struct` definition, followed by an `impl` block.");
        }

        Ok(r)
    }
}

#[proc_macro_error]
#[proc_macro]
pub fn engine(stream: TokenStream) -> TokenStream {
    let EngineDefinition {
        mut state_struct,
        instruction_implementations,
    } = parse_macro_input!(stream as EngineDefinition);

    let engine_ref = Ident::new(
        &format!(
            "{}_ENGINE",
            state_struct.ident.to_string().to_shouty_snake_case()
        ),
        Span2::call_site(),
    );
    let mut name = parse_str(&format!(
        r#""{}""#,
        state_struct.ident.to_string().to_title_case()
    ))
    .unwrap();
    let mut lua_name = parse_str(&format!(
        r#""{}""#,
        state_struct.ident.to_string().to_upper_camel_case()
    ))
    .unwrap();
    let mut version = None;

    let mut engine_attribute_span = None;

    // Validate struct attributes
    for attr in &state_struct.attrs {
        if attr.path().is_ident("engine") {
            engine_attribute_span = Some(attr.meta.path().span());
            if let Some(vars) = parse_as_kv_attr("engine", attr) {
                for var in vars.vars {
                    match var.key.to_string().as_str() {
                        "name" => name = var.value,
                        "lua_name" => lua_name = var.value,
                        "version" => version = Some(var.value),
                        _ => emit_error!(
                            var.key.span(),
                            "Invalid key, expecting 'name', 'lua_name' or 'version'."
                        ),
                    }
                }
            }
        }
    }

    // Remove engine attribute from output
    state_struct.attrs = state_struct
        .attrs
        .iter()
        .filter(|attr| !attr.path().is_ident("engine"))
        .cloned()
        .collect();

    // Get ident of state struct
    let state_struct_ident = state_struct.ident.clone();

    // Process version
    if version.is_none() {
        let span = engine_attribute_span.unwrap_or(state_struct.span());
        abort!(
            span,
            r#"A version must be specified: #[engine(version = env!("CARGO_PKG_VERSION"))]"#
        );
    }
    let version = version.unwrap();

    // Convert the implementation to instructions
    let instrucs = instruction_implementations.map(|impl_| impl_.to_tokens(&state_struct_ident));

    quote! {
        #[derive(Default)]
        #state_struct

        ::testangel_engine::lazy_static! {
            static ref #engine_ref: ::std::sync::Mutex<::testangel_engine::Engine<'static, #state_struct_ident>> = {
                let mut engine = ::testangel_engine::Engine::<#state_struct_ident>::new(#name, #lua_name, #version);
                #instrucs
                ::std::sync::Mutex::new(engine)
            };
        }

        ::testangel_engine::plugin_impl! {
            ::testangel_engine::EngineInterface,

            unsafe fn ta_call(input: *const ::testangel_engine::c_char) -> *const ::testangel_engine::c_char {
                let input = ::std::ffi::CStr::from_ptr(input);
                let request = ::std::string::String::from_utf8_lossy(input.to_bytes()).to_string();
                let response = match ::testangel_engine::Request::try_from(request) {
                    Err(e) => ::testangel_engine::Response::Error {
                        kind: ::testangel_engine::ErrorKind::FailedToParseIPCJson,
                        reason: format!("The IPC message was invalid. ({:?})", e),
                    }
                    .to_json(),
                    Ok(request) => #engine_ref.lock().unwrap().process_request(request).to_json(),
                };
                let c_response = ::std::ffi::CString::new(response).expect("valid response");
                c_response.into_raw()
            }

            unsafe fn ta_release(input: *mut ::std::ffi::c_char) {
                if !input.is_null() {
                    #[allow(clippy::not_unsafe_ptr_arg_deref)]
                    drop(::std::ffi::CString::from_raw(input));
                }
            }
        }
    }
    .into()
}
