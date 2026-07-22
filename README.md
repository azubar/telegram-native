<div align="center">

# ⚡ Telegram Native for 1C:Enterprise / BAS (v2.1.1)

<p align="center">
  <b>Сучасна, супершвидка та безпечна Native API компонента для 1С:Підприємство та BAS мовою Rust на базі TDLib 1.8.65</b>
</p>

[![Version](https://img.shields.io/badge/Version-2.1.1-blue.svg?style=for-the-badge)](https://github.com/azubar/telegram-native/releases/tag/v2.1.1)
[![Rust](https://img.shields.io/badge/Rust-2021-orange.svg?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
[![TDLib](https://img.shields.io/badge/TDLib-1.8.65-blue.svg?style=for-the-badge&logo=telegram)](https://github.com/tdlib/td)
[![1C:Enterprise](https://img.shields.io/badge/1C%3AEnterprise-8.3+-red.svg?style=for-the-badge)](https://1c.ru)
[![License](https://img.shields.io/badge/License-BSL_1.0-brightgreen.svg?style=for-the-badge)](LICENSE_1_0.txt)

---

</div>

## 🚀 Переваги та ключові відмінності оновленої версії (v2.1.1)

Порівняно з класичними реалізаціями компоненти на C++, нове покоління `telegram-native` пропонує принципово новий рівень надійності та швидкодії:

- 🦀 **Повний перехід на Rust (Edition 2021)**: Гарантована безпека роботи з пам'яттю (Memory Safety). Повністю усунуто небезпечні вказівники, витоки пам'яті (memory leaks) та падіння 1С через сегфолти.
- 📦 **Повністю автономні (Standalone) бінарники**: Всі залежності — **TDLib 1.8.65**, **OpenSSL**, **SQLite3** та **zlib** — скомпільовані статично та повністю вбудовані (*embedded*) всередину `.dll` та `.so` файлів. Компонента працює "Out of the box" і не вимагає наявності сторонніх DLL/SO у системі.
- 💻 **Полна кросплатформеність (4 в 1)**: Одночасна підтримка **Windows x86 / x64** та **Linux x86 / x64** в одному підсумковому архіві `AddIn.zip`.
- ⚡ **Вдосконалена асинхронна обробка**: Використання потокобезпечних Rust-каналів забезпечує стабільну передачу сповіщень Telegram в 1С без замирання інтерфейсу користувача.
- 🛠 **Сучасна CI/CD автоматизація**: Повністю автоматична мультиплатформена збірка релізів у GitHub Actions.

---

## 🏛 Архітектура

```mermaid
graph TD
    A[1С:Підприємство / BAS] <-->|Native API C-FFI| B[telegram_native.dll / .so]
    B <-->|Rust Wrapper| C[native_api_1c Template]
    B <-->|Static Linkage| D[TDLib 1.8.65 + OpenSSL + SQLite3 + ZLib]
    D <-->|MTProto / HTTPS| E[Telegram Cloud Servers]
```

### Підтримувані операційні системи:
- 🪟 **Windows**: x86 (32-bit), x64 (64-bit)
- 🐧 **Linux**: x86 (32-bit), x64 (64-bit)

---

## ⚡ Швидкий старт в 1С

```bsl
// 1. Підключення компоненти з макету або файлу AddIn.zip
Якщо Не ПодключитьВнешнююКомпоненту("ОбщийМакет.TelegramNative", "TelegramNative", ТипВнешнейКомпоненты.Native) Тогда
    ВызватьИсключение "Не вдалося подключити зовнішню компоненту TelegramNative!";
КонецЕсли;

// 2. Створення екземпляра об'єкта
Телеграм = Новый("AddIn.TelegramNative.TelegramNative");

// 3. Синхронне виконання легких запитів
ПараметрИмя = "{""@type"":""getOption"",""name"":""version""}";
Версия = Телеграм.Выполнить(ПараметрИмя);
Сообщить("Версія TDLib: " + Версия);

// 4. Включення асинхронного режиму для фонового отримання подій
Телеграм.ИмяИсточникаСобытий = "TelegramNative";
Телеграм.УстановитьАсинхронныйРежим(Истина);
```

---

## 📚 Таблиця API

### Властивості

| Англійська назва | Російська назва | Тип | Опис |
| :--- | :--- | :--- | :--- |
| `EventSourceName` | `ИмяИсточникаСобытий` | Рядок | Назва джерела події при генерації зовнішньої події в 1С. |

### Методи

| Англійська назва | Російська назва | Тип | Опис |
| :--- | :--- | :--- | :--- |
| `Send(Request)` | `Отправить(Запрос)` | Процедура | Асинхронна відправка JSON-запиту в TDLib. |
| `Receive()` | `Получить()` | Функція | Отримання наступної JSON-відповіді або оновлення від TDLib. |
| `Execute(Request)` | `Выполнить(Запрос)` | Функція | Синхронний виклик JSON-запиту в TDLib. |
| `SetAsyncMode(Enable)` | `УстановитьАсинхронныйРежим(Включить)` | Процедура | Управління фоновим потоком обробки оновлень. |
| `SetLogFilePath(Path)` | `УстановитьФайлЖурнала(Путь)` | Функція | Встановлення шляху для логування TDLib. |
| `SetLogMaxFileSize(Size)` | `УстановитьМаксимальныйРазмерФайлаЖурнала(Размер)` | Процедура | Встановлення максимального розміру лог-файлу. |
| `SetLogVerbosityLevel(Level)` | `УстановитьУровеньДетализацииЖурнала(Уровень)` | Процедура | Встановлення деталізації логування TDLib (0-10). |

---

## 🛠 Збірка з вихідного коду

### Вимоги:
- [Rust toolchain](https://rustup.rs/) (`1.70+`)
- CMake та C++ компілятор (MSVC / GCC)

### Команда збірки:
```bash
cargo build --release
```

Скомпільована динамічна бібліотека буде знаходитись у теці `target/release/`:
- Windows: `telegram_native.dll`
- Linux: `libtelegram_native.so`

---

## 📖 Документація

- 📘 [Детальне керівництво з використання компоненти в 1С](docs/1c_usage_guide.md)
- 📙 [План переходу на Rust та архітектура](docs/rust_transition_plan.md)

---

## 📄 Ліцензія

Проєкт розповсюджується під ліцензією [Boost Software License 1.0](LICENSE_1_0.txt).
