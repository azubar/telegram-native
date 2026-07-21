# C++ to Rust Native API Component Transition Plan

This document outlines the design and implementation details for transitioning the C++ Native API component `TelegramNative` to Rust using the `native_api_1c` template.

## Objectives
- Replace the legacy C++ code with safe, modern, and idiomatic Rust.
- Maintain full compatibility with 1C:Enterprise platform (matching property and method signatures, English and Russian names).
- Interface safely with the C-linkage TDLib JSON client API.

---

## 1. Rust Crate Setup

We will create a new Rust library crate. The `Cargo.toml` will configure the build to produce a dynamic library (`cdylib`) and define required dependencies.

### Proposed `Cargo.toml`
```toml
[package]
name = "telegram_native"
version = "1.8.65"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
utf16_lit = "2.0"
native_api_1c = { path = "./native_api_1c" } # Reference the local version
libc = "0.2"
once_cell = "1.18"
```

---

## 2. Component State Mapping

The C++ class `TelegramNative` contains several member variables managing the TDLib client pointer, execution state, and async receiving thread.

### C++ Fields to Rust Struct Fields Mapping

| C++ Member Variable | C++ Type | Rust Field Type | Description / Handling |
| :--- | :--- | :--- | :--- |
| `connection` | `IAddInDefBase*` | `Arc<Option<&'static Connection>>` | 1C Connection handle decorated with `#[add_in_con]`. |
| `memory_manager` | `IMemoryManager*` | Handled by `native_api_1c` | Allocation for string and binary types is managed transparently. |
| `event_source_name` | `icu::UnicodeString` | `String` | Event source name. Defaults to `"TelegramNative"`. |
| `telegram_client` | `void*` | `*mut c_void` | Pointer to the active TDLib JSON client instance. |
| `async_mode` | `bool` | `std::sync::atomic::AtomicBool` | Flag indicating if async receive loop is running. |
| `rcv_thread` | `std::thread` | `Option<std::thread::JoinHandle<()>>` | Thread handle for the background receive loop. |
| `rcv_timeout` | `double` | `f64` | Timeout value passed to `td_json_client_receive`. Defaults to `1.0`. |

---

## 3. TDLib FFI Declarations in Rust

We will bind directly to the TDLib JSON client's export functions.

```rust
use std::os::raw::{c_char, c_double, c_int, c_void};

extern "C" {
    pub fn td_json_client_create() -> *mut c_void;
    pub fn td_json_client_destroy(client: *mut c_void);
    pub fn td_json_client_send(client: *mut c_void, request: *const c_char);
    pub fn td_json_client_receive(client: *mut c_void, timeout: c_double) -> *const c_char;
    pub fn td_json_client_execute(client: *mut c_void, request: *const c_char) -> *const c_char;
    
    pub fn td_set_log_file_path(file_path: *const c_char) -> c_int;
    pub fn td_set_log_max_file_size(max_file_size: i64);
    pub fn td_set_log_verbosity_level(log_verbosity_level: c_int);
}
```

---

## 4. Property Mapping

The component defines one property.

| English Name | Russian Name | Type | Read/Write | Description |
| :--- | :--- | :--- | :--- | :--- |
| `EventSourceName` | `ИмяИсточникаСобытий` | `String` | Read/Write | Custom name for the source of events sent to 1C. |

### Rust Implementation
```rust
#[add_in_prop(
    ty = Str,
    name = "EventSourceName",
    name_ru = "ИмяИсточникаСобытий",
    readable,
    writable
)]
pub event_source_name: String,
```

---

## 5. Methods Mapping

All component methods will be registered using `#[add_in_func(...)]` attributes.

### Method 1: `Send` / `Отправить`
- **Arguments**: `command: String`
- **Return Type**: `None`
- **C++ Behavior**: Call `td_json_client_send` with the request string.

```rust
#[add_in_func(name = "Send", name_ru = "Отправить")]
#[arg(ty = Str)]
pub fn send(&self, command: String) {
    let c_str = std::ffi::CString::new(command).unwrap_or_default();
    unsafe {
        td_json_client_send(self.telegram_client, c_str.as_ptr());
    }
}
```

### Method 2: `Receive` / `Получить`
- **Arguments**: None
- **Return Type**: `String`
- **C++ Behavior**: Call `td_json_client_receive` with `rcv_timeout` and return the result string.

```rust
#[add_in_func(name = "Receive", name_ru = "Получить")]
#[returns(ty = Str)]
pub fn receive(&self) -> String {
    unsafe {
        let ptr = td_json_client_receive(self.telegram_client, self.rcv_timeout);
        if ptr.is_null() {
            String::new()
        } else {
            std::ffi::CStr::from_ptr(ptr)
                .to_string_lossy()
                .into_owned()
        }
    }
}
```

### Method 3: `Execute` / `Выполнить`
- **Arguments**: `command: String`
- **Return Type**: `String`
- **C++ Behavior**: Call `td_json_client_execute` and return the response.

```rust
#[add_in_func(name = "Execute", name_ru = "Выполнить")]
#[arg(ty = Str)]
#[returns(ty = Str)]
pub fn execute(&self, command: String) -> String {
    let c_str = std::ffi::CString::new(command).unwrap_or_default();
    unsafe {
        let ptr = td_json_client_execute(self.telegram_client, c_str.as_ptr());
        if ptr.is_null() {
            String::new()
        } else {
            std::ffi::CStr::from_ptr(ptr)
                .to_string_lossy()
                .into_owned()
        }
    }
}
```

### Method 4: `SetAsyncMode` / `УстановитьАсинхронныйРежим`
- **Arguments**: `enable: bool`
- **Return Type**: `None`
- **C++ Behavior**: Start or stop the background receive thread `rcv_loop`.

```rust
#[add_in_func(name = "SetAsyncMode", name_ru = "УстановитьАсинхронныйРежим")]
#[arg(ty = Bool)]
pub fn set_async_mode(&mut self, enable: bool) {
    let current = self.async_mode.load(std::sync::atomic::Ordering::Relaxed);
    if current == enable {
        return;
    }
    self.async_mode.store(enable, std::sync::atomic::Ordering::Relaxed);
    
    if enable {
        let client = self.telegram_client;
        let timeout = self.rcv_timeout;
        let conn_arc = Arc::clone(&self.connection);
        let event_source = self.event_source_name.clone();
        let async_flag = Arc::clone(&self.async_mode_atomic); // Share atomic reference
        
        self.rcv_thread = Some(std::thread::spawn(move || {
            while async_flag.load(std::sync::atomic::Ordering::Relaxed) {
                unsafe {
                    let ptr = td_json_client_receive(client, timeout);
                    if !ptr.is_null() && !conn_arc.is_none() {
                        let response = std::ffi::CStr::from_ptr(ptr).to_string_lossy().into_owned();
                        if !response.is_empty() {
                            if let Some(conn) = *conn_arc {
                                conn.external_event(&event_source, "Response", &response);
                            }
                        }
                    }
                }
            }
        }));
    } else {
        if let Some(handle) = self.rcv_thread.take() {
            let _ = handle.join();
        }
    }
}
```

### Method 5: `SetLogFilePath` / `УстановитьФайлЖурнала`
- **Arguments**: `path: String`
- **Return Type**: `bool`
- **C++ Behavior**: Calls `td_set_log_file_path` and returns success/failure.

```rust
#[add_in_func(name = "SetLogFilePath", name_ru = "УстановитьФайлЖурнала")]
#[arg(ty = Str)]
#[returns(ty = Bool)]
pub fn set_log_file_path(&self, path: String) -> bool {
    let c_str = std::ffi::CString::new(path).unwrap_or_default();
    unsafe {
        td_set_log_file_path(c_str.as_ptr()) != 0
    }
}
```

### Method 6: `SetLogMaxFileSize` / `УстановитьМаксимальныйРазмерФайлаЖурнала`
- **Arguments**: `size: i32`
- **Return Type**: `None`
- **C++ Behavior**: Call `td_set_log_max_file_size` with the file size argument.

```rust
#[add_in_func(name = "SetLogMaxFileSize", name_ru = "УстановитьМаксимальныйРазмерФайлаЖурнала")]
#[arg(ty = Int)]
pub fn set_log_max_file_size(&self, size: i32) {
    unsafe {
        td_set_log_max_file_size(size as i64);
    }
}
```

### Method 7: `SetLogVerbosityLevel` / `УстановитьУровеньДетализацииЖурнала`
- **Arguments**: `level: i32`
- **Return Type**: `None`
- **C++ Behavior**: Call `td_set_log_verbosity_level` with the log verbosity level.

```rust
#[add_in_func(name = "SetLogVerbosityLevel", name_ru = "УстановитьУровеньДетализацииЖурнала")]
#[arg(ty = Int)]
pub fn set_log_verbosity_level(&self, level: i32) {
    unsafe {
        td_set_log_verbosity_level(level);
    }
}
```

---

## 6. Component Initialization & Cleanup

### Constructor (`Default` implementation)
```rust
impl Default for TelegramNativeAddIn {
    fn default() -> Self {
        unsafe {
            Self {
                connection: Arc::new(None),
                event_source_name: String::from("TelegramNative"),
                telegram_client: td_json_client_create(),
                async_mode: Arc::new(AtomicBool::new(false)),
                rcv_thread: None,
                rcv_timeout: 1.0,
            }
        }
    }
}
```

### Destructor (`Drop` implementation)
To replicate the C++ destructor behavior, we implement `Drop` to ensure the receiving thread is joined and the TDLib JSON client is properly destroyed.
```rust
impl Drop for TelegramNativeAddIn {
    fn drop(&mut self) {
        // Stop receiving loop
        self.async_mode.store(false, std::sync::atomic::Ordering::Relaxed);
        if let Some(handle) = self.rcv_thread.take() {
            let _ = handle.join();
        }
        // Destroy client
        unsafe {
            td_json_client_destroy(self.telegram_client);
        }
    }
}
```

---

## 7. Build and Integration Steps
1. Create directory `src` inside the Rust package containing `lib.rs`.
2. Define the main struct and macro invocation at the end of the file:
   ```rust
   extern_functions! {
       TelegramNativeAddIn::default(),
   }
   ```
3. Compile with `cargo build --release`.
4. The generated `.dll` (Windows) or `.so` (Linux) file will export the same Native API interfaces expected by 1C.
