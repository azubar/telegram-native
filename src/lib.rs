use std::os::raw::{c_char, c_double, c_int, c_void};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use native_api_1c::{
    native_api_1c_core::ffi::connection::Connection,
    native_api_1c_macro::{extern_functions, AddIn},
};

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

#[derive(AddIn)]
pub struct TelegramNativeAddIn {
    #[add_in_con]
    connection: Arc<Option<&'static Connection>>,

    #[add_in_prop(
        ty = Str,
        name = "EventSourceName",
        name_ru = "ИмяИсточникаСобытий",
        readable,
        writable
    )]
    pub event_source_name: String,

    #[add_in_func(name = "Send", name_ru = "Отправить")]
    #[arg(ty = Str)]
    pub send: fn(&Self, String),

    #[add_in_func(name = "Receive", name_ru = "Получить")]
    #[returns(ty = Str)]
    pub receive: fn(&Self) -> String,

    #[add_in_func(name = "Execute", name_ru = "Выполнить")]
    #[arg(ty = Str)]
    #[returns(ty = Str)]
    pub execute: fn(&Self, String) -> String,

    #[add_in_func(name = "SetAsyncMode", name_ru = "УстановитьАсинхронныйРежим")]
    #[arg(ty = Bool)]
    pub set_async_mode: fn(&mut Self, bool),

    #[add_in_func(name = "SetLogFilePath", name_ru = "УстановитьФайлЖурнала")]
    #[arg(ty = Str)]
    #[returns(ty = Bool)]
    pub set_log_file_path: fn(&Self, String) -> bool,

    #[add_in_func(
        name = "SetLogMaxFileSize",
        name_ru = "УстановитьМаксимальныйРазмерФайлаЖурнала"
    )]
    #[arg(ty = Int)]
    pub set_log_max_file_size: fn(&Self, i32),

    #[add_in_func(
        name = "SetLogVerbosityLevel",
        name_ru = "УстановитьУровеньДетализацииЖурнала"
    )]
    #[arg(ty = Int)]
    pub set_log_verbosity_level: fn(&Self, i32),

    telegram_client: *mut c_void,
    async_mode: Arc<AtomicBool>,
    rcv_thread: Option<std::thread::JoinHandle<()>>,
    rcv_timeout: f64,
}

unsafe impl Send for TelegramNativeAddIn {}
unsafe impl Sync for TelegramNativeAddIn {}

impl TelegramNativeAddIn {
    fn send_inner(&self, command: String) {
        if self.telegram_client.is_null() {
            return;
        }
        if let Ok(c_str) = std::ffi::CString::new(command) {
            unsafe {
                td_json_client_send(self.telegram_client, c_str.as_ptr());
            }
        }
    }

    fn receive_inner(&self) -> String {
        if self.telegram_client.is_null() {
            return String::new();
        }
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

    fn execute_inner(&self, command: String) -> String {
        if self.telegram_client.is_null() {
            return String::new();
        }
        if let Ok(c_str) = std::ffi::CString::new(command) {
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
        } else {
            String::new()
        }
    }

    fn set_async_mode_inner(&mut self, enable: bool) {
        let current = self.async_mode.load(Ordering::Relaxed);
        if current == enable {
            return;
        }

        self.async_mode.store(enable, Ordering::Relaxed);

        if enable {
            if let Some(conn) = self.connection.as_ref() {
                let depth = conn.get_event_buffer_depth();
                if depth < 100 {
                    conn.set_event_buffer_depth(100);
                }
            }

            let client_addr = self.telegram_client as usize;
            let timeout = self.rcv_timeout;
            let conn_arc = Arc::clone(&self.connection);
            let event_source = self.event_source_name.clone();
            let async_flag = Arc::clone(&self.async_mode);

            self.rcv_thread = Some(std::thread::spawn(move || {
                let client = client_addr as *mut c_void;
                if client.is_null() {
                    return;
                }
                while async_flag.load(Ordering::Relaxed) {
                    unsafe {
                        let ptr = td_json_client_receive(client, timeout);
                        if !ptr.is_null() {
                            let response = std::ffi::CStr::from_ptr(ptr)
                                .to_string_lossy()
                                .into_owned();
                            if !response.is_empty() {
                                if let Some(conn) = conn_arc.as_ref() {
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

    fn set_log_file_path_inner(&self, path: String) -> bool {
        if let Ok(c_str) = std::ffi::CString::new(path) {
            unsafe { td_set_log_file_path(c_str.as_ptr()) != 0 }
        } else {
            false
        }
    }

    fn set_log_max_file_size_inner(&self, size: i32) {
        unsafe {
            td_set_log_max_file_size(size as i64);
        }
    }

    fn set_log_verbosity_level_inner(&self, level: i32) {
        unsafe {
            td_set_log_verbosity_level(level);
        }
    }
}

impl Default for TelegramNativeAddIn {
    fn default() -> Self {
        let client = unsafe { td_json_client_create() };
        Self {
            connection: Arc::new(None),
            event_source_name: String::from("TelegramNative"),
            send: Self::send_inner,
            receive: Self::receive_inner,
            execute: Self::execute_inner,
            set_async_mode: Self::set_async_mode_inner,
            set_log_file_path: Self::set_log_file_path_inner,
            set_log_max_file_size: Self::set_log_max_file_size_inner,
            set_log_verbosity_level: Self::set_log_verbosity_level_inner,
            telegram_client: client,
            async_mode: Arc::new(AtomicBool::new(false)),
            rcv_thread: None,
            rcv_timeout: 1.0,
        }
    }
}

impl Drop for TelegramNativeAddIn {
    fn drop(&mut self) {
        self.async_mode.store(false, Ordering::Relaxed);
        if let Some(handle) = self.rcv_thread.take() {
            let _ = handle.join();
        }
        if !self.telegram_client.is_null() {
            unsafe {
                td_json_client_destroy(self.telegram_client);
            }
        }
    }
}

extern_functions! {
    TelegramNativeAddIn::default(),
}
