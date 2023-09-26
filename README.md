# TestAngel

## Introduction

TestAngel makes automating tests easy by allowing you to make reusable actions that can be used as part of a bigger test process.

In TestAngel, you start off creating a Test Flow. This will be the instructions followed by TestAngel to complete your automation. This flow can then be built up of different actions, which can be provided from two sources. Actions can either come directly from engines, which can perform low-level tasks with systems, for example the HTTP engine can make HTTP requests. Alternatively, actions can come from custom-made complex actions. These are pre-defined flow-within-a-flows that allow complex, repetitive behaviour to be abstracted into it's own self-contained action.

## Parts

| Part | Description |
|:-----|:------------|
|`testangel`|The main executable and UI that controls the platform ("the controller").|
|`testangel-ipc`|The library that contains the serialisable messages that can be exchanged between the controller and the engine plugins.|
|`testangel-arithmetic`|An arithmetic engine plugin.|
|`testangel-compare`|A comparison engine plugin.|
|`testangel-convert`|A conversion engine plugin.|
|`testangel-evidence`|An evidence-producing engine plugin.|
|`testangel-regex`|A regular expression processing engine plugin.|
|`testangel-user-interaction`|A user interaction engine plugin.|

## Other Engines

You can install new engines simply by downloading them and dropping them in the `engines` folder.

| Other engine | Description |
|:-----|:------------|
|[`testangel-sap`](https://github.com/lilopkins/testangel-sap)|An engine that interfaces with SAP GUI for Windows.|

## Environment Variables

The tool can be configured through a number of environment variables:

| Environment Variable | Description |
|:---------------------|:------------|
| `TA_ENGINE_DIR`      | The directory that should be searched through to locate TestAngel engines. By default, `./engines` is used. |
| `TA_ACTION_DIR`      | The directory that should be searched through to locate TestAngel actions. By default, `./actions` is used. |
| `TA_FLOW_DIR`        | The directory that should be suggested to save flows in. |
| `TA_SHOW_HIDDEN_ACTIONS` | If set to `yes`, actions will be shown in the flow editor even if set to hidden. |
| `TA_HIDE_ACTION_EDITOR` | If set to anything other than `no`, the action editor items on the Getting Started screen will be hidden. This can be useful in commercial settings as the action editor is more complex to learn and master. |
| `TA_LOCAL_SUPPORT_CONTACT` | If set, the Getting Started screen will show the value as a contact for obtaining help. Useful for commercial settings. |
| `TA_SKIP_VERSION_CHECK` | Skip checking if the latest version is installed. |

## Developers: Writing an Engine

If you are a developer in a language with C compatible FFI, you can write an engine. The following provides some details about how it can be achieved.

### Engine Communication

Engines are dynamically linked libraries (`.dll`s on Windows, `.dylib`s on Mac, `.so`s on Linux systems) which have two functions, `ta_call` and `ta_release`.
The call function has the signature:
```c
char* ta_call(char* input);
void ta_release(char* target);
```
These strings are JSON formatted and comply with the schema defined in `testangel-ipc`. `ta_release` is called with the string to release the memory.

In Rust, you can implement this communication like this:

```rust
#[no_mangle]
pub unsafe extern "C" fn ta_call(input: *const c_char) -> *mut c_char {
    let input = CStr::from_ptr(input);
    let response = call_internal(String::from_utf8_lossy(input.to_bytes()).to_string());
    let c_response = CString::new(response).expect("valid response");
    c_response.into_raw()
}

#[no_mangle]
pub unsafe extern "C" fn ta_release(input: *mut c_char) {
    if !input.is_null() {
        drop(CString::from_raw(input));
    }
}

fn call_internal(request: String) -> String {
    // Handle parsing request, processing and building a response.
    todo!()
}
```

You can generate up-to-date JSON schemas by cloning this repository and running:
```
cargo run -p testangel-ipc --features schemas
```
