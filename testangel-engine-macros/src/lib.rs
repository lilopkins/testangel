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
use syn::Expr;
use syn::Ident;

mod kv_attributes;
use kv_attributes::*;

mod instruction_implementation;
use instruction_implementation::*;
use syn::Lit;
use syn::Meta;

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
    let mut description = String::new();

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
        } else if attr.path().is_ident("doc") {
            if let Meta::NameValue(name_val) = &attr.meta {
                if let Expr::Lit(lit) = &name_val.value {
                    if let Lit::Str(s) = &lit.lit {
                        description = s.value().trim().to_string();
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
                let mut engine = ::testangel_engine::Engine::<#state_struct_ident>::new(#name, #lua_name, #version, #description);
                #instrucs
                ::std::sync::Mutex::new(engine)
            };
        }

        ::testangel_engine::plugin_impl! {
            ::testangel_engine::EngineInterface,

            unsafe fn ta_request_instructions(
                p_output_engine_metadata: *mut ::testangel_engine::ta_engine_metadata,
                parp_output_instructions: *mut *const *const ::testangel_engine::ta_instruction_metadata,
            ) -> *mut ::testangel_engine::ta_result {
                use ::testangel_engine::{ta_result, ta_result_code};

                (*p_output_engine_metadata).iSupportsIpcVersion = 1;

                let name = ::std::ffi::CString::new(#engine_ref.lock().unwrap().name().as_str())
                    .expect("Nul bytes in the engine name");
                (*p_output_engine_metadata).szFriendlyName = name.as_ptr();
                ::std::mem::forget(name.as_ptr());

                let version = ::std::ffi::CString::new(#engine_ref.lock().unwrap().version().as_str())
                    .expect("Nul bytes in the engine version");
                (*p_output_engine_metadata).szVersion = version.as_ptr();
                ::std::mem::forget(version.as_ptr());

                let lua_name = ::std::ffi::CString::new(#engine_ref.lock().unwrap().lua_name().as_str())
                    .expect("Nul bytes in the engine lua name");
                (*p_output_engine_metadata).szLuaName = lua_name.as_ptr();
                ::std::mem::forget(lua_name.as_ptr());

                let description = ::std::ffi::CString::new(#engine_ref.lock().unwrap().description().as_str())
                    .expect("Nul bytes in the engine description");
                (*p_output_engine_metadata).szDescription = description.as_ptr();
                ::std::mem::forget(description.as_ptr());

                // TODO Present instructions

                let r: *mut ta_result = &mut ta_result {
                    code: ta_result_code::TESTANGEL_OK,
                    szReason: ::std::ptr::null(),
                };
                ::std::mem::forget(r);
                r
            }

            fn ta_execute(
                sz_instruction_id: *const ::std::ffi::c_char,
                arp_parameter_list: *const *const ::testangel_engine::ta_named_value,
                n_parameter_count: u32,
                parp_output_list: *mut *mut *mut ::testangel_engine::ta_named_value,
                parp_output_evidence_list: *mut *mut *mut ::testangel_engine::ta_evidence,
            ) -> *mut ::testangel_engine::ta_result {
                todo!()
            }

            fn ta_reset_state() -> *mut ::testangel_engine::ta_result {
                use ::testangel_engine::{ta_result, ta_result_code};

                #engine_ref.lock().unwrap().reset_state();

                let r: *mut ta_result = &mut ta_result {
                    code: ta_result_code::TESTANGEL_OK,
                    szReason: ::std::ptr::null(),
                };
                ::std::mem::forget(r);
                r
            }

            unsafe fn ta_free_result(p_target: *const ::testangel_engine::ta_result) {
                if !p_target.is_null() {
                    let _ = *p_target;
                }
            }

            fn ta_free_engine_metadata(p_target: *const ::testangel_engine::ta_engine_metadata) {
                todo!()
            }

            fn ta_free_instruction_metadata_array(arp_target: *const *const ::testangel_engine::ta_instruction_metadata) {
                todo!()
            }

            fn ta_free_named_value_array(arp_target: *const *const ::testangel_engine::ta_named_value) {
                todo!()
            }

            fn ta_free_evidence_array(arp_target: *const *const ::testangel_engine::ta_evidence) {
                todo!()
            }
        }
    }
    .into()
}
