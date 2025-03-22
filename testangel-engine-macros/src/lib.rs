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
                parp_output_instructions: *mut *mut *const ::testangel_engine::ta_instruction_metadata,
            ) -> *mut ::testangel_engine::ta_result {
                use ::testangel_engine::{ta_result, ta_result_code, ta_instruction_metadata, ta_instruction_named_kind, ta_parameter_kind, InstructionNamedKind, ParameterKind, malloc, strcpy};
                use ::std::boxed::Box;
                use ::std::ffi::{CString, c_char};
                use ::std::mem::{forget, size_of};
                use ::std::ptr;

                (*p_output_engine_metadata).iSupportsIpcVersion = 3;

                let engine = #engine_ref.lock().unwrap();

                let name = Box::new(CString::new(engine.name().as_str())
                    .expect("Nul bytes in the engine name"));
                (*p_output_engine_metadata).szFriendlyName = name.as_ptr();
                forget(name);

                let version = Box::new(CString::new(engine.version().as_str())
                    .expect("Nul bytes in the engine version"));
                (*p_output_engine_metadata).szVersion = version.as_ptr();
                forget(version);

                let lua_name = Box::new(CString::new(engine.lua_name().as_str())
                    .expect("Nul bytes in the engine lua name"));
                (*p_output_engine_metadata).szLuaName = lua_name.as_ptr();
                forget(lua_name);

                let description = Box::new(CString::new(engine.description().as_str())
                    .expect("Nul bytes in the engine description"));
                (*p_output_engine_metadata).szDescription = description.as_ptr();
                forget(description);

                // Present instructions
                let instruction_array: *mut *const ta_instruction_metadata =
                        malloc((engine.instructions().len() + 1) * size_of::<*mut ta_instruction_metadata>()).cast();
                *instruction_array.add(engine.instructions().len()) = ptr::null_mut();
                *parp_output_instructions = instruction_array;

                for (idx, instruction) in engine.instructions().iter().enumerate() {
                    // Metadata
                    let instMeta: *mut ta_instruction_metadata =
                        malloc(size_of::<ta_instruction_metadata>()).cast();
                    let id = CString::new(instruction.id().as_str())
                        .expect("Nul bytes in the instruction ID");
                    (*instMeta).szId = malloc(id.count_bytes()).cast::<c_char>();
                    strcpy((*instMeta).szId.cast_mut(), id.as_ptr());
                    let lua_name = CString::new(instruction.lua_name().as_str())
                        .expect("Nul bytes in the instruction lua name");
                    (*instMeta).szLuaName = malloc(lua_name.count_bytes()).cast::<c_char>();
                    strcpy((*instMeta).szLuaName.cast_mut(), lua_name.as_ptr());
                    let friendly_name = CString::new(instruction.friendly_name().as_str())
                        .expect("Nul bytes in the instruction name");
                    (*instMeta).szFriendlyName = malloc(friendly_name.count_bytes()).cast::<c_char>();
                    strcpy((*instMeta).szFriendlyName.cast_mut(), friendly_name.as_ptr());
                    let description = CString::new(instruction.description().as_str())
                        .expect("Nul bytes in the instruction description");
                    (*instMeta).szDescription = malloc(description.count_bytes()).cast::<c_char>();
                    strcpy((*instMeta).szDescription.cast_mut(), description.as_ptr());

                    // Parameters
                    let parameters_array: *mut *mut ta_instruction_named_kind = malloc((instruction.parameters().len() + 1) * size_of::<*mut ta_instruction_named_kind>()).cast();
                    *parameters_array.add(instruction.parameters().len()) = ptr::null_mut();
                    for (idx, InstructionNamedKind { id, friendly_name, kind }) in instruction.parameters().iter().enumerate() {
                        let param: *mut ta_instruction_named_kind = malloc(size_of::<ta_instruction_named_kind>()).cast();

                        let id = CString::new(id.as_str())
                            .expect("Nul bytes in the parameter id");
                        (*param).szId = malloc(id.count_bytes()).cast::<c_char>();
                        strcpy((*param).szId.cast_mut(), id.as_ptr());

                        let name = CString::new(friendly_name.as_str())
                            .expect("Nul bytes in the parameter name");
                        (*param).szName = malloc(name.count_bytes()).cast::<c_char>();
                        strcpy((*param).szName.cast_mut(), name.as_ptr());

                        (*param).kind = match kind {
                            ParameterKind::String => ta_parameter_kind::TA_PARAMETER_STRING,
                            ParameterKind::Integer => ta_parameter_kind::TA_PARAMETER_INTEGER,
                            ParameterKind::Decimal => ta_parameter_kind::TA_PARAMETER_DECIMAL,
                            ParameterKind::Boolean => ta_parameter_kind::TA_PARAMETER_BOOLEAN,
                        };

                        *parameters_array.add(idx) = param;
                    }
                    (*instMeta).arpParameterList = parameters_array;

                    // Outputs
                    let outputs_array: *mut *mut ta_instruction_named_kind = malloc((instruction.outputs().len() + 1) * size_of::<*mut ta_instruction_named_kind>()).cast();
                    *outputs_array.add(instruction.outputs().len()) = ptr::null_mut();
                    for (idx, InstructionNamedKind { id, friendly_name, kind }) in instruction.outputs().iter().enumerate() {
                        let param: *mut ta_instruction_named_kind = malloc(size_of::<ta_instruction_named_kind>()).cast();

                        let id = CString::new(id.as_str())
                            .expect("Nul bytes in the output id");
                        (*param).szId = malloc(id.count_bytes()).cast::<c_char>();
                        strcpy((*param).szId.cast_mut(), id.as_ptr());

                        let name = CString::new(friendly_name.as_str())
                            .expect("Nul bytes in the output name");
                        (*param).szName = malloc(name.count_bytes()).cast::<c_char>();
                        strcpy((*param).szName.cast_mut(), name.as_ptr());

                        (*param).kind = match kind {
                            ParameterKind::String => ta_parameter_kind::TA_PARAMETER_STRING,
                            ParameterKind::Integer => ta_parameter_kind::TA_PARAMETER_INTEGER,
                            ParameterKind::Decimal => ta_parameter_kind::TA_PARAMETER_DECIMAL,
                            ParameterKind::Boolean => ta_parameter_kind::TA_PARAMETER_BOOLEAN,
                        };

                        *outputs_array.add(idx) = param;
                    }
                    (*instMeta).arpOutputList = outputs_array;

                    *instruction_array.add(idx) = instMeta;
                }

                Box::into_raw(Box::new(ta_result {
                    code: ta_result_code::TESTANGEL_OK,
                    szReason: ptr::null(),
                }))
            }

            unsafe fn ta_execute(
                sz_instruction_id: *const ::std::ffi::c_char,
                arp_parameter_list: *const *const ::testangel_engine::ta_named_value,
                n_parameter_count: u32,
                b_dry_run: bool,
                parp_output_list: *mut *mut *mut ::testangel_engine::ta_named_value,
                parp_output_evidence_list: *mut *mut *mut ::testangel_engine::ta_evidence,
            ) -> *mut ::testangel_engine::ta_result {
                use ::testangel_engine::{ta_result, ta_result_code, ta_parameter_kind, ta_evidence, ta_evidence_kind, ta_named_value, ta_inner_value, ParameterKind, ParameterValue, EvidenceContent, ErrorKind, OutputMap, EvidenceList, malloc, strcpy};
                use ::std::boxed::Box;
                use ::std::ptr;
                use ::std::ffi::{CStr, CString, c_char};
                use ::std::mem::size_of;

                let instruction_id = {
                    let cstr = CStr::from_ptr(sz_instruction_id);
                    let str_slice = cstr.to_str().unwrap();
                    str_slice.to_owned()
                };

                // Match correct instruction
                let mut instruction_match = None;
                {
                    let engine = #engine_ref.lock().unwrap();
                    for instruction in engine.instructions() {
                        if *instruction.id() == instruction_id {
                            instruction_match = Some(instruction.clone());
                            break;
                        }
                    }
                }

                if let Some(instruction) = instruction_match {
                    // Convert parameters
                    let mut iwp = ::testangel_engine::InstructionWithParameters {
                        instruction: instruction_id.clone(),
                        dry_run: b_dry_run,
                        parameters: ::std::collections::HashMap::new(),
                    };
                    for idx in 0..n_parameter_count {
                        let param = *arp_parameter_list.add(usize::try_from(idx).unwrap());
                        let name = {
                            let cstr = CStr::from_ptr((*param).szName);
                            let str_slice = cstr.to_str().unwrap();
                            str_slice.to_owned()
                        };
                        iwp.parameters.insert(name, match (*param).value.kind {
                            ta_parameter_kind::TA_PARAMETER_STRING => {
                                let val = {
                                    let cstr = CStr::from_ptr((*param).value.value.szValue);
                                    let str_slice = cstr.to_str().unwrap();
                                    str_slice.to_owned()
                                };
                                ParameterValue::String(val)
                            },
                            ta_parameter_kind::TA_PARAMETER_INTEGER => ParameterValue::Integer(*(*param).value.value.iValue),
                            ta_parameter_kind::TA_PARAMETER_DECIMAL => ParameterValue::Decimal(*(*param).value.value.fValue),
                            ta_parameter_kind::TA_PARAMETER_BOOLEAN => ParameterValue::Boolean(*(*param).value.value.bValue),
                        });
                    }

                    // Validate parameters
                    if let Err((kind, reason)) = instruction.validate(&iwp)
                    {
                        let reason = CString::new(reason.as_str()).unwrap();
                        let sz_reason = malloc(reason.count_bytes()).cast::<c_char>();
                        strcpy(sz_reason, reason.as_ptr());
                        return Box::into_raw(Box::new(ta_result {
                            code: match kind {
                                ErrorKind::InvalidInstruction => ta_result_code::TESTANGEL_ERROR_INVALID_INSTRUCTION,
                                ErrorKind::MissingParameter => ta_result_code::TESTANGEL_ERROR_MISSING_PARAMETER,
                                ErrorKind::InvalidParameterType => ta_result_code::TESTANGEL_ERROR_INVALID_PARAMETER_TYPE,
                                ErrorKind::EngineProcessingError => ta_result_code::TESTANGEL_ERROR_ENGINE_PROCESSING,
                            },
                            szReason: sz_reason,
                        }));
                    }

                    // Trigger function
                    let instruction_result = #engine_ref.lock().unwrap().run_instruction(iwp);
                    if let Err(e) = instruction_result {
                        let reason = CString::new(e.as_str()).unwrap();
                        let sz_reason = malloc(reason.count_bytes()).cast::<c_char>();
                        strcpy(sz_reason, reason.as_ptr());
                        return Box::into_raw(Box::new(ta_result {
                            code: ta_result_code::TESTANGEL_ERROR_ENGINE_PROCESSING,
                            szReason: sz_reason,
                        }));
                    }
                    let (output, evidence) = instruction_result.unwrap();

                    // Convert output
                    let output_array: *mut *mut ta_named_value = malloc((output.len() + 1) + size_of::<*mut ta_named_value>()).cast();
                    *parp_output_list = output_array;
                    *output_array.add(output.len()) = ptr::null_mut();
                    for (idx, (id, value)) in output.iter().enumerate() {
                        let named_val: *mut ta_named_value = malloc(size_of::<ta_named_value>()).cast();

                        let name = CString::new(id.as_str()).unwrap();
                        (*named_val).szName = malloc(name.count_bytes()).cast::<c_char>();
                        strcpy((*named_val).szName.cast_mut(), name.as_ptr());

                        (*named_val).value.kind = match value.kind() {
                            ParameterKind::String => ta_parameter_kind::TA_PARAMETER_STRING,
                            ParameterKind::Boolean => ta_parameter_kind::TA_PARAMETER_BOOLEAN,
                            ParameterKind::Integer => ta_parameter_kind::TA_PARAMETER_INTEGER,
                            ParameterKind::Decimal => ta_parameter_kind::TA_PARAMETER_DECIMAL,
                        };

                        let p_value: ta_inner_value = match value.kind() {
                            ParameterKind::Boolean => {
                                let val = Box::new(value.value_bool());
                                let val = Box::into_raw(val);
                                ta_inner_value { bValue: val }
                            }
                            ParameterKind::Decimal => {
                                let val = Box::new(value.value_f64());
                                let val = Box::into_raw(val);
                                ta_inner_value { fValue: val }
                            }
                            ParameterKind::Integer => {
                                let val = Box::new(value.value_i32());
                                let val = Box::into_raw(val);
                                ta_inner_value { iValue: val }
                            }
                            ParameterKind::String => {
                                let val = value.value_string();
                                let val = CString::new(val.as_str()).unwrap();
                                let p_val = malloc(val.count_bytes()).cast::<c_char>();
                                strcpy(p_val, val.as_ptr());
                                ta_inner_value { szValue: p_val }
                            }
                        };
                        (*named_val).value.value = p_value;

                        *output_array.add(idx) = named_val;
                    }

                    // Convert evidence
                    let evidence_array: *mut *mut ta_evidence = malloc((evidence.len() + 1) + size_of::<*mut ta_evidence>()).cast();
                    *parp_output_evidence_list = evidence_array;
                    *evidence_array.add(evidence.len()) = ptr::null_mut();
                    for (idx, ev) in evidence.iter().enumerate() {
                        let evidence: *mut ta_evidence = malloc(size_of::<ta_evidence>()).cast();

                        let name = CString::new(ev.label.as_str()).unwrap();
                        (*evidence).szLabel = malloc(name.count_bytes()).cast::<c_char>();
                        strcpy((*evidence).szLabel.cast_mut(), name.as_ptr());

                        (*evidence).kind = match &ev.content {
                            EvidenceContent::Textual(_) => ta_evidence_kind::TA_EVIDENCE_TEXTUAL,
                            EvidenceContent::ImageAsPngBase64(_) => ta_evidence_kind::TA_EVIDENCE_PNGBASE64,
                        };

                        match &ev.content {
                            EvidenceContent::Textual(txt) | EvidenceContent::ImageAsPngBase64(txt) => {
                                let data = CString::new(txt.as_str()).unwrap();
                                (*evidence).value = malloc(data.count_bytes()).cast::<c_char>();
                                strcpy((*evidence).value.cast_mut(), data.as_ptr());
                            },
                        }

                        *evidence_array.add(idx) = evidence;
                    }

                    return Box::into_raw(Box::new(ta_result {
                        code: ta_result_code::TESTANGEL_OK,
                        szReason: ptr::null(),
                    }));
                } else {
                    let reason = CString::new("This engine does not know about the requested instruction.").unwrap();
                    let sz_reason = malloc(reason.count_bytes()).cast::<c_char>();
                    strcpy(sz_reason, reason.as_ptr());
                    return Box::into_raw(Box::new(ta_result {
                        code: ta_result_code::TESTANGEL_ERROR_INVALID_INSTRUCTION,
                        szReason: sz_reason,
                    }));
                }
            }

            fn ta_reset_state() -> *mut ::testangel_engine::ta_result {
                use ::testangel_engine::{ta_result, ta_result_code};
                use ::std::boxed::Box;
                use ::std::ptr;

                #engine_ref.lock().unwrap().reset_state();

                Box::into_raw(Box::new(ta_result {
                    code: ta_result_code::TESTANGEL_OK,
                    szReason: ptr::null(),
                }))
            }

            unsafe fn ta_free_result(p_target: *const ::testangel_engine::ta_result) {
                use ::std::boxed::Box;

                if !p_target.is_null() {
                    let res = Box::from_raw(p_target.cast_mut());
                    if !res.szReason.is_null() {
                        let _ = Box::from_raw(res.szReason.cast_mut());
                    }
                }
            }

            unsafe fn ta_free_engine_metadata(p_target: *const ::testangel_engine::ta_engine_metadata) {
                use ::testangel_engine::free;
                use ::std::ffi::c_void;

                free((*p_target).szFriendlyName.cast::<c_void>().cast_mut());
                free((*p_target).szVersion.cast::<c_void>().cast_mut());
                free((*p_target).szLuaName.cast::<c_void>().cast_mut());
                free((*p_target).szDescription.cast::<c_void>().cast_mut());
            }

            unsafe fn ta_free_instruction_metadata_array(arp_target: *const *const ::testangel_engine::ta_instruction_metadata) {
                use ::testangel_engine::free;
                use ::std::ffi::c_void;

                let mut i = 0;
                loop {
                    let meta_raw = *arp_target.add(i);
                    if meta_raw.is_null() {
                        break;
                    }

                    let mut j = 0;
                    loop {
                        let param = *(*meta_raw).arpParameterList.add(j);
                        if param.is_null() {
                            break;
                        }

                        free(param.cast::<c_void>());
                        j += 1;
                    }

                    let mut j = 0;
                    loop {
                        let output = *(*meta_raw).arpOutputList.add(j);
                        if output.is_null() {
                            break;
                        }

                        free(output.cast::<c_void>());
                        j += 1;
                    }

                    free(meta_raw.cast::<c_void>().cast_mut());

                    i += 1;
                }
                free(arp_target.cast::<c_void>().cast_mut());
            }

            unsafe fn ta_free_named_value_array(arp_target: *const *const ::testangel_engine::ta_named_value) {
                use ::testangel_engine::free;
                use ::std::ffi::c_void;

                let mut i = 0;
                loop {
                    let named_value_raw = *arp_target.add(i);
                    if named_value_raw.is_null() {
                        break;
                    }

                    free((*named_value_raw).szName.cast::<c_void>().cast_mut());
                    free((*named_value_raw).value.value.iValue.cast::<c_void>().cast_mut());
                    free(named_value_raw.cast::<c_void>().cast_mut());
                    i += 1;
                }
                free(arp_target.cast::<c_void>().cast_mut());
            }

            unsafe fn ta_free_evidence_array(arp_target: *const *const ::testangel_engine::ta_evidence) {
                use ::testangel_engine::free;
                use ::std::ffi::c_void;

                let mut i = 0;
                loop {
                    let evidence_raw = *arp_target.add(i);
                    if evidence_raw.is_null() {
                        break;
                    }

                    free((*evidence_raw).szLabel.cast::<c_void>().cast_mut());
                    free((*evidence_raw).value.cast::<c_void>().cast_mut());
                    free(evidence_raw.cast::<c_void>().cast_mut());
                    i += 1;
                }
                free(arp_target.cast::<c_void>().cast_mut());
            }
        }
    }
    .into()
}
