<div align="center">

# 🚀 rs-telex

**Bộ gõ Tiếng Việt Telex thuần Rust (Pure Rust) siêu nhẹ, hiệu năng cực cao cho Linux (IBus / GNOME / Wayland / X11)**

[![Rust](https://img.shields.io/badge/language-Rust%202024-orange.svg?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![IBus](https://img.shields.io/badge/integration-IBus%20D--Bus-blue.svg?style=flat-square&logo=linux)](https://github.com/ibus/ibus)
[![License: MIT](https://img.shields.io/badge/license-MIT-green.svg?style=flat-square)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Linux%20(Wayland%20%2F%20X11)-lightgrey.svg?style=flat-square&logo=linux)](https://kernel.org)
[![Tests](https://img.shields.io/badge/tests-15%2F15%20passed-brightgreen.svg?style=flat-square)]()

*Không phụ thuộc C/C++ FFI nặng nề — Phản hồi tức thì < 1ms — Hỗ trợ Free Telex thông minh & Tối ưu cho Lập trình viên*

</div>

---

## 🌟 Điểm nổi bật (Key Features)

- 🦀 **Pure Rust & Zero C-FFI Runtime:** Tích hợp trực tiếp với giao thức IBus D-Bus thông qua `zbus 5.x`, loại bỏ hoàn toàn các lỗi memory leak, crash ngầm và các phụ thuộc C/GLib cồng kềnh.
- ⚡ **Siêu nhẹ & Độ trễ cực thấp (< 1ms):** Bộ nhớ RAM sử dụng thực tế < 8MB, CPU 0% khi nhàn rỗi. Tối ưu hóa tối đa cho phản hồi gõ tức thời trên cả Wayland lẫn X11.
- 🎯 **Thuật toán Free Telex tự do:** Bỏ dấu thanh (`s, f, r, x, j`) và phím biến âm (`a, e, o, w, d`) ở bất kỳ vị trí nào trong từ mà không sợ gõ sai thứ tự:
  - `vanax` $\rightarrow$ **vẫn**
  - `loiox` $\rightarrow$ **lỗi**
  - `khongo` $\rightarrow$ **không**
  - `gox` $\rightarrow$ **gõ**
  - `dduocwj` / `duowcj` $\rightarrow$ **được** / **dược**
  - `luuw` $\rightarrow$ **lưu**
  - `thuyr trieefu` $\rightarrow$ **thủy triều**
- 👨‍💻 **Coder Smart Mode (Tối ưu cho Lập trình viên):**
  - Tự động nhận diện code identifiers, `camelCase`, `snake_case`, từ khoá lập trình (`async`, `struct`, `class`, `printf`, `function`, `string`, `rust`...).
  - Không nuốt phím khi từ chưa có nguyên âm (gõ `printf`, `string`, `test` hoàn toàn tự nhiên không bị dính dấu).
  - Tự động hoàn tác thông minh khi gõ từ tiếng Anh.
- 🔄 **Chuyển đổi Tiếng Việt / Tiếng Anh thông minh:**
  - Hỗ trợ phím tắt chuyển đổi nhanh: <kbd>Ctrl</kbd> + <kbd>Shift</kbd>, <kbd>Alt</kbd> + <kbd>Z</kbd>, hoặc <kbd>CapsLock</kbd>.
  - Thoát nhanh chế độ bằng phím <kbd>Esc</kbd>.
  - Khi ở chế độ **EN**, engine chuyển 100% phím bấm dạng pass-through, không can thiệp vào buffer hệ thống.
- 📦 **Đóng gói linh hoạt:** Hỗ trợ cài đặt User-level (không cần quyền `sudo`), System-level qua `Makefile` hoặc gói Debian `.deb`.

---

## 📋 Bảng Quy tắc Gõ Telex (Typing Rules)

### 1. Phím Biến Âm (Vowel & Consonant Modifiers)

| Phím Telex | Ký tự gốc | Kết quả | Ví dụ |
| :---: | :---: | :---: | :--- |
| `aa` / `a` | `a` | **â** | `caan` $\rightarrow$ cân, `vana` $\rightarrow$ vân |
| `aw` / `w` | `a` | **ă** | `cawn` $\rightarrow$ căn, `anw` $\rightarrow$ ăn |
| `ee` / `e` | `e` | **ê** | `ddem` + `e` $\rightarrow$ đêm, `le` + `e` $\rightarrow$ lê |
| `oo` / `o` | `o` | **ô** | `coo` $\rightarrow$ cô, `khong` + `o` $\rightarrow$ không |
| `ow` / `w` | `o` | **ơ** | `cow` $\rightarrow$ cơ, `owri` $\rightarrow$ ơi |
| `uw` / `w` | `u` | **ư** | `tuw` $\rightarrow$ tư, `thuwr` $\rightarrow$ thử |
| `w` | `uo` / `uu` | **ươ** / **ưu** | `dduocwj` $\rightarrow$ được, `luuw` $\rightarrow$ lưu |
| `dd` / `d` | `d` | **đ** | `ddi` $\rightarrow$ đi, `dad` $\rightarrow$ đa |

### 2. Phím Dấu Thanh (Tones)

| Phím | Dấu thanh | Ví dụ |
| :---: | :--- | :--- |
| `s` | Sắc | `cas` $\rightarrow$ **cá**, `rus` $\rightarrow$ **rú** |
| `f` | Huyền | `caf` $\rightarrow$ **cà**, `moof` $\rightarrow$ **mồ** |
| `r` | Hỏi | `car` $\rightarrow$ **cả**, `thuyr` $\rightarrow$ **thủy** |
| `x` | Ngã | `cax` $\rightarrow$ **cã**, `loiox` $\rightarrow$ **lỗi** |
| `j` | Nặng | `caj` $\rightarrow$ **cạ**, `vieetj` $\rightarrow$ **việt** |
| `z` | Huỷ dấu thanh (giữ mũ/móc) | `casz` $\rightarrow$ **ca**, `aasz` $\rightarrow$ **â** |

> **Mẹo:** Bấm lặp lại phím dấu để huỷ dấu về chữ cái gốc (ví dụ: `as` $\rightarrow$ `á`, bấm thêm `s` $\rightarrow$ `as`, sau đó gõ `t` $\rightarrow$ `ast`).

---

## 🛠️ Cài đặt (Installation)

### Yêu cầu hệ thống (Prerequisites)
- Linux với **IBus** daemon đã cài đặt (`ibus`, `libibus-1.0-dev` hoặc tương đương).
- **Rust toolchain** (nếu tự build từ mã nguồn): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

---

### Cách 1: Cài đặt cho User hiện tại (Khuyên dùng - Không cần root)

```bash
# Clone repository
git clone https://github.com/duyphan0503/rs-telex.git
cd rs-telex

# Build và cài đặt vào ~/.local
make install-user

# Khởi động lại IBus daemon và chọn engine
ibus restart
ibus engine rs-telex
```

---

### Cách 2: Cài đặt toàn hệ thống (System-wide)

```bash
# Build và cài đặt vào /usr/
sudo make install

# Khởi động lại IBus
ibus restart
ibus engine rs-telex
```

---

### Cách 3: Cài đặt qua gói Debian `.deb` (Ubuntu / Debian / Linux Mint)

```bash
# Đóng gói file .deb
make deb

# Cài đặt gói vừa tạo
sudo dpkg -i target/rs-telex_0.1.0_amd64.deb

# Khởi động lại IBus
ibus restart
ibus engine rs-telex
```

---

## ⚙️ Kích hoạt trên Desktop Environment

### Trên GNOME (Ubuntu, Fedora, Debian...)
1. Mở **Settings** $\rightarrow$ **Keyboard** (Bàn phím).
2. Tại mục **Input Sources** (Nguồn nhập liệu), bấm dấu **+** (Thêm).
3. Chọn **Vietnamese** $\rightarrow$ chọn **Vietnamese - Telex (rs-telex)**.
4. Bấm tổ hợp <kbd>Super</kbd> + <kbd>Space</kbd> để chuyển sang bộ gõ `rs-telex`.

### Chuyển đổi nhanh qua Terminal
```bash
# Bật engine rs-telex
ibus engine rs-telex

# Kiểm tra engine hiện tại
ibus engine
```

---

## ⌨️ Phím tắt chuyển chế độ (Shortcuts)

| Phím tắt | Chức năng |
| :--- | :--- |
| <kbd>Ctrl</kbd> + <kbd>Shift</kbd> | Chuyển đổi qua lại giữa **Tiếng Việt** và **Tiếng Anh** |
| <kbd>Alt</kbd> + <kbd>Z</kbd> | Chuyển đổi qua lại giữa **Tiếng Việt** và **Tiếng Anh** |
| <kbd>CapsLock</kbd> | Bật/tắt nhanh chế độ gõ Tiếng Việt |
| <kbd>Esc</kbd> | Huỷ tạm thời từ đang gõ / Reset buffer |
| <kbd>Ctrl</kbd> + <kbd>Z</kbd> | Khôi phục lại từ thô vừa gõ (Undo word) |

---

## 🧪 Kiểm thử (Testing)

Dự án có bộ test suite bao phủ toàn bộ các trường hợp ngữ âm tiếng Việt, các từ khoá lập trình và các chuỗi gõ tự do:

```bash
cargo test
```

Kết quả:
```text
running 15 tests
test ibus::types::tests::test_make_ibus_text_signature ... ok
test ibus::address::tests::test_address_resolution ... ok
test tests::test_disabled_engine_mode ... ok
test tests::test_tone_undo_and_overwrite ... ok
test tests::test_basic_typing ... ok
test tests::test_english_mode_detection ... ok
test tests::test_toneless_after_tone ... ok
test tests::test_coder_smart_pass_through ... ok
test tests::test_unikey_tone_placement ... ok
test tests::test_uppercase ... ok
test tests::test_w_key_handling ... ok
test tests::test_z_removes_tone_preserves_hat ... ok
test tests::test_consonant_clusters ... ok
test tests::test_user_reported_cases ... ok
test tests::test_more_vietnamese_words ... ok

test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

---

## 📁 Cấu trúc Thư mục Dự án

```text
rs-telex/
├── Cargo.toml               # Rust workspace config
├── Makefile                 # Automation scripts (build, test, install, deb)
├── data/
│   ├── rs-telex.xml         # IBus Component & Engine descriptor
│   └── *.gschema.xml        # GSettings configuration schema
├── debian/
│   └── control              # Debian package metadata
├── rs-telex/
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs          # Daemon entry point & unit tests
│       ├── telex.rs         # Free Telex state engine & tone placement
│       ├── unicode.rs       # Vietnamese Unicode tables & marks
│       ├── syllable.rs      # Syllable validation & coder word detector
│       └── ibus/            # Pure Rust IBus D-Bus integration (zbus)
│           ├── bus.rs       # D-Bus session connection & registration
│           ├── engine.rs    # IBus Engine interface & key event handler
│           ├── factory.rs   # IBus Factory service
│           └── types.rs     # IBus D-Bus signatures & structures
└── rs-telex-sys/            # Fallback bindings for libibus
```

---

## 🤝 Đóng góp (Contributing)

Mọi đóng góp nhằm cải thiện tốc độ, bổ sung tính năng hoặc sửa lỗi đều được hoan nghênh:

1. Fork dự án trên GitHub.
2. Tạo branch mới (`git checkout -b feature/tinh-nang-moi`).
3. Commit các thay đổi (`git commit -m 'Add: Tính năng mới'`).
4. Push lên branch (`git push origin feature/tinh-nang-moi`).
5. Tạo **Pull Request**.

---

## 📄 Giấy phép (License)

Dự án được phân phối dưới giấy phép [MIT License](LICENSE).
Tự do sử dụng, chỉnh sửa và phân phối cho mục đích cá nhân lẫn thương mại.
