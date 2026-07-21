# План переходу компоненти Native API з C++ на Rust

Цей документ містить опис архітектури та деталі реалізації переходу зовнішньої компоненти 1С `TelegramNative` з C++ на мову **Rust** за допомогою шаблону `native_api_1c`.

---

## 🎯 Цілі та Статус

| Завдання | Опис | Статус |
| :--- | :--- | :---: |
| **Заміна C++ коду** | Повна заміна legacy коду C++ на безпечний та ідіоматичний Rust | ✅ Завершено |
| **Сумісність з 1С** | Підтримка повного комплекту властивостей та методів (англ. та рус. назви) | ✅ Завершено |
| **FFI до TDLib** | Пряма взаємодія з C-API TDLib JSON Client | ✅ Завершено |
| **Асинхронний режим** | Фоновий потік оновлень та генерація зовнішніх подій в 1С | ✅ Завершено |
| **CI/CD та Реліз** | Автоматична збірка Windows/Linux (x86/x64) в GitHub Actions | ✅ Завершено |

---

## 1. Структура Rust-пакета (`Cargo.toml`)

Компонента збирається як динамічна бібліотека типу `cdylib`.

```toml
[package]
name = "telegram_native"
version = "1.8.65"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
utf16_lit = "2.0"
native_api_1c = { path = "./native_api_1c/native_api_1c" }
libc = "0.2"
once_cell = "1.18"
```

---

## 2. Відповідність типів та стану

| Поле C++ | Тип C++ | Тип Rust | Опис |
| :--- | :--- | :--- | :--- |
| `connection` | `IAddInDefBase*` | `Arc<Option<&'static Connection>>` | Декоратор підключення 1С `#[add_in_con]`. |
| `memory_manager` | `IMemoryManager*` | Прозоро в `native_api_1c` | Керування пам'яттю для рядків та двійкових даних. |
| `event_source_name` | `icu::UnicodeString` | `String` | Назва джерела подій (`"TelegramNative"`). |
| `telegram_client` | `void*` | `*mut c_void` | Вказівник на екземпляр TDLib JSON Client. |
| `async_mode` | `bool` | `Arc<AtomicBool>` | Прапор активності фонового потоку. |
| `rcv_thread` | `std::thread` | `Option<std::thread::JoinHandle<()>>` | Хендл фонового потоку оновлень. |
| `rcv_timeout` | `double` | `f64` | Таймаут `td_json_client_receive` (1.0 сек). |

---

## 3. Декларації TDLib FFI

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

## 4. Карта властивостей та методів

### Властивості
- `EventSourceName` / `ИмяИсточникаСобытий` (Рядок, Читання/Запис)

### Методи
1. `Send` / `Отправить` — Надіслати JSON запит (Процедура).
2. `Receive` / `Получить` — Отримати JSON відповідь (Функція).
3. `Execute` / `Выполнить` — Синхронне виконання JSON запиту (Функція).
4. `SetAsyncMode` / `УстановитьАсинхронныйРежим` — Керування фоновим потоком (Процедура).
5. `SetLogFilePath` / `УстановитьФайлЖурнала` — Встановити файл логів (Функція).
6. `SetLogMaxFileSize` / `УстановитьМаксимальныйРазмерФайлаЖурнала` — Встановити макс. розмір логу (Процедура).
7. `SetLogVerbosityLevel` / `УстановитьУровеньДетализацииЖурнала` — Встановити рівень логування (Процедура).

---

## 5. Очищення та Drop
Реалізовано Трейт `Drop` для автоматичної зупинки фонового потоку та знищення клієнта TDLib при вивантаженні компоненти 1С:

```rust
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
```
