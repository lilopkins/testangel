use proc_macro::TokenStream;

#[proc_macro]
pub fn expose_engine(stream: TokenStream) -> TokenStream {
    let mut stream_it = stream.into_iter();
    let name_token = stream_it
        .next()
        .expect("You must specify the name of the engine to expose!");
    if stream_it.next().is_some() {
        panic!("You must only specify one engine identifier to expose");
    }
    let engine_name = match name_token {
        proc_macro::TokenTree::Ident(val) => val.to_string(),
        _ => panic!("You must specify an identifier to expose!"),
    };
    format!(r#"
        #[no_mangle]
        pub unsafe extern "C" fn ta_call(input: *const ::std::ffi::c_char) -> *mut ::std::ffi::c_char {{
            let input = ::std::ffi::CStr::from_ptr(input);
            let request = String::from_utf8_lossy(input.to_bytes()).to_string();
            let response = match Request::try_from(request) {{
                Err(e) => Response::Error {{
                    kind: ErrorKind::FailedToParseIPCJson,
                    reason: format!("The IPC message was invalid. ({{:?}})", e),
                }}
                .to_json(),
                Ok(request) => {engine_name}.lock().expect("must be able to lock engine").process_request(request).to_json(),
            }};
            let c_response = ::std::ffi::CString::new(response).expect("valid response");
            c_response.into_raw()
        }}

        #[no_mangle]
        pub unsafe extern "C" fn ta_release(input: *mut ::std::ffi::c_char) {{
            if !input.is_null() {{
                drop(::std::ffi::CString::from_raw(input));
            }}
        }}
    "#).parse().unwrap()
}
