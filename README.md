# Hubbub

<div align="center">

![Hubbub Desktop Banner](src/assets/hero.png)

### The Privacy-First Multi-Agent Desktop AI Workspace

Landing page: [thanghn14/hubbub-chat-landing](https://github.com/thanghn14/hubbub-chat-landing)

[![Release](https://img.shields.io/badge/release-v0.1.0-6366f1.svg?style=flat-square)](https://github.com/thanghn14/hubbub-chat/releases)
[![Platform](https://img.shields.io/badge/platform-Windows%20x64-0284c7.svg?style=flat-square)](https://tauri.app)
[![Rust](https://img.shields.io/badge/rust-2021%20edition-orange.svg?style=flat-square)](https://www.rust-lang.org)
[![Tauri](https://img.shields.io/badge/tauri-v2.12-24c8db.svg?style=flat-square)](https://tauri.app)
[![React](https://img.shields.io/badge/react-19.2-61dafb.svg?style=flat-square)](https://react.dev)
[![TypeScript](https://img.shields.io/badge/typescript-6.0-3178c6.svg?style=flat-square)](https://www.typescriptlang.org)
[![Database](https://img.shields.io/badge/database-SQLite3%20%2B%20FTS5-003b57.svg?style=flat-square)](https://sqlite.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-emerald.svg?style=flat-square)](LICENSE)

*An enterprise-grade, memory-efficient desktop workspace that orchestrates multi-provider AI agents with zero cloud credential leakage and full-text workspace indexing.*

[Khám phá Tính năng](#key-features) • [Kiến trúc Kỹ thuật](#technical-highlights) • [Bắt đầu Nhanh](#getting-started) • [Chính sách Bảo mật](#security--privacy) • [Đóng góp](#contributing)

</div>

---

## Tổng quan (Overview)

**Hubbub** là ứng dụng desktop cá nhân thế hệ mới dành cho lập trình viên, kỹ sư và nhà nghiên cứu. Được xây dựng trên nền tảng **Rust + Tauri 2** kết hợp **React 19**, Hubbub tái định nghĩa trải nghiệm làm việc với AI bằng cách đưa khả năng điều phối đa tác tử (Multi-Agent Orchestration), kiểm soát chính sách bảo mật (Security Policy Guard), và lập chỉ mục báo cáo toàn văn (Full-Text Search FTS5) trực tiếp về thiết bị cục bộ của người dùng.

Khác với các ứng dụng web thông thường, Hubbub bảo vệ tuyệt đối quyền riêng tư:
- **Toàn bộ khóa API** được mã hóa và lưu trữ độc quyền trong **Windows Credential Manager (OS Keyring)** — hoàn toàn không lưu plaintext trong file cấu hình hay cơ sở dữ liệu.
- **Dữ liệu trò chuyện & tài liệu** được lưu trữ cục bộ trong cơ sở dữ liệu SQLite nhúng, không gửi về bất kỳ máy chủ trung gian nào.
- **Tiêu thụ tài nguyên tối ưu**: Rust Core chạy nền chỉ tiêu tốn ~3.2MB RAM, tổng ứng dụng dưới 180MB RAM với khả năng stream phản hồi 50+ tokens/giây mượt mà.

---

## Tính năng Nổi bật (Key Features)

### 1. Điều phối Đa Tác tử Chuyên biệt (Role-based Functional Agents)
- Thiết lập nhiều tác tử phục vụ từng mục đích chuyên sâu: **Nhà phân tích hệ thống (Analyst)**, **Kỹ sư phần mềm (Developer)**, **Chuyên viên nghiên cứu (Researcher)**, **Tác giả kỹ thuật (Writer)**, và **Cố vấn học thuật (Tutor)**.
- Mỗi tác tử sở hữu System Prompt riêng biệt, giới hạn ngân sách (Token Budget, Max Steps, Cost Guard) và danh mục công cụ được cấp quyền.

### 2. Cổng Kết nối Đa Mô hình Linh hoạt (Hybrid Multi-Provider Gateway)
- Hỗ trợ tức thì các nhà cung cấp AI hàng đầu:
  - **Google Gemini**: Gemini 3.8 Flash, Gemini 1.5 Pro, Gemini 2.5 Flash.
  - **Anthropic Claude**: Claude Sonnet 4, Claude 3.5 Sonnet.
  - **OpenAI**: GPT-4o, GPT-4o-mini.
  - **Groq Cloud**: Llama 3.3 70B Versatile với tốc độ suy luận siêu tốc.
  - **Ollama Local**: Chạy offline 100% không cần Internet với Qwen 2.5 Coder, Llama 3.2.
  - **OpenRouter**: Truy cập hàng trăm mô hình mã nguồn mở toàn cầu.

### 3. Bảo mật Cấp Doanh nghiệp (Enterprise OS Keyring & Sandbox Policy)
- **Zero Cloud Leakage**: Sử dụng `keyring-rs` tương tác trực tiếp với Windows Data Protection API (DPAPI).
- **Path Guard**: Ngăn chặn tấn công Path Traversal (`../`), giam giữ mọi thao tác đọc/ghi file trong phạm vi Workspace an toàn.
- **URL Guard**: Chặn truy cập dải mạng nội bộ (SSRF prevention), chỉ cho phép nạp tài liệu từ các domain an toàn.
- **Nhật ký Kiểm toán (Audit Logs)**: Ghi nhận chi tiết mọi lần thực thi công cụ kèm tham số băm (digest) và quyết định cho phép/từ chối.

### 4. Giám sát Hạn mức & Chi phí Trực quan (Smart Model Quota Intelligence)
- Tự động theo dõi số lượt gọi API trong ngày (Requests Per Day - RPD) và tốc độ gọi (RPM).
- Thống kê chi tiết lượng Token đầu vào (Prompt Tokens), Token hoàn thành (Completion Tokens), và ước tính chi phí USD thời gian thực.
- Cảnh báo trực quan khi tiến trình sử dụng chạm ngưỡng hạn mức API.

### 5. Kho Báo cáo Workspace & Tìm kiếm Toàn văn FTS5
- Cho phép AI tự động xuất kết quả nghiên cứu thành các tài liệu Markdown có cấu trúc vào thư mục `reports/`.
- Tích hợp trình soạn thảo WYSIWYG thời gian thực **Milkdown** (Nord Theme).
- Công cụ tìm kiếm **SQLite FTS5** cho phép tra cứu toàn văn tài liệu và hội thoại với độ trễ dưới 5ms.

### 6. Giao diện Đậm chất Công nghệ (Antigravity & Cursor Developer Aesthetic)
- Thiết kế Dark Mode phân tầng sang trọng (Obsidian / Zinc / Slate), các góc bo tròn tinh tế và viền phát sáng nhẹ hairline.
- Thanh bên thông minh có thể **thu gọn / mở rộng linh hoạt (Collapsible Sidebar)** tối ưu cho cả chế độ màn hình nhỏ (Split Screen 720px - 900px) và màn hình lớn 4K Ultrawide.
- Khung nhập lệnh hiện đại hỗ trợ đầy đủ bộ gõ tiếng Việt (UniKey, EVKey) không lỗi ký tự.

---

## Kiến trúc Kỹ thuật (Technical Highlights)

Hệ thống được thiết kế theo nguyên lý **Clean Architecture** và phân tách độc lập giữa giao diện người dùng và lõi xử lý hệ thống:

- **Desktop Shell & IPC**: Nền tảng **Tauri 2** với cơ chế IPC bất đồng bộ an toàn kiểu dữ liệu (Strongly-typed IPC Commands & Events).
- **Core Runtime**: Lõi backend được viết 100% bằng **Rust** kết hợp **Tokio Async Engine**, đảm bảo tốc độ tối đa và kiểm soát an toàn bộ nhớ.
- **Storage Subsystem**: Hệ thống lưu trữ SQLite nhúng với tính năng Transaction WAL mode và bộ lập chỉ mục toàn văn bản FTS5.
- **Credential Storage**: Tích hợp Windows Credential Manager thông qua DPAPI, đảm bảo API key được mã hóa ở cấp tài khoản hệ điều hành.
- **Frontend Stack**: Ứng dụng client xây dựng trên **React 19**, **TypeScript**, **Tailwind CSS v4**, và **Vite 8**, kết hợp ảo hóa danh sách bằng **TanStack Virtual** cho phép cuộn mượt mà hàng ngàn tin nhắn.

---

## Bắt đầu Nhanh (Getting Started)

### Yêu cầu Tiên quyết (Prerequisites)

1. **Hệ điều hành**: Windows 10/11 x64.
2. **Rust Toolchain**: Rust phiên bản stable mới nhất (MSVC target).
   ```bash
   rustup default stable-x86_64-pc-windows-msvc
   ```
3. **C++ Build Tools**: Visual Studio 2022 với workload "Desktop development with C++".
4. **Node.js**: Phiên bản `>= 20.x` và `npm` đi kèm.
5. **Just Command Runner** (Tùy chọn nhưng khuyên dùng):
   ```bash
   cargo install just
   ```

### Cài đặt & Khởi chạy (Installation)

```bash
# 1. Clone repository
git clone https://github.com/thanghn14/hubbub-chat.git
cd hubbub-chat

# 2. Cài đặt dependencies cho ứng dụng Desktop
npm install

# 3. Khởi chạy chế độ phát triển (Dev Mode)
just dev
# Hoặc chạy lệnh trực tiếp nếu không cài just:
# cargo tauri dev --manifest-path src-tauri/Cargo.toml
```

### Kiểm thử & Đóng gói (Testing & Building)

```bash
# Chạy toàn bộ test suites của Rust Core
just test

# Kiểm tra linting và chuẩn format mã nguồn
just check

# Đóng gói bản phát hành Windows (.msi, .exe, .zip portable)
just build
```

Các gói cài đặt sau khi đóng gói thành công sẽ nằm tại:
`target/release/bundle/msi/` và `target/release/bundle/nsis/`.

---

## Phím tắt & Thao tác Tiện ích (Shortcuts & Usage)

| Phím tắt / Hành động | Mô tả |
| :--- | :--- |
| `Enter` | Gửi tin nhắn đến tác tử AI đang chọn |
| `Shift + Enter` | Xuống dòng trong khung soạn thảo |
| `Toggle Panel` | Đóng/mở thanh hội thoại bên trái để mở rộng không gian |
| `Copy Code` | Một cú nhấp để sao chép toàn bộ khối mã nguồn trong câu trả lời |
| `Save Report` | Tự động lưu nội dung nghiên cứu vào `reports/` của Workspace |
| `Model Switch` | Đổi mô hình LLM tức thời ngay trên thanh tiêu đề |

---

## Chính sách Bảo mật (Security & Privacy)

Hubbub được thiết kế theo tư duy bảo mật phòng thủ chiều sâu (Defense-in-Depth):

1. **Zero-Knowledge Core**: Ứng dụng không tích hợp bất kỳ công cụ thu thập telemetry ẩn hoặc theo dõi hành vi người dùng nào.
2. **Encrypted Keyring**: Khóa bí mật chỉ được nạp vào bộ nhớ RAM khi thực hiện request gọi API và bị xóa ngay sau đó.
3. **Workspace Isolation**: Tác tử AI chỉ được phép tương tác với các tệp tin thuộc thư mục Workspace người dùng đã chỉ định; mọi yêu cầu vượt ra ngoài đều bị Policy Guard chặn đứng lập tức.

---

## Tài liệu Tham khảo (Documentation)

Thông tin kỹ thuật chuyên sâu được duy trì trong thư mục `docs/`:

- [Kế hoạch Phát triển Tổng thể (Master Plan)](docs/MASTER_PLAN.md)
- [Nhật ký Quyết định Kỹ thuật (Decision Log)](docs/DECISION_LOG.md)
- [Kiến trúc Chi tiết Hệ thống (Architecture)](docs/ARCHITECTURE.md)
- [Mô hình Phân tích Mối đe dọa (Threat Model)](docs/THREAT_MODEL.md)
- [Quy trình Phát triển với AI (AI Workflow)](docs/AI_WORKFLOW.md)

---

## Bản quyền & Giấy phép (License)

Dự án được phân phối dưới giấy phép **Apache License 2.0**. Xem chi tiết tại tệp [LICENSE](LICENSE).

<div align="center">
<sub>Được thiết kế và phát triển với cam kết tối đa về hiệu năng, bảo mật và trải nghiệm nhà phát triển.</sub>
</div>
