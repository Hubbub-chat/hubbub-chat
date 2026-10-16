# Decision Log — Hubbub

Mọi quyết định quan trọng và thay đổi lớn trong quá trình phát triển **PHẢI** được ghi lại tại đây.
Sắp xếp theo thứ tự thời gian (mới nhất ở trên).

> **QUY TẮC:** PR/commit có thay đổi quan trọng mà KHÔNG cập nhật file này sẽ bị reject.

## [2026-10-16] Policy công cụ: mặc định từ chối, cấm truy cập `data/`

**Bối cảnh:**
Đánh giá bảo mật phát hiện `ToolRunner::check_policy` mặc định *cho phép*: nhánh `_ => {}` bỏ qua mọi kiểm tra với `report_list` và mọi tool không biết; không kiểm tra tool có được gán cho agent hay không (một lời gọi bị prompt injection có thể gọi tool ngoài danh sách). `fs_list("")` liệt kê thư mục gốc workspace mà không kiểm tra quyền. Thư mục `data/` (chứa `hubbub.db`, `agents.json`) nằm trong workspace nên agent có `fs_read: ["**"]` đọc được.

**Quyết định:**
1. Tool phải nằm trong `agent.tools.builtin` hoặc `agent.tools.mcp`, nếu không thì từ chối.
2. Builtin không có nhánh policy riêng thì từ chối (`unknown builtin tool`). Tool MCP được gán cho agent thì cho qua (MCP chưa triển khai; server MCP sẽ mang policy riêng).
3. `report_list` cần quyền đọc `reports`; liệt kê gốc workspace cần quyền đọc toàn bộ (`**`); `report_write` kiểm tra đúng tên file thật.
4. **Giữ nguyên vị trí `data/`** (không di chuyển DB, để không ảnh hưởng các bản cài v0.1.0) và chặn trong `PathGuard`: `check_read_path` cấm đọc, `is_protected_path` cấm ghi. So sánh đường dẫn theo cách Windows: không phân biệt hoa thường, bỏ `.` và dấu chấm/khoảng trắng cuối (`DATA/`, `./data`, `data./` đều bị chặn).
5. Agent mặc định `analyst`, `researcher` được thêm `fs_read: ["reports/**"]` để `report_read`/`report_list` dùng được. Lưu ý: bản cài đã lưu `agents.json` giữ quyền cũ của agent đó.

**Hệ quả:** Thêm builtin tool mới bắt buộc thêm nhánh trong `check_policy`, nếu không tool sẽ bị từ chối (trước đây bị cho qua âm thầm).

## [2026-10-04] Cấu hình Kết nối Model Cục bộ Qwen 2.5 Coder 7B Qua Ollama Cho Agent Developer

**Bối cảnh:**
Người dùng sở hữu model chuyên mã nguồn `qwen2.5-coder:7b` chạy cục bộ thông qua Ollama (`http://localhost:11434`) và yêu cầu cấu hình Hubbub kết nối tới mô hình này để phục vụ tác vụ lập trình mà không cần phụ thuộc vào cloud API key hay lo ngại về chi phí/hạn mức rate limit.

**Quyết định & Giải pháp Triển khai:**
1. **Định tuyến Provider Tự động (`crates/app/src/service.rs`):**
   - Cập nhật logic nhận diện model trong `AppService::send_message`: Nhận diện các họ model local (`qwen`, `deepseek`, `mistral`, `phi`) tự động trỏ về provider `"ollama"`.
   - Bỏ qua bước kiểm tra API key bắt buộc đối với provider `"ollama"`, cho phép sử dụng hoàn toàn offline.
2. **Cấu hình Mặc định Provider Ollama (`crates/app/src/config.rs`):**
   - Thiết lập endpoint chuẩn OpenAI-compatible của Ollama: `http://localhost:11434/v1`.
   - Cài đặt model mặc định là `"qwen2.5-coder:7b"` với thời gian chờ phản hồi an toàn `timeout_s: 180`.
3. **Gán Model Mặc định cho Tác tử Lập trình (`crates/app/src/seed.rs`):**
   - Chuyển đổi mô hình mặc định của Agent `developer` (Kỹ sư Lập trình) sang `"qwen2.5-coder:7b"`.
4. **Theo dõi Hạn mức & Mức Tiêu thụ Token (`crates/domain`, `crates/store`):**
   - Trong `model_quota.rs`: Tự động nhận diện model có tiền tố `qwen` thuộc provider `"ollama"` với hạn mức vô hạn (`0 RPM / 0 RPD`).
   - Trong `model_usage.rs`: Bổ sung `"qwen2.5-coder:7b"` vào danh sách theo dõi tài nguyên mặc định (`default_seed_models`).
5. **Giao diện Người Dùng (`apps/desktop`):**
   - Bổ sung `Qwen 2.5 Coder 7B (Ollama Cục bộ)` vào danh sách mô hình phổ biến trong `ModelSelector.tsx`, `AgentModal.tsx` và `AgentSkillsDrawer.tsx`.
6. **Kiểm thử & Đảm bảo Chất lượng:**
   - Đã kiểm tra trực tiếp endpoint Ollama cục bộ: `qwen2.5-coder:7b` hỗ trợ tốt cả text completion lẫn native tool calling.
   - 88/88 tests PASS (`cargo test --workspace`), 0 clippy warnings, `pnpm build` thành công 100%.

**Trạng thái:** Đã áp dụng

---

## [2026-10-04] Xử lý 4 Vấn đề Cốt lõi: Hợp nhất Lượt Hội thoại, Quản lý Kỹ năng Tác tử, Theo dõi Hạn mức Model và Tái thiết kế Toàn diện Giao diện Chuẩn LobeChat & Element

**Bối cảnh:**
Người dùng gửi yêu cầu giải quyết 4 vấn đề quan trọng trong ứng dụng:
1. Agent phản hồi nhiều lần cho 1 tin nhắn do các bước gọi công cụ bị phân mảnh thành nhiều bubble độc lập và thiếu khóa chống gửi đúp.
2. Tác tử (Agent) cho từng công việc cần được cấu hình chuyên môn, bộ kỹ năng và công cụ riêng biệt nhưng trước đây chưa được thể hiện rõ nét trên ứng dụng.
3. Thiếu tính năng theo dõi hạn mức (quota), số lượt gọi, token tiêu thụ và giới hạn RPM/RPD của từng model AI dẫn tới nguy cơ chạm lỗi 429.
4. Giao diện chưa đẹp, cần thiết kế lại toàn bộ theo chuẩn thanh lịch, gọn gàng, độ tương phản cao tham khảo từ **LobeChat** và **Element**.

**Quyết định & Giải pháp Triển khai:**
1. **Khắc phục lỗi Agent phản hồi nhiều lần (Vấn đề 1):**
   - **Mô hình Lượt Hội thoại Hợp nhất (`turnGrouper.ts`, `ChatTurnItem.tsx`):** Gom toàn bộ chuỗi message trong 1 lượt (tin nhắn User + các bước gọi công cụ `ToolCall` + kết quả `ToolResult` + văn bản kết luận của `Assistant`) vào đúng 1 khối `ChatTurn` duy nhất.
   - **Thẻ Accordion Công cụ gập mở:** Các công cụ gọi trung gian hiển thị dạng danh sách thẻ trạng thái gấp gọn (Đang chạy / Hoàn thành / Thất bại), nhấp vào để kiểm tra Input/Output chi tiết mà không phân mảnh luồng đàm thoại.
   - **Khóa In-Flight gửi tin an toàn (`isSendingRef`):** Khóa đồng bộ ngay lập tức tại frontend khi nhấn gửi hoặc Enter, ngăn ngừa triệt để gọi đúp lệnh IPC khi bộ gõ tiếng Việt đang hoạt động.
   - **Đồng bộ Cancellation ID (`runtime.rs`, `service.rs`):** Chuẩn hóa `execute_run_with_id` để bản đồ hủy `active_cancellations` khớp chính xác với `run.id` thực thi.

2. **Thể hiện rõ ràng Cấu hình & Kỹ năng của từng Tác tử (Vấn đề 2):**
   - **Chuẩn hóa Siêu dữ liệu Kỹ năng (`SKILLS_METADATA`):** Phân định rạch ròi 6 nhóm kỹ năng chuyên môn: 🌐 Web Search (Google / Tavily), 📄 Web Fetch (Trích xuất trang web), 📁 File Reader (Đọc workspace), 🗂️ Directory List, 📝 Report Generator (Báo cáo Markdown), 📖 Report Reader.
   - **Huy hiệu Kỹ năng trên Chat Header:** Hiển thị trực quan số kỹ năng đang kích hoạt của tác tử; nhấp vào để mở ngay ngăn kéo cấu hình.
   - **Ngăn kéo Thanh bên `AgentSkillsDrawer.tsx` (LobeChat Inspector):** Cho phép xem/chỉnh sửa trực tiếp System Prompt, bật/tắt từng kỹ năng công cụ, đổi mô hình vận hành và xem ngân sách của tác tử.
   - **Phân hệ Thư viện Tác tử `AgentsHubView.tsx`:** Màn hình quản lý toàn bộ tác tử với thẻ bài chuyên gia, vai trò, huy hiệu kỹ năng và nút trò chuyện tức thì.

3. **Tính năng Giám sát & Quản lý Hạn mức Model (Vấn đề 3):**
   - **Backend Aggregation Service (`model_usage.rs`, `model_quota.rs`):** Truy vấn tổng hợp từ SQLite tính toán tổng token (prompt + completion), số lượt gọi trong ngày, tổng số run, chi phí tích lũy ước tính và mốc sử dụng gần nhất theo từng model AI.
   - **Định mức Quota Mặc định:** Cung cấp thông số RPM và RPD chuẩn xác theo gói miễn phí của Google Gemini (15 RPM / 1,500 RPD), Claude (50 RPM / 5,000 RPD), OpenAI (500 RPM), Groq (30 RPM / 14,400 RPD), Ollama (Local vô hạn).
   - **Giao diện Giám sát `ModelQuotaModal.tsx` & Mini Quota Pill:** Bảng điều khiển trực quan với thanh tiến độ hạn mức ngày, tỉ lệ %, trạng thái cảnh báo khi chạm ngưỡng 80% hoặc lỗi 429.

4. **Tái thiết kế Toàn diện Giao diện (Vấn đề 4 - LobeChat & Element):**
   - **Bố cục Điều hướng 2 Tầng (Dual-Tier Shell):**
     - Cột điều hướng hẹp **Activity Dock (56px)** phong cách Element: Logo nhận diện phát sáng, icon chuyển đổi nhanh giữa Trò chuyện (Chat), Trung tâm Tác tử (Agents), Báo cáo Workspace (Workspace), Hạn mức (Quota), Cài đặt và Benchmark.
     - Cột danh sách phụ **Secondary Sidebar (260px)**: Tìm kiếm hội thoại, phân nhóm theo dòng thời gian (Hôm nay, Hôm qua, 7 ngày trước, Cũ hơn), và bộ chuyển nhanh Tác tử (Agent chips).
   - **Bảng màu Obsidian Dark cao cấp (`#08090d` / `#10141e` / `#181d29`):** Viền kính siêu mỏng (`hairline-border`), hiệu ứng mờ sương (`backdrop-blur-md`), độ tương phản chữ đạt chuẩn WCAG AA (`#f8fafc` và `#94a3b8`), không chói lóa và không mờ tối.
   - **Hộp soạn thảo Nổi (Floating Modern Composer):** Thiết kế bo cong nổi thanh lịch (`rounded-2xl shadow-2xl`), hiển thị huy hiệu tác tử/model đang kích hoạt, textarea tự động dãn nở, nút Gửi/Dừng phản hồi hiệu ứng êm dịu.

5. **Kiểm thử & Đảm bảo Chất lượng:**
   - 88/88 tests PASS (`cargo test --workspace`).
   - 0 warnings trên `cargo clippy --workspace --all-targets -- -D warnings`.
   - `pnpm build` đạt 0 lỗi TypeScript/Vite.
   - Toàn bộ files mới và chỉnh sửa đều tuân thủ nghiêm ngặt quy định `< 400 dòng`, functions `< 60 dòng`.

**Trạng thái:** Đã áp dụng

---

## [2026-10-04] Triển khai Sprint 2.4: Audit Logs, Workspace Watcher & Re-indexing, Security Battery, và Giao diện Báo cáo Workspace

**Bối cảnh:**
Theo kế hoạch phát triển Phase 2 (`MASTER_PLAN.md`), Sprint 2.4 hoàn thiện các mảnh ghép còn lại của hệ sinh thái Workspace và bảo mật Zero-Trust:
1. Ghi nhật ký kiểm toán (Audit Log) toàn diện cho mọi lượt gọi công cụ của tác tử (cả cho phép và từ chối).
2. Lắng nghe thay đổi tệp tin Workspace (`WorkspaceWatcher`) tự động lập lại chỉ mục FTS5 khi tệp được sửa từ bên ngoài (hoặc bên trong app), có cơ chế debounce và self-write ignore.
3. Bộ kiểm thử bảo mật nâng cao (Security Battery Tests): Prompt Injection resistance, Path Traversal comprehensive battery, và SSRF comprehensive battery.
4. Giao diện Desktop hoàn chỉnh cho Workspace: Quản lý & tìm kiếm báo cáo/tài liệu (FTS5 search), trình soạn thảo WYSIWYG Milkdown & xem Markdown, nút "Lưu thành Báo cáo" từ hội thoại, và bảng xem nhật ký kiểm toán bảo mật.

**Quyết định & Giải pháp Triển khai:**
1. **Thực thể & Kho lưu trữ AuditLog (`crates/domain`, `crates/store`):**
   - Bổ sung thực thể `AuditLog` và các ports `record_audit_log`, `list_audit_logs`.
   - Triển khai `AuditLogRepository` truy vấn bảng `audit_logs` có sẵn trong SQLite.
   - Tích hợp ghi log trong `ToolRunner`: mọi lần thực thi công cụ đều được ghi nhận quyết định `allow` hoặc `deny`, mã băm tham số và kết quả.
2. **Bộ Lắng nghe Tệp tin Workspace (`crates/workspace` - `WorkspaceWatcher`):**
   - Triển khai `WorkspaceWatcher` sử dụng `notify::RecommendedWatcher` kết hợp `tokio::sync::mpsc`.
   - Bộ lọc watchable path thông minh: bỏ qua các tệp ẩn (`.tmp_*`, `.git`), tệp trong `.versions/`, chỉ lập chỉ mục tệp tài liệu (`.md`, `.txt`).
   - Cơ chế Debouncing 200ms bằng `Instant` deadline map tránh trùng lặp sự kiện OS.
   - Cơ chế "Self-write Ignore": theo dõi `self_writes` trong `LocalWorkspaceService` để loại trừ các sự kiện sinh ra bởi chính ứng dụng.
   - Gắn kết tự động vào `ReportService` và `AppService::init`, đồng bộ chỉ mục FTS5 tức thời.
3. **Bộ Kiểm thử Bảo mật Nâng cao (`crates/agent/tests/security_battery_tests.rs`):**
   - `test_prompt_injection_simulation_blocked_and_audited`: Giả lập LLM bị nhiễm prompt injection cố gắng đọc tệp nhạy cảm hệ điều hành và gửi request SSRF tới cloud metadata. Policy engine chặn đứng cả hai, ghi nhận 2 bản ghi `decision: "deny"` trong audit logs.
   - `test_path_traversal_battery_all_variants_rejected`: Kiểm thử toàn diện 20+ biến thể path traversal (`../`, Windows UNC `\\server\share`, ADS `::$DATA`, URL-encoded `%2e%2e`, `%2f`, `%5c`, protected system files). Nâng cấp `PathGuard` để chặn đứng các vector mã hóa URL.
   - `test_ssrf_battery_all_private_targets_rejected`: Kiểm thử toàn diện 25+ mục tiêu SSRF (IPv4 loopback, Private Class A/B/C, AWS/GCP cloud metadata `169.254.169.254`, IPv6 loopback `[::1]`, Unique Local `[fc00::]`, non-HTTP schemes).
4. **Giao diện Desktop Workspace & Báo cáo (`apps/desktop`):**
   - Tạo component `WorkspaceReportsView`:
     - Thanh tìm kiếm toàn văn FTS5 siêu tốc kết nối với backend SQLite.
     - Danh sách tài liệu với dung lượng, ngày cập nhật.
     - Tích hợp trình soạn thảo WYSIWYG `MilkdownEditor` và chế độ xem trước `MarkdownContent`.
     - Phân hệ xem Nhật ký kiểm toán bảo mật với bộ lọc Tất cả / Cho phép / Từ chối.
   - Cập nhật `ChatView`: Bổ sung nút "Lưu thành Báo cáo" trên mọi câu trả lời của AI, kèm toast thông báo tức thời.
   - Cập nhật `Sidebar`: Thanh chuyển đổi điều hướng linh hoạt giữa "Trò chuyện" và "Kho Báo cáo".
5. **Kỷ luật Mã nguồn & Đảm bảo Chất lượng:**
   - Files < 400 dòng, functions < 60 dòng.
   - 0 `unwrap()`, 0 `expect()`, 0 `panic!` trong mã nguồn production.
   - 100% tests PASS (`cargo test --workspace`).
   - 0 warnings trên `cargo clippy --workspace --all-targets -- -D warnings`.
   - `pnpm build` biên dịch sạch sẽ 100%.

**Trạng thái:** Đã áp dụng

---

## [2026-10-03] Triển khai Sprint 2.3: Local Filesystem Tools (`hubbub-tools`) & Workspace Service (`hubbub-workspace`)

**Bối cảnh:**
Theo kế hoạch phát triển Phase 2 (`MASTER_PLAN.md`), Agent cần có khả năng đọc/quản lý tệp cục bộ có kiểm soát, tạo báo cáo nghiên cứu Markdown có cấu trúc, tự động đánh chỉ mục vào cơ sở dữ liệu SQLite FTS5 và quản lý phiên bản lịch sử tài liệu (.versions/).
Đồng thời, hệ thống phải tuân thủ nghiêm ngặt mô hình bảo mật Zero-Trust:
1. Agent có quyền đọc tệp cục bộ (`librarian`, `developer`) thì bắt buộc `network = "none"` để chống rò rỉ dữ liệu (Data Exfiltration).
2. Agent có quyền truy cập Internet (`analyst`, `researcher`) thì `fs_read` bị khóa hoàn toàn, chỉ được ghi vào `reports/**`.
3. Mọi thao tác ghi tệp phải là Atomic Write (ghi temp file -> fsync -> rename) và tự động sao lưu phiên bản cũ vào `.versions/` (giữ tối đa 20 phiên bản).
4. Các thư mục hệ thống nhạy cảm (`agents/`, `config.toml`, `.versions/`, `.git/`, `.env`) là bất khả xâm phạm đối với toàn bộ công cụ.

**Quyết định & Giải pháp Triển khai:**
1. **Bảo vệ Hệ thống Cốt lõi (`crates/policy` - `PathGuard`):**
   - Bổ sung `PathGuard::is_protected_path`: Chặn đứng mọi hành vi ghi đè hoặc can thiệp vào `config.toml`, `.env`, `.git/`, `agents/`, và `.versions/`.
   - Bổ sung `PathGuard::check_write_path`: Kiểm tra đường dẫn hợp lệ trong Workspace đồng thời xác thực không thuộc danh mục protected paths.
2. **Dịch vụ Workspace Cục bộ (`crates/workspace` - `LocalWorkspaceService`):**
   - Triển khai trọn vẹn domain port `WorkspaceService`:
     - `write_file`: Ghi tệp nguyên tử (Atomic Write) với `Uuid` temp file và `sync_all()`, tự động sao lưu phiên bản vào `.versions/{filename}.{timestamp}.bak` (tối đa 20 bản).
     - `read_file`: Kiểm tra `PathGuard`, từ chối tệp quá dung lượng cho phép (> 5MB), xác thực UTF-8 hợp lệ.
     - `list_files`: Liệt kê tệp tin an toàn theo thư mục, tự động lọc bỏ các thư mục ẩn/hệ thống.
     - `reindex_file`: Tính mã băm SHA-256, tự động trích xuất tiêu đề Markdown (`# Tiêu đề`), khởi tạo thực thể `Document` chuẩn.
3. **Bộ Công cụ Tệp & Báo cáo Tác tử (`crates/tools`):**
   - `FsReadTool` (`fs_read`): Đọc tệp tin với cơ chế rút gọn ngữ cảnh thông minh (tối đa 20.000 ký tự) cho LLM.
   - `FsListTool` (`fs_list`): Liệt kê tệp trong Workspace dạng Markdown có cấu trúc.
   - `ReportWriteTool` (`report_write`): Lưu báo cáo vào `reports/`, chuẩn hóa tiêu đề và tên tệp `YYYY-MM-DD-slug.md`, ghi tệp nguyên tử và tự động đồng bộ siêu dữ liệu vào bảng `documents` + `documents_fts` trong SQLite.
   - `ReportReadTool` (`report_read`): Đọc báo cáo từ `reports/`.
   - `ReportListTool` (`report_list`): Liệt kê tất cả báo cáo đã lưu kèm dung lượng.
   - `BuiltinToolHost`: Tích hợp các công cụ mới thông qua constructor `with_workspace(workspace, store)`.
4. **Tích hợp Tầng Ứng dụng & Shell (`crates/app` & `apps/desktop/src-tauri`):**
   - Tự động khởi tạo cấu trúc thư mục Workspace: `reports/`, `notes/`, `sources/`, `.versions/`, `data/`.
   - Tách `ProviderFactory` khỏi `service.rs` để duy trì kích thước file strictly `< 400 dòng`.
   - Cung cấp các lệnh Tauri IPC: `list_reports`, `read_report`, `list_documents`, `search_documents`.
   - Cập nhật quyền hạn cho tác tử `writer`: cho phép đọc/ghi `reports/**` với `network: none`.
5. **Kiểm thử Toàn diện (Test-First & Edge Cases):**
   - Bổ sung 7 test cases trong `workspace_tests.rs` (tiếng Việt có dấu, emoji, path traversal, file size limit, auto-versioning, atomic write).
   - Bổ sung test cases trong `tools_tests.rs` và `app_tests.rs`.
   - Bổ sung integration test `report_tool_tests.rs` trong `hubbub-agent`: xác minh FakeLlm gọi `report_write` tạo file thực tế trên đĩa và index vào SQLite store.
   - Toàn bộ 81 test cases trên workspace PASS 100%, Clippy 0 warnings, Frontend build thành công.

**Trạng thái:** Đã áp dụng

---

## [2026-10-03] Khắc Phục Triệt Để Lỗi Phản Hồi Bị Cắt Giữa Chừng (Response Truncation & Auto-Continuation)

**Bối cảnh:**
Người dùng phản ánh phản hồi của Agent đôi khi không đầy đủ, bị cắt cụt giữa chừng (ví dụ: ngắt đột ngột ở giữa từ `đặc ru...`, `chia nhỏ...`, hoặc giữa khối mã JSON `"score...`).
Kiểm tra thực tế trong cơ sở dữ liệu SQLite cục bộ xác nhận:
1. `max_tokens` bị nhầm lẫn giữa Lifetime Budget và Single-Turn Limit: Trong `llm_runner.rs`, cấu hình `LlmConfig.max_tokens` truyền trực tiếp giá trị `budget.max_tokens` (200.000 tokens) vào tham số `max_tokens` của payload HTTP gửi lên Google Gemini / OpenAI. Đối với API của Google Gemini, giới hạn `maxOutputTokens` tối đa là 8.192 tokens. Khi gửi 200.000, API của Google hoặc ép hạn mức về giá trị mặc định nhỏ hơn nhiều, hoặc gây xung đột với bộ đệm suy nghĩ.
2. Suy nghĩ ngầm (Extended Thinking) tiêu tốn token đầu ra: Trên các mô hình thế hệ mới (Gemini 2.5 / 3.8 Flash), các token suy luận nội bộ (`thought_signature` / thinking tokens) được tính gộp trực tiếp vào trần token đầu ra của lượt gọi. Khi hội thoại dài hoặc có nhiều công cụ, thinking tokens chiếm phần lớn ngân sách khiến phần câu trả lời hiển thị bị cạn kiệt token giữa chừng và trả về `finish_reason: "length"`.
3. Thiếu cơ chế Tự động Nối tiếp (Auto-Continuation): Khi mô hình chạm trần token đầu ra (`finish_reason: "length"`), luồng SSE cũ chỉ đơn thuần kết thúc và coi như hoàn thành, lưu chuỗi văn bản cụt vào database mà không có cơ chế nối dài tiếp.

**Quyết định & Giải pháp Triển khai:**
1. **Phân định rõ ràng Hạn mức Lượt gọi và Ngân sách Vòng đời (`llm_runner.rs`):**
   - Không gán `budget.max_tokens` (200.000) vào `LlmConfig.max_tokens`. Ngân sách vòng đời được kiểm soát an toàn bởi `BudgetTracker` trong Rust.
   - Để `LlmConfig.max_tokens` mặc định là `None`, cho phép mô hình khai thác trọn vẹn giới hạn đầu ra tối đa của nó (8.192 tokens).
2. **Kẹp an toàn Tham số Đầu ra (`openai_compat.rs`):**
   - Trong `OpenAiCompatAdapter`, nếu `config.max_tokens` có giá trị, tự động kẹp (clamp) về ngưỡng an toàn của mô hình (`<= 8192` cho Gemini, `<= 16384` cho các mô hình khác) để tránh bị từ chối hoặc fallback sai lệch.
3. **Bắt tín hiệu Kết thúc `finish_reason` & Tự động Nối tiếp (`Auto-Continuation`):**
   - Bổ sung variant `LlmChunk::FinishReason(String)` vào port `LlmChunk` trong `hubbub-domain`.
   - Trong `OpenAiStream` và `AnthropicStream`: bắt chính xác trường `finish_reason` từ API và phát thành `LlmChunk::FinishReason`.
   - Trong `LlmRunner`: khi phát hiện phản hồi kết thúc vì chạm trần độ dài (`finish_reason == "length"`), hệ thống tự động kích hoạt lượt sinh tiếp theo (tối đa 3 lần), hướng dẫn mô hình tiếp tục viết tiếp từ điểm dừng mà không lặp lại đoạn đã viết. Các đoạn văn bản mới được nối liền mạch và stream trực tiếp ra giao diện người dùng.
4. **Tăng cường Chỉ dẫn Hoàn thiện Cú pháp (`context.rs`):**
   - Bổ sung chỉ dẫn hệ thống: *"Always conclude your thoughts cleanly and close all open code blocks, lists, and JSON blocks properly"*, đảm bảo mọi khối code markdown luôn đóng trọn vẹn.
5. **Kiểm thử Toàn diện:**
   - Tạo mới `crates/agent/tests/continuation_tests.rs`: kiểm thử xác minh tính năng tự động nối tiếp khi gặp `finish_reason: "length"`.
   - Bổ sung test `test_gemini_stream_with_finish_reason_length` trong `crates/llm/tests/gemini_stream_tests.rs`.
   - Toàn bộ workspace test PASS 100%, Clippy 0 warnings, Frontend build sạch sẽ.

**Trạng thái:** Đã áp dụng

---

## [2026-10-03] Xử lý Lỗi Giới hạn Tần suất Gọi API (HTTP 429 Rate Limit) & Tự Động Hồi Phục Quota

**Bối cảnh:**
Khi người dùng chat với tác tử có kích hoạt công cụ tìm kiếm và duyệt web (`analyst`, `researcher`), tác tử có thể thực hiện chuỗi nhiều lượt gọi LLM trong cùng một hội thoại (ví dụ: Turn 1 gọi `web_search` -> Turn 2 gọi `web_fetch` -> Turn 3 tổng hợp câu trả lời). Với gói Miễn phí (Free Tier) của Google AI Studio cho mô hình `gemini-3.8-flash`, Google áp dụng hạn mức khắt khe **5 yêu cầu/phút (5 RPM)**. Khi số yêu cầu vượt quá 5 trong vòng 60 giây, Google trả về lỗi HTTP `429 RESOURCE_EXHAUSTED`:
`"Quota exceeded for metric: .../generate_content_free_tier_requests, limit: 5, model: gemini-3.8-flash. Please retry in 25.447400747s."` kèm `retryDelay: "25s"`.

Trước đây:
1. Hubbub chỉ retry với khoảng nghỉ ngắn 50ms, 100ms, 200ms (tổng cộng ~350ms), bỏ qua hoàn toàn khuyến cáo `retryDelay: 25s` của Google. Do đó toàn bộ các lần retry đều thất bại ngay tức thì.
2. Khi thất bại, app trả về chuỗi JSON thô `Agent error: Domain error: Internal error: API error (429): [{ "error": ... }]` hiển thị lên giao diện rất khó nhìn và không chỉ dẫn cho người dùng.

**Quyết định & Giải pháp Triển khai:**
1. **Trích xuất Động Thời gian Chờ (`extract_retry_delay`):**
   - Trong `crates/llm/src/errors.rs`, xây dựng hàm `extract_retry_delay` phân tích cả header `Retry-After` lẫn cấu trúc JSON của Google RPC (`/error/details[].retryDelay` và chuỗi `Please retry in Xs`).
2. **Cơ chế Tự Động Chờ Phục Hồi Hạn Mức (Smart 429 Backoff):**
   - Trong `OpenAiCompatAdapter` và `AnthropicAdapter`: khi gặp HTTP 429, nếu thời gian chờ `delay <= 35s` và chưa vượt quá số lần thử lại (`max_retries`), hệ thống sẽ tạm dừng và tự động ngủ đúng số giây máy chủ yêu cầu (+500ms dự phòng), sau đó tự động gửi lại request.
   - Nhờ đó, người dùng không bị gián đoạn hay nhận thông báo lỗi; yêu cầu sẽ tự động hoàn tất ngay khi quota phút mới của Google mở ra.
3. **Định dạng Thông báo Thân thiện (`format_api_error`):**
   - Nếu đã hết số lần retry hoặc thời gian chờ quá lâu, hệ thống chuyển đổi lỗi 429 thành thông điệp tiếng Việt rõ ràng, giải thích cụ thể nguyên nhân (gói Free Tier 5 RPM của model) và đưa ra gợi ý giải pháp (chờ Xs hoặc chuyển sang model có hạn mức cao hơn như `gemini-2.5-flash`).
4. **Bổ sung `gemini-2.5-flash` vào Danh mục Mô hình Nhanh:**
   - Trong `ModelSelector.tsx`, bổ sung lựa chọn `Gemini 2.5 Flash (15 RPM Free)` (hạn mức Free Tier cao gấp 3 lần `gemini-3.8-flash`) để người dùng dễ dàng chuyển đổi khi cần.
5. **Tối ưu Hiển thị Lỗi trên Giao diện (`ChatView.tsx`):**
   - Loại bỏ các tiền tố kỹ thuật lặp lại (`Agent error: Domain error: Internal error:`) khỏi Error Banner.

**Trạng thái:** Đã áp dụng

---

## [2026-10-03] Triển khai Phase 2: Security Policy Engine (`hubbub-policy`) & Live Web Tools (`hubbub-tools`)

**Bối cảnh:**
Để hỗ trợ Agent tìm kiếm thông tin chính xác theo thời gian thực trên Internet và đảm bảo an toàn tuyệt đối cho người dùng cục bộ, Hubbub cần triển khai Phase 2 theo MASTER_PLAN.md:
1. **Sprint 2.1 - Security Policy Engine (`hubbub-policy`):** Hệ thống chính sách bảo mật Zero-Trust độc lập hoàn toàn với I/O (theo Clean Architecture), bảo vệ chống SSRF, Path Traversal, truy cập tệp tùy ý và vi phạm quyền tác tử (agent capabilities).
2. **Sprint 2.2 - Live Web Tools (`hubbub-tools`):** Công cụ tìm kiếm web thời gian thực `web_search` (zero-config, không cần API key) và công cụ đọc nội dung trang web `web_fetch` (chuyển đổi HTML sang Markdown sạch, có cơ chế chống SSRF).

**Quyết định & Chi tiết Triển khai:**
1. **Sprint 2.1 - `hubbub-policy`:**
   - **`UrlGuard`:** Chặn toàn bộ tấn công SSRF (Server-Side Request Forgery). Kiểm tra scheme (chỉ chấp nhận `http` / `https`), phân giải DNS hoặc IP literal và chặn triệt để: IPv4 loopback (`127.0.0.0/8`, `0.0.0.0/8`), IPv4 private ranges (`10.0.0.0/8`, `172.16.0.0/12`, `192.168.0.0/16`), Cloud Metadata (`169.254.169.254`), Carrier-grade NAT (`100.64.0.0/10`), Link-local (`fe80::/10`, `169.254.0.0/16`), IPv6 Loopback (`::1`), IPv6 ULA (`fc00::/7`), và các hostname nội bộ (`localhost`, `*.local`, `*.internal`, `metadata.google.internal`).
   - **`PathGuard`:** Chặn toàn bộ tấn công Path Traversal. Chuẩn hóa đường dẫn, chặn `..`, đường dẫn tuyệt đối (`/`, `C:\`), null bytes, Windows Alternate Data Streams (`:`), và UNC paths (`\\`). Hỗ trợ an toàn đường dẫn `.` nội bộ thư mục gốc workspace.
   - **`PermissionChecker`:** Kiểm soát quyền hạn tác tử theo `AgentPermissions` và `NetworkPolicy` (`Open`, `SearchOnly`, `Allowlist`, `None`). Kiểm tra khớp pattern glob cho `fs_read` và `fs_write`.
   - **Kiểm thử Sprint 2.1:** 14 bài kiểm thử gồm unit tests và 5 bộ property-based testing (`proptest`) bao phủ toàn diện dải IP và đường dẫn độc hại.

2. **Sprint 2.2 - `hubbub-tools`:**
   - **`WebSearchTool` (`web_search`):** Sử dụng DuckDuckGo Lite (`https://lite.duckduckgo.com/lite/`), gửi truy vấn POST với `q=...&kl=wt-wt`. Hoạt động 100% zero-config không cần API key, phân tích cú pháp HTML trả về tiêu đề, tóm tắt và URL nguồn sạch sẽ cho Agent.
   - **`WebFetchTool` (`web_fetch`):** Tải nội dung trang web công khai có bảo vệ SSRF qua `UrlGuard`. Hạn chế redirect tối đa 5 bước với kiểm tra URL mỗi bước redirect, giới hạn dung lượng tải về (tối đa 2MB), trích xuất vùng nội dung chính (`<main>`, `<article>`, `<body>`), loại bỏ rác/scripts, và chuyển đổi sang định dạng Markdown sạch (tối đa 25,000 ký tự thân thiện với context window).
   - **`BuiltinToolHost`:** Điều phối và xuất schema công cụ chuẩn OpenAPI cho các tác tử.

3. **Tích hợp Tầng Tác tử (`hubbub-agent`):**
   - Trong `ToolRunner::execute_tool_call`, tích hợp hàm `check_policy` tự động kiểm tra `PermissionChecker`, `PathGuard`, và `UrlGuard` TRƯỚC KHI thực thi bất kỳ công cụ nào.
   - Nếu tác tử vi phạm chính sách (ví dụ tác tử offline cố gọi `web_search`, hoặc tác tử đọc đường dẫn ra ngoài workspace `../../windows`), hành động bị từ chối an toàn ngay tại tầng Policy mà không bao giờ kích hoạt I/O thực tế.
   - Bổ sung `crates/agent/tests/policy_enforcement_tests.rs` với 3 bài kiểm thử tích hợp (mock LLM + runtime) chứng minh sự can thiệp của Policy Engine.

**Kết quả & Tuân thủ:**
- Toàn bộ 50+ bài kiểm thử trong workspace vượt qua 100%.
- `cargo clippy --workspace --all-targets -- -D warnings` đạt 0 cảnh báo.
- Quy tắc mã nguồn: 0 `unwrap()`/`expect()` trong code nghiệp vụ, mọi file < 400 dòng, mọi hàm < 60 dòng.

**Trạng thái:** Đã áp dụng

---

## [2026-10-03] Xử lý Gemini Thought Signatures & Bảo toàn Ngữ cảnh Tool Calling (Multi-turn)

**Bối cảnh:**
Khi người dùng tương tác với các tác tử có kích hoạt công cụ (như `analyst`, `researcher` với công cụ `web_search`) sử dụng mô hình Google Gemini (`gemini-3.8-flash` hoặc các model thuộc thế hệ Gemini 3.x), model đã gọi công cụ `web_search`. Sau khi Hubbub thực thi công cụ và gửi lại lịch sử hội thoại lên API cho lượt tiếp theo (turn 2), Google Gemini từ chối yêu cầu với lỗi `400 INVALID_ARGUMENT`:
`Function call is missing a thought_signature in functionCall parts. This is required for tools to work correctly... Additional data, function call default_api:web_search, position 6.`

**Nguyên nhân:**
Thế hệ mô hình Gemini 3.x sử dụng cơ chế "Extended Thinking" và mã hóa trạng thái suy luận thành một token mật mã `thought_signature`. Trong luồng tương thích OpenAI (`/v1beta/openai/chat/completions`), token này được trả về trong `choice.delta.tool_calls[i].extra_content.google.thought_signature`. Khi client gửi tiếp lượt hội thoại có chứa kết quả công cụ, Gemini yêu cầu trong phần tin nhắn `assistant` trước đó, mỗi phần tử `tool_calls` phải kèm theo chính xác `extra_content` chứa `thought_signature` ban đầu. Trước đây, adapter chỉ lưu `{id, name, arguments}` và loại bỏ toàn bộ metadata này, dẫn đến việc Gemini báo lỗi 400.

**Quyết định & Giải pháp Triển khai:**
1. **Mở rộng Domain Entities & Ports (`hubbub-domain`):**
   - Bổ sung trường `extra_content: Option<serde_json::Value>` vào struct `ToolCall` trong port `hubbub_domain::ports::llm`, kèm các helper constructors `ToolCall::new` và `with_extra_content`.
   - Bổ sung `extra_content: Option<serde_json::Value>` vào `MessagePart::ToolCall` trong `hubbub_domain::entities::conversation` với thuộc tính `#[serde(default, skip_serializing_if = "Option::is_none")]`. Điều này đảm bảo tính tương thích tuyệt đối 100% với các bản ghi tin nhắn lịch sử đã lưu trước đó trong cơ sở dữ liệu SQLite.
2. **Bảo tồn và Tái tạo Ngữ cảnh qua SQLite (`hubbub-agent`):**
   - Trong `AgentRuntime::handle_assistant_tool_calls`, lưu giữ `tc.extra_content` vào `MessagePart::ToolCall` khi ghi xuống database.
   - Trong `ContextBuilder::build`, tái tạo `ToolCall` kèm `extra_content` nguyên vẹn từ các tin nhắn trong SQLite để đưa vào ngữ cảnh hội thoại cho các lượt gọi LLM tiếp theo.
3. **Thu thập & Tự Động Phục Hồi trong LLM Adapter (`hubbub-llm`):**
   - Trong `OpenAiStream`, trích xuất `extra_content` hoặc `thought_signature` từ SSE chunks (hỗ trợ cả ở cấp `tool_calls[i]`, `function`, và `delta`) và tích lũy vào `ToolCall`.
   - Trong `OpenAiCompatAdapter::build_openai_messages`:
     * Nếu `tc.extra_content` có sẵn, gửi lại nguyên vẹn `extra_content` lên API.
     * **Cơ chế Fallback Sentinel:** Nếu `extra_content` không có sẵn (ví dụ các tin nhắn cũ trong cơ sở dữ liệu được tạo trước khi có bản vá này) và mô hình đang gọi là Gemini, tự động chèn giá trị sentinel chính thức của Google: `skip_thought_signature_validator` (`{"google": {"thought_signature": "skip_thought_signature_validator"}}`). Giá trị này chỉ thị cho Gemini API bỏ qua bước xác thực chữ ký của lượt cũ mà không gây lỗi 400.
     * Đối với các model ngoài Gemini (như OpenAI, Anthropic), không chèn thêm trường thừa này.
4. **Cải tiến Công cụ Giả lập (`hubbub-tools`):**
   - Nâng cấp `web_search` trong `BuiltinToolHost` trả về nội dung mô phỏng chi tiết có chứa từ khóa truy vấn thay vì chuỗi ngắn "Web search tool ready.", giúp mô hình dễ dàng tổng hợp câu trả lời tự nhiên.
5. **Kiểm thử Toàn diện (Test-First & Regression):**
   - `crates/llm/tests/gemini_tests.rs`: 5 test case kiểm thử việc bảo toàn signature qua SSE stream, gửi round-trip, và fallback sentinel khi thiếu signature.
   - `crates/agent/tests/thought_signature_tests.rs`: Kiểm thử quy trình Agent Runtime thực thi tool call, lưu trữ signature xuống SQLite và tái tạo context cho lượt kế tiếp.
   - Toàn bộ 42/42 tests trong workspace PASS 100%, clippy đạt 0 warnings, frontend build thành công.

**Trạng thái:** Đã áp dụng

---

## [2026-10-03] Phân chia Agent theo Chức năng (Decoupled from Models) & Tối ưu Cuộn màn hình kiểu Gemini

**Bối cảnh:**
1. **Trải nghiệm cuộn khi streaming:** Trước đây màn hình liên tục auto-scroll xuống đáy theo từng token văn bản streaming, khiến người dùng bị cuốn theo và không thể đọc được phần đầu câu trả lời. Người dùng mong muốn màn hình đứng yên ở phần đầu nội dung câu hỏi/câu trả lời, trong khi nội dung tiếp tục được gen dài xuống phía dưới (giống giao diện Google Gemini).
2. **Kiến trúc Tác tử (Agents):** Trước đây Agent bị gắn cứng với tên Model (ví dụ `Gemini Analyst`, hoặc các agent khác bị gắn với model Claude/OpenAI khiến người dùng có key Gemini không dùng được). Người dùng yêu cầu: Agent phải được phân chia theo **chức năng / nhiệm vụ** chứ không phải theo Model. Một model duy nhất (như `gemini-3.8-flash`) có thể dùng để vận hành nhiều Agent khác nhau cho từng nhiệm vụ, đồng thời người dùng có thể đổi model hoặc tạo thêm Agent mới theo nhu cầu.

**Quyết định & Triển khai:**
1. **Cơ chế Giữ màn hình đứng yên kiểu Gemini (`ChatView.tsx`):**
   - Loại bỏ việc auto-scroll theo `streamingText` và `activeTools`.
   - Khi người dùng gửi tin nhắn hoặc khi streaming bắt đầu, tự động cuộn nhẹ nhàng để đưa câu hỏi và phần đầu câu trả lời vào vị trí đọc tự nhiên (`block: 'start'`).
   - Trong suốt quá trình Agent sinh nội dung (streaming tokens), màn hình giữ nguyên vị trí hoàn toàn, cho phép người dùng đọc liên tục từ đoạn văn đầu tiên trong khi nội dung mới xuất hiện mở rộng dần xuống phía dưới.
2. **Tái cấu trúc Tác tử theo Chức năng (`seed.rs`):**
   - Định nghĩa lại 6 tác tử mẫu chuyên biệt theo vai trò chức năng:
     - `analyst` - **Chuyên viên Phân tích**: Phân tích logic, tổng hợp dữ liệu, báo cáo chi tiết.
     - `developer` - **Kỹ sư Lập trình**: Lập trình đa ngôn ngữ, review code, debug, thiết kế kiến trúc.
     - `researcher` - **Trợ lý Nghiên cứu**: Thu thập thông tin web, đối chiếu đa nguồn, trích dẫn URL.
     - `writer` - **Biên tập & Soạn thảo**: Soạn thảo tài liệu kỹ thuật, bài viết, email, tóm tắt.
     - `tutor` - **Gia sư Đồng hành**: Hướng dẫn học tập từng bước (Socratic), giải thích trực quan.
     - `librarian` - **Thủ thư Quản lý Tệp**: Quản trị ghi chú, tệp tin nội bộ workspace.
   - Mặc định tất cả các tác tử chức năng này đều có thể chạy ngay với `gemini-3.8-flash` (hoặc bất kỳ model nào được cấu hình).
3. **Bộ chọn Mô hình Linh hoạt (`ModelSelector.tsx`):**
   - Tích hợp component chọn model trực tiếp trên thanh tiêu đề của ChatView.
   - Cho phép người dùng chuyển đổi mô hình AI chạy cho tác tử hiện tại (Google Gemini, OpenAI, Claude, Groq, Ollama) với 1 cú click.
4. **Tạo Tác tử Tùy chỉnh (`AgentModal.tsx` & Tauri IPC):**
   - Bổ sung nút "+ Tạo Agent" trên Sidebar cho phép người dùng tạo thêm bất kỳ Agent chuyên biệt nào (chọn tên nhiệm vụ, system prompt, model vận hành, và danh mục tools).
   - Bổ sung các lệnh Tauri IPC: `set_agent_model`, `upsert_agent`, `delete_agent`.
   - Tự động lưu trữ danh sách tác tử vào file `workspace/data/agents.json` để duy trì qua các phiên sử dụng.
5. **Kiểm thử Toàn diện:**
   - 37/37 tests PASS (`cargo test --workspace`).
   - `cargo clippy --workspace --all-targets -- -D warnings` đạt 0 cảnh báo.
   - `npm run build` trong `apps/desktop` đạt 0 lỗi.
   - Tất cả 11 files mã nguồn đều tuân thủ nghiêm ngặt giới hạn dưới 400 dòng code.

**Hệ quả:** Người dùng có thể sử dụng trọn vẹn sức mạnh của một mô hình (như Google Gemini) để phục vụ nhiều chuyên gia AI khác nhau; giao diện đọc khi streaming êm ái, tĩnh tại và tự nhiên như Google Gemini.

**Trạng thái:** Đã áp dụng

---

## [2026-10-03] Tái thiết kế Giao diện Chat theo phong cách Gemini/Antigravity & Tối ưu Chất lượng Phản hồi

**Bối cảnh:** 
1. Người dùng yêu cầu thay đổi thiết kế giao diện chat: Không muốn hiển thị dạng 2 bong bóng tin nhắn đối thoại thông thường (kiểu SMS). Tin nhắn người dùng hiển thị dạng box message bên phải, còn phản hồi của Agent hiển thị trực tiếp trên nền canvas tự nhiên giống giao diện Google Gemini hoặc Antigravity.
2. Nội dung trả lời của Agent trước đó bị cắt ngắn do ngữ cảnh lịch sử tồn tại các tin rỗng hoặc tin nhắn người dùng liên tiếp gây nhiễu luồng đàm thoại của LLM, đồng thời system prompt cần chỉ thị chi tiết về việc cung cấp câu trả lời hoàn chỉnh.

**Quyết định & Triển khai:**
1. **Tái thiết kế Giao diện Chat (`apps/desktop/src/components/ChatView.tsx`):**
   - **Tin nhắn Người dùng:** Hiển thị dạng box message gọn gàng, bo góc hiện đại (`rounded-2xl rounded-tr-xs`), nằm lệch bên phải (`ml-auto`), nền `bg-zinc-800` với viền tinh tế.
   - **Phản hồi của Tác tử (Agent):** Bỏ hoàn toàn khung viền và hộp nền bao quanh. Hiển thị văn bản trực tiếp trên nền tối (`bg-transparent`), độ rộng đọc tối ưu (`max-w-4xl`), căn lề chuẩn xác kèm biểu tượng avatar tác tử và tên gọi ở đầu.
   - **Tích hợp Bộ hiển thị Markdown Chuyên sâu (`MarkdownContent.tsx`):** Sử dụng `react-markdown` + `remark-gfm` định dạng tiêu đề (h1-h3), khối mã nguồn code block (syntax card), bảng biểu (tables), trích dẫn (blockquotes), và danh sách.
   - **Tách Component Chuẩn Quy tắc < 400 Dòng:** Tách `MarkdownContent.tsx` (96 dòng) giúp `ChatView.tsx` đạt 371 dòng (đáp ứng nghiêm ngặt quy định `< 400 lines`).
2. **Tối ưu Ngữ cảnh & Đảm bảo Tính Toàn vẹn Câu trả lời (`crates/agent/src/context.rs`, `seed.rs`):**
   - Chuẩn hóa lượt đàm thoại trong `ContextBuilder`: Tự động gộp các tin nhắn người dùng liên tiếp, loại bỏ các tin nhắn trợ lý rỗng phát sinh do lỗi mạng trước đó nhằm giữ cấu trúc `user <-> assistant` luôn sạch sẽ.
   - Bổ sung chỉ thị hệ thống yêu cầu trả lời chuyên sâu, đầy đủ mã nguồn và không ngắt ngang lời giải thích.
3. **Kiểm thử Toàn diện:**
   - Frontend `npm run build` thành công 100%.
   - Backend `cargo test --workspace` (37/37 tests PASS) và `cargo clippy` 0 warning.

**Hệ quả:** Giao diện ứng dụng sở hữu diện mạo chuyên nghiệp, rộng rãi và thoáng mắt tương đương các nền tảng AI hàng đầu; câu trả lời của Gemini Analyst đầy đủ, chi tiết và có định dạng Markdown đẹp mắt.

**Trạng thái:** Đã áp dụng

---

## [2026-10-03] Sửa lỗi Phản hồi Rỗng (Empty Response) khi Chat với Google Gemini

**Bối cảnh:** Khi người dùng gửi tin nhắn trò chuyện với Google Gemini (`gemini-3.8-flash`), API trả về thành công mã HTTP 200, tuy nhiên giao diện chat không hiển thị bất kỳ nội dung nào và tin nhắn kết quả của tác tử bị rỗng (`""`).

**Nguyên nhân gốc rễ (Root Cause):**
- Google Gemini endpoint OpenAI-compatible gửi trường `usage: { prompt_tokens, completion_tokens }` trong **mọi gói tin SSE chunk** cùng lúc với `choices: [{ delta: { content: "..." } }]`.
- Trong bộ phân tích SSE `OpenAiStream` (`crates/llm/src/adapters/openai_compat.rs`), logic cũ kiểm tra trường `usage` trước và thực hiện `return Some(Ok(LlmChunk::Usage(...)))` ngay lập tức, dẫn tới việc bỏ qua hoàn toàn phần `choices[0].delta.content` trong cùng chunk đó. Vì gói tin nào cũng có `usage`, tất cả các đoạn văn bản (text delta) đều bị loại bỏ, dẫn đến `text` tích lũy cuối cùng luôn là chuỗi rỗng `""`.

**Quyết định & Khắc phục:**
1. **Lưu trữ Trạng thái Usage Thay vì Return Ngắt Dòng:**
   - Trong `OpenAiStream`, bổ sung trường `last_usage: Option<LlmUsage>`. Khi nhận được thông tin `usage` trong chunk SSE (dù là ở mọi chunk như Gemini hay ở chunk cuối như OpenAI), lưu lại `last_usage = Some(...)` và tiếp tục phân tích `choices[0].delta` mà không ngắt dòng.
2. **Phát Sinh Sự Kiện Usage ở Cuối Luồng:**
   - Khi luồng stream nhận được `[DONE]` hoặc kết thúc dữ liệu (`Ok(None)`), hàm `finish_stream()` sẽ đẩy `LlmChunk::Usage(last_usage)` trước `LlmChunk::Done`.
   - Đảm bảo token usage được ghi nhận đúng và duy nhất 1 lần ở cuối tiến trình thực thi, không bị nhân bội và không làm rớt text streaming.
3. **Bổ sung Kiểm thử Hồi quy (`crates/llm/tests/gemini_stream_tests.rs`):**
   - Viết test `test_gemini_stream_with_usage_and_content_in_same_chunk` mô phỏng chính xác định dạng gói SSE chứa đồng thời `usage` và `content` của Gemini. Test xác minh thu thập trọn vẹn văn bản và ghi nhận đúng usage.
   - Toàn bộ 37/37 tests PASS, `cargo clippy` 0 warning.

**Hệ quả:** Khắc phục triệt để lỗi phản hồi rỗng; tác tử Gemini Analyst hiển thị văn bản streaming thời gian thực mượt mà và lưu trữ đầy đủ nội dung câu trả lời.

**Trạng thái:** Đã áp dụng

---

## [2026-10-03] Nâng cấp Mô hình Google Gemini sang `gemini-3.8-flash`

**Bối cảnh:** Khi người dùng gửi tin nhắn trò chuyện với tác tử `Gemini Analyst`, Google AI Studio API trả về lỗi 404: `This model models/gemini-2.0-flash is no longer available. Please update your code to use models/gemini-3.8-flash for the latest features and improvements`. Google đã chính thức ngừng cung cấp mô hình `gemini-2.0-flash` trên API và chuyển sang thế hệ mô hình mới `gemini-3.8-flash` (phát hành tháng 9/2026).

**Quyết định:**
1. **Cập nhật Cấu hình Mặc định (`AppConfig`):**
   - Đổi `default_model` của provider `gemini` thành `gemini-3.8-flash` trong `crates/app/src/config.rs`.
2. **Cập nhật Tác tử Gemini Analyst (`seed.rs`):**
   - Đổi `model` của tác tử `analyst` thành `gemini-3.8-flash` trong `crates/app/src/seed.rs`.
3. **Cập nhật Giao diện Cài đặt (`SettingsModal.tsx`):**
   - Cập nhật mô tả hiển thị của nhà cung cấp Gemini thành `Gemini 3.8 Flash, Gemini 1.5 Pro (Google AI Studio)`.
4. **Kiểm thử Hồi quy (`app_tests.rs`):**
   - Cập nhật unit test assertion kiểm tra model của `analyst` sang `gemini-3.8-flash`. Toàn bộ 36/36 tests PASS.

**Hệ quả:** Khắc phục triệt để lỗi 404 từ Google API; người dùng có thể trò chuyện với Gemini Analyst bằng `gemini-3.8-flash` mượt mà, phản hồi nhanh và hỗ trợ ngữ cảnh lớn 1M tokens.

**Trạng thái:** Đã áp dụng

---

## [2026-10-03] Bổ sung Hỗ trợ Google Gemini & Tác tử Gemini Analyst

**Bối cảnh:** Google Gemini (Gemini 2.0 Flash, Gemini 1.5 Pro) là họ mô hình AI tốc độ cao, hỗ trợ ngữ cảnh lớn (1M - 2M tokens) và có chi phí tối ưu. Google cung cấp endpoint REST tương thích chuẩn OpenAI tại `https://generativelanguage.googleapis.com/v1beta/openai`. Cần bổ sung Gemini vào hệ thống cấu hình provider, kho bảo mật, định tuyến runtime và giao diện người dùng.

**Quyết định:**
1. **Cấu hình Nhà Cung Cấp (`AppConfig`):**
   - Bổ sung cấu hình provider `gemini` vào `AppConfig::default()`, sử dụng `kind: "openai_compat"`, `base_url: "https://generativelanguage.googleapis.com/v1beta/openai"`, và model mặc định `gemini-2.0-flash`.
2. **Định Tuyến Mô Hình Tự Động (`AppService`):**
   - Tự động nhận diện các model có tiền tố `gemini*` để định tuyến yêu cầu về provider `gemini` và kiểm tra khóa bảo mật `gemini_api_key`.
3. **Cung Cấp Tác Tử Mẫu Mới (`Gemini Analyst`):**
   - Bổ sung tác tử `analyst` (Gemini Analyst) trong `seed.rs` sử dụng mô hình `gemini-2.0-flash`, tích hợp công cụ `web_search` và `report_write` với quyền truy cập mở.
4. **Giao Diện Người Dùng (SettingsModal & Sidebar):**
   - Đưa Google Gemini vào danh sách cấu hình API key trong `SettingsModal.tsx` với placeholder `AIzaSy...`.
   - Cập nhật Sidebar hiển thị icon `Sparkles` màu cyan cho tác tử Analyst và điều chỉnh lưới hiển thị 2x2 cân đối cho 4 tác tử.
5. **Kiểm thử Toàn diện:**
   - Bổ sung kiểm thử nạp cấu hình và khởi tạo provider `gemini` trong `app_tests.rs`. Toàn bộ 36/36 tests PASS, clippy 0 warning, build thành công 100%.

**Hệ quả:** Người dùng hiện có thể cấu hình API key Google Gemini và trò chuyện trực tiếp với tác tử Gemini Analyst mượt mà.

**Trạng thái:** Đã áp dụng

---

## [2026-10-02] Hoàn thành Sprint 1.6: React UI Chat MVP & Kết thúc Phase 1 (apps/desktop/src)

**Bối cảnh:** Triển khai giao diện người dùng Desktop Chat MVP trên React 19 + TypeScript + Vite + Tailwind CSS (`apps/desktop/src`), kết nối trực tiếp với backend Rust Core thông qua Tauri 2 IPC commands và streaming events `run_event`. Mục tiêu là cung cấp trải nghiệm chat đa tác tử mượt mà, hỗ trợ bộ gõ tiếng Việt (IME), hiển thị thời gian thực các bước thực thi công cụ (tools), quản lý API keys an toàn (write-only), chuyển đổi tác tử (Agent Switcher), và duy trì Gate 0 Benchmark Spike phục vụ kiểm thử hiệu năng.

**Quyết định:**
1. **Thiết kế Cấu trúc Component Modularity & Giới hạn < 400 Dòng:**
   - Phân tách giao diện thành các thành phần độc lập, chuyên biệt:
     - `Sidebar.tsx`: Danh sách lịch sử hội thoại, Agent Switcher (Researcher, Librarian, Tutor), nút tạo chat mới, nút mở cài đặt và chuyển đổi Gate 0 Spike.
     - `ChatView.tsx`: Danh sách tin nhắn với avatar phân biệt, hiển thị token delta streaming thời gian thực kèm con trỏ nhấp nháy, thẻ trạng thái thực thi công cụ (running/completed/failed), tự động cuộn xuống cuối (auto-scroll), và thanh soạn thảo (composer) tích hợp xử lý chống gửi sớm khi đang gõ IME tiếng Việt (`isComposing`).
     - `SettingsModal.tsx`: Hộp thoại cấu hình API keys cho 5 nhà cung cấp (OpenAI, Anthropic, OpenRouter, Groq, Ollama) với cơ chế bảo mật Write-Only (chỉ gửi ghi xuống Vault, không đọc ngược lại giao diện).
     - `BenchmarkSpike.tsx`: Tách toàn bộ module kiểm thử Gate 0 spike (kiểm tra gõ tiếng Việt, ảo hóa 1.000 tin nhắn qua `@tanstack/react-virtual`, mô phỏng stream 50t/s, Milkdown WYSIWYG editor) thành component riêng biệt.
     - `App.tsx`: Điều phối trung tâm kết nối trạng thái (conversations, messages, active agent, streaming text, active tools) qua Tauri IPC commands (`list_conversations`, `create_conversation`, `list_messages`, `send_message`, `cancel_run`) và lắng nghe sự kiện `run_event`.
2. **Xử lý An toàn Bộ gõ Tiếng Việt (IME Composition Guard):**
   - Lắng nghe `onCompositionStart` và `onCompositionEnd` trên khung nhập liệu để ngăn chặn sự kiện phím Enter kích hoạt gửi tin nhắn khi người dùng đang gõ bỏ dấu tiếng Việt (UniKey / EVKey).
3. **Cơ chế Streaming & Hiển thị Tác vụ Công cụ:**
   - Lắng nghe sự kiện `run_event` từ `TauriEventSink`:
     - `MessageDelta`: Tích lũy delta text vào khung phản hồi tức thời.
     - `ToolStarted`: Hiển thị thẻ card trạng thái công cụ kèm spinner đang chạy.
     - `ToolFinished`: Cập nhật trạng thái thành công/thất bại kèm tóm tắt kết quả.
     - `RunFinished`: Đồng bộ lại tin nhắn hoàn chỉnh từ SQLite store và giải phóng trạng thái streaming.
4. **Kết quả Nghiệm thu Toàn diện:**
   - Frontend: `tsc -b && vite build` biên dịch thành công 0 lỗi. `oxlint` sạch 0 lỗi.
   - Backend Rust: `cargo clippy --workspace --all-targets -- -D warnings` đạt 0 cảnh báo. `cargo fmt --all -- --check` đạt chuẩn 100%.
   - Toàn bộ test suite repository đạt **36/36 tests PASS** (Store: 10, LLM: 7, Agent: 9, Vault: 5, App: 5).
   - **Chính thức hoàn thành 100% Phase 1 (Chat MVP) qua toàn bộ 6 Sprints (1.1 -> 1.6).**

**Hệ quả:** Hoàn tất Phase 1. Ứng dụng Hubbub đã có đầy đủ khung nền từ Database, LLM Gateway, Agent Runtime, Keyring Vault, Composition Root, cho đến Giao diện React Desktop hoàn chỉnh. Sẵn sàng bước sang **Phase 2: Tool System & Workspace**.

**Trạng thái:** Đã áp dụng

---

## [2026-10-02] Hoàn thành Sprint 1.5: Composition Root & Desktop IPC Shell (hubbub-app & hubbub-desktop)

**Bối cảnh:** Triển khai tầng ứng dụng cốt lõi (`hubbub-app`) đóng vai trò Composition Root kết nối toàn bộ các Ports và Adapters đã xây dựng (`SqliteStore`, `KeyringVault`, `AgentRuntime`, `BuiltinToolHost`, `OpenAiCompatAdapter`, `AnthropicAdapter`). Đồng thời hoàn thiện lớp Tauri IPC Commands (`apps/desktop/src-tauri`) và bộ chuyển tiếp sự kiện streaming `TauriEventSink` để giao diện người dùng có thể kích hoạt các tác vụ chat đa tác tử an toàn.

**Quyết định:**
1. **Thiết lập Cấu hình Ứng dụng (`AppConfig`):**
   - Quản lý đường dẫn Workspace, provider mặc định, agent mặc định, và cấu hình chi tiết cho 5 nhà cung cấp: OpenAI, Anthropic, Ollama, OpenRouter, Groq.
   - Hỗ trợ lưu trữ và nạp tự động qua định dạng file `config.toml`.
2. **Triển khai Composition Root (`AppService`):**
   - Khởi tạo và liên kết các thành phần độc lập thành một dịch vụ đồng nhất (`init`), tự động nạp cơ sở dữ liệu SQLite tại thư mục workspace và kết nối kho bảo mật `KeyringVault`.
   - Cung cấp facade quản lý hội thoại (`create_conversation`, `list_conversations`, `get_conversation`, `list_messages`).
   - Cung cấp facade bảo mật API key (write-only: `set_provider_key`, `has_provider_key`, `delete_provider_key`).
   - Tích hợp điều phối Agent Run với cơ chế đăng ký và hủy tác vụ qua `CancellationToken` (`send_message`, `cancel_run`).
3. **Cung cấp Bộ Agent Mặc định (`seed.rs`):**
   - Tích hợp 3 agent cốt lõi sẵn dùng: `researcher` (Nghiên cứu & Web Search), `librarian` (Quản lý file nội bộ, ngắt kết nối mạng), và `tutor` (Gia sư học tập).
4. **Xây dựng Shell IPC Tauri (`commands.rs` & `events.rs`):**
   - Lớp Tauri shell tuân thủ tiêu chí "thin shell": chỉ đảm nhiệm định tuyến IPC và chuyển phát sự kiện, không chứa business logic.
   - `TauriEventSink`: Chuyển phát tức thời các sự kiện `RunEvent` (bao gồm `MessageDelta`, `ToolStarted`, `ToolFinished`, `RunFinished`) về frontend qua Tauri Event `run_event`.
   - Đăng ký đầy đủ 12 Tauri Commands: `health_check`, `get_version`, `list_conversations`, `create_conversation`, `get_conversation`, `list_messages`, `list_agents`, `set_provider_key`, `has_provider_key`, `delete_provider_key`, `send_message`, `cancel_run`.
5. **Kết quả nghiệm thu:** 5/5 test case trong `crates/app/tests/app_tests.rs` vượt qua 100%. Toàn bộ test suite toàn repo đạt **36/36 tests PASS**, `cargo clippy -D warnings` đạt 0 cảnh báo, `cargo fmt` chuẩn hóa, `tsc -b && vite build` và `oxlint` sạch lỗi.

**Hệ quả:** Hoàn tất Sprint 1.5. Sẵn sàng bước sang **Sprint 1.6: Hoàn thiện Giao diện React UI Chat MVP**.

**Trạng thái:** Đã áp dụng

---

## [2026-10-02] Hoàn thành Sprint 1.4: Keyring Vault (hubbub-vault)

**Bối cảnh:** Triển khai hạ tầng lưu trữ bí mật (API keys, credentials) an toàn cho ứng dụng trong `hubbub-vault`, cài đặt trait `SecretStore` từ `hubbub-domain`. Tuân thủ nghiêm ngặt nguyên tắc **Không lưu trữ plaintext trên đĩa**, sử dụng kho bảo mật bản địa của hệ điều hành (**Windows Credential Manager**), chạy bất đồng bộ an toàn qua `spawn_blocking`, và đảm bảo thông điệp lỗi tuyệt đối không làm lộ giá trị bí mật.

**Quyết định:**
1. **Cấu hình Native Backend cho Crate `keyring`:**
   - Kích hoạt tính năng native theo từng hệ điều hành: `windows-native` (Windows Credential Manager), `apple-native` (macOS Keychain), và `sync-secret-service` (Linux Secret Service).
2. **Triển khai `KeyringVault` với `tokio::task::spawn_blocking`:**
   - Các API hệ điều hành như `CredRead`, `CredWrite`, `CredDelete` là các hàm C/Windows API đồng bộ và có khả năng đọc/ghi đĩa. Đưa các lời gọi này vào `spawn_blocking` giúp bảo vệ vòng lặp Tokio worker threads không bị gián đoạn.
   - Xử lý mượt mà trường hợp xóa idempotent (xóa key không tồn tại vẫn trả về `Ok(())` thay vì trả lỗi).
3. **Triển khai `InMemoryVault`:**
   - Cung cấp triển khai dựa trên `Arc<RwLock<HashMap<String, String>>>` phục vụ các môi trường kiểm thử tự động (CI / test suite) và headless mà không làm ô nhiễm kho chứng chỉ thực của người dùng.
4. **Bảo mật An Toàn Thông Điệp Lỗi (`VaultError`):**
   - Định dạng lỗi chỉ hiển thị tên `key` và thông báo mã lỗi từ OS, **tuyệt đối không bao giờ chứa giá trị bí mật** (`value`).
5. **Kết quả nghiệm thu:** 5/5 test case trong `crates/vault/tests/vault_tests.rs` vượt qua 100% (bao gồm CRUD trên Windows Credential Manager thật, kiểm thử concurrent access, tiếng Việt có dấu và emoji trong key/secret, và kiểm tra không rò rỉ secret qua error log). Toàn bộ 31/31 tests toàn repo PASS, clippy 0 warning, fmt chuẩn hóa.

**Hệ quả:** Hoàn tất Sprint 1.4. Sẵn sàng bước sang **Sprint 1.5: Composition Root & App Layer (`hubbub-app` & Tauri Commands)**.

**Trạng thái:** Đã áp dụng

---

## [2026-10-02] Hoàn thành Sprint 1.3: Agent Runtime Core v1 (hubbub-agent & hubbub-testkit)

**Bối cảnh:** Triển khai lõi điều phối Agent (Agent Runtime Loop) trong `hubbub-agent` theo kiến trúc Clean Monolith. Agent crate chỉ phụ thuộc vào `domain` ports, độc lập hoàn toàn với I/O crate. Cần một bộ công cụ kiểm thử test-first (`hubbub-testkit`) mô phỏng LLM và công cụ độc lập, hỗ trợ hủy tác vụ an toàn bằng `CancellationToken`, kiểm soát ngân sách chạy (`BudgetTracker`), và kiểm thử toàn diện các tình huống biên (common edge cases).

**Quyết định:**
1. **Xây dựng Testkit Fixtures (`hubbub-testkit`):**
   - `FakeLlm`: Cài đặt `LlmProvider` cho phép nạp trước các phản hồi kịch bản (text deltas, tool calls, token usage, độ trễ chunk giả lập để test hủy luồng, lỗi mô phỏng).
   - `InMemoryEventSink`: Cài đặt `EventSink` thu thập toàn bộ sự kiện `RunEvent` phát ra cho giao diện UI.
   - `MockToolHost`: Cài đặt `ToolHost` cho phép đăng ký mock tool có độ trễ hoặc trả về lỗi, phục vụ kiểm thử phục hồi và hủy tác vụ.
2. **Quản lý Ngữ cảnh & Lời nhắc (`ContextBuilder`):**
   - Tự động bổ sung thời gian hiện tại (UTC) và chỉ thị ngôn ngữ (ưu tiên tiếng Việt chuẩn xác khi người dùng dùng tiếng Việt).
   - Ánh xạ lịch sử tin nhắn `Message` (với các phần `Text`, `ToolCall`, `ToolResult`) sang định dạng `LlmMessage` chuẩn.
3. **Giám sát Ngân sách Chạy (`BudgetTracker`):**
   - Thực thi kiểm tra giới hạn `max_steps`, `max_tokens`, và `timeout_s` tại mỗi bước trong vòng lặp.
   - Khi vượt ngưỡng, cập nhật trạng thái Run thành `Failed`, phát sự kiện `RunEvent::Error` và dừng vòng lặp an toàn, không bao giờ rơi vào vòng lặp vô tận.
4. **Vòng lặp Điều phối Chính (`AgentRuntime`):**
   - Tích hợp `CancellationToken` tại mọi điểm chờ bất đồng bộ (trước khi gọi LLM, trong khi stream từng chunk LLM, và trong khi thực thi tool).
   - Ghi nhận đầy đủ vòng đời của `Run` và các `Step` (loại `Llm` và `Tool`) vào `Store`.
   - Phục hồi có cấu trúc khi công cụ trả về lỗi (gửi kết quả lỗi về LLM để LLM phản hồi giải thích hoặc sửa sai).
5. **Cải tiến `Store::create_step`:**
   - Bổ sung `ON CONFLICT(id) DO UPDATE` trong SQLite để hỗ trợ cập nhật trạng thái bước thực thi từ `running` sang `completed`/`failed` kèm `duration_ms` và `output_json`.
6. **Kết quả nghiệm thu:** 9/9 test case trong `crates/agent/tests/runtime_tests.rs` vượt qua 100%. Toàn bộ test suite toàn repo: 26/26 tests PASS, `cargo clippy -D warnings` đạt 0 cảnh báo, `cargo fmt` đạt chuẩn.

**Hệ quả:** Hoàn tất Sprint 1.3. Sẵn sàng bước sang **Sprint 1.4: Keyring Vault (`hubbub-vault`)**.

**Trạng thái:** Đã áp dụng

---

## [2026-10-02] Hoàn thành Sprint 1.2: LLM Gateway & SSE Stream (hubbub-llm)

**Bối cảnh:** Triển khai hạ tầng giao tiếp với các mô hình ngôn ngữ lớn (LLM Gateway) cho Phase 1, hỗ trợ cả hai chuẩn API phổ biến nhất hiện nay: OpenAI-compatible (OpenAI, Ollama, OpenRouter, Groq) và Anthropic Claude Messages API. Quy trình tuân thủ nghiêm ngặt **Test-First**, xử lý luồng stream SSE ở cấp byte để chống rách ký tự UTF-8, và đảm bảo khả năng tích lũy tool call streaming.

**Quyết định:**
1. **Byte-level SSE Stream Reader (`SseEventReader`):**
   - Không parse từng chunk byte sang string ngay vì một byte split của mạng có thể cắt ngang một ký tự UTF-8 đa byte tiếng Việt (3 bytes) hoặc Emoji (4 bytes).
   - Tích lũy byte stream vào buffer và chỉ tách dòng theo `\n`, sau đó mới decode UTF-8 và parse event SSE `data:`.
2. **OpenAI-Compatible Adapter (`OpenAiCompatAdapter`):**
   - Hỗ trợ streaming text deltas (`content`).
   - Tích lũy streaming tool calls theo index (`accumulated_tool_calls` qua `BTreeMap`), tự động flush khi nhận `finish_reason: "tool_calls"`.
   - Thu thập usage token (`prompt_tokens`, `completion_tokens`, `total_tokens`) từ chunk cuối cùng.
   - Cơ chế Retry với Exponential Backoff (50ms * 2^(attempt-1)) đối với mã lỗi 5xx và 429; fail-fast ngay lập tức đối với lỗi xác thực (401/403).
3. **Anthropic Adapter (`AnthropicAdapter`):**
   - Chuyển đổi định dạng prompt: trích xuất riêng `system` message và ánh xạ các message còn lại thành `user`/`assistant`.
   - Ánh xạ tool definitions thành định dạng `input_schema` của Anthropic.
   - Hỗ trợ các event: `content_block_start` (tool_use), `content_block_delta` (text_delta, input_json_delta), `message_delta` (usage: output_tokens), `message_start` (usage: input_tokens).
   - Tương tự OpenAI, áp dụng retry với backoff cho lỗi 5xx/429.
4. **Quy trình Test-First với Wiremock:**
   - 7/7 bài kiểm tra tích hợp trong `crates/llm/tests/llm_tests.rs`:
     * Streaming text deltas & `[DONE]` marker.
     * Streaming usage metadata token counts.
     * Streaming tool call arguments accumulation across multiple chunks.
     * Anthropic SSE events (tool_use block, text delta, input/output tokens).
     * Tự động phục hồi khi gặp chunk SSE chứa JSON lỗi/rác mà không làm gián đoạn luồng stream.
     * Retry thành công sau khi gặp 500 Internal Server Error.
     * Dừng ngay lập tức (fail-fast, không retry vô nghĩa) khi nhận mã lỗi 401 Unauthorized.
5. **Kết quả nghiệm thu:** 100% test case vượt qua (7/7 tests, 0.07s), `cargo clippy -D warnings` đạt 0 cảnh báo, `cargo fmt` chuẩn hóa.

**Hệ quả:** Hoàn tất Sprint 1.2. Sẵn sàng bước sang **Sprint 1.3: Agent Runtime Core v1 (`hubbub-agent`)**.

**Trạng thái:** Đã áp dụng

---

## [2026-10-02] Hoàn thành Sprint 1.1: Database SQLite & Migrations (hubbub-store)

**Bối cảnh:** Triển khai tầng lưu trữ dữ liệu bền vững (Persistence Layer) đầu tiên của Phase 1, tuân thủ nghiêm ngặt nguyên tắc **Test-First** và bao quát các **common edge cases** (tiếng Việt, FTS5 unicode, emoji, concurrent access).

**Quyết định:**
1. **Thiết lập Migration SQLite (`0001_initial_schema.sql`):**
   - Bảng `conversations`: Khóa chính UUID v7, theo dõi thời gian `created_at` và `updated_at`, cờ `archived`.
   - Bảng `messages`: Thiết kế append-only, liên kết khóa ngoại CASCADE với `conversations`, lưu trữ các khối nội dung (`parts_json`) hỗ trợ mở rộng.
   - Bảng `runs` & `steps`: Lưu vết toàn diện vòng đời thực thi của Agent, chi phí token và thời gian chạy.
   - Bảng `documents`: Lưu trữ metadata của các tài liệu Workspace dạng Markdown.
   - Bảng ảo `messages_fts` & `documents_fts` (FTS5): Sử dụng bộ tách từ `unicode61` để hỗ trợ tìm kiếm toàn văn tiếng Việt có dấu.
2. **Quy trình Test-First & Edge Cases:**
   - Viết trước 10 test case trong `crates/store/tests/store_tests.rs`:
     * Tiêu đề và nội dung chứa đầy đủ dấu tiếng Việt phức tạp.
     * Emoji Unicode (🚀, 🎉) và khối mã nguồn Markdown.
     * Bản tin báo cáo rất dài (~60 KB văn bản).
     * Các trường hợp phân trang biên (`limit = 0`, `offset` vượt quá tổng số tin).
     * Tìm kiếm toàn văn FTS5 tiếng Việt chính xác với cụm từ có dấu.
     * Cập nhật tài liệu trùng đường dẫn (Upsert conflict resolution).
     * Truy cập đồng thời đa luồng (10 tác vụ async ghi và đọc đồng thời trong chế độ SQLite WAL).
3. **Triển khai `Store` Port:**
   - Cài đặt trait `hubbub_domain::ports::store::Store` trong struct `SqliteStore`.
   - Cấu hình SQLite Pool: Tự động kích hoạt `PRAGMA foreign_keys = ON;`, `journal_mode = WAL;`, `synchronous = NORMAL;`.
   - Cung cấp `SqliteStore::open` cho production và `SqliteStore::open_in_memory` (shared cache) cho kiểm thử độc lập.
4. **Kết quả nghiệm thu:** 10/10 test case vượt qua (0.06s), `cargo clippy -D warnings` đạt 100% không cảnh báo.

**Hệ quả:** Hoàn tất Sprint 1.1. Sẵn sàng bước sang **Sprint 1.2: LLM Gateway & SSE Stream (`hubbub-llm`)**.

**Trạng thái:** Đã áp dụng

---

## [2026-10-02] Nghiệm thu Gate 0: Chốt Stack Tauri 2 + React + Rust Core

**Bối cảnh:** Cần xác nhận kết quả kiểm thử thực tế về mức tiêu thụ RAM và hiệu năng của ứng dụng trên môi trường Windows 10/11 trước khi chuyển sang Phase 1.

**Quyết định:**
1. **Chính thức chốt stack:** Sử dụng **Tauri 2 + React 19 + TypeScript + Rust Core**. Không cần kích hoạt phương án dự phòng Flutter.
2. **Số liệu kiểm thử thực tế:**
   - Tiến trình **Hubbub Rust Core:** Chỉ chiếm **~3.2 MB** RAM.
   - Toàn bộ ứng dụng (bao gồm các tiến trình WebView2 runtime): Chiếm **~180 MB** RAM khi đang chạy trong môi trường phát triển (có Vite HMR và DevTools đi kèm) cùng lúc với tải 1.000 tin nhắn ảo hóa và Milkdown editor.
   - Mức RAM này đạt đúng ngưỡng mục tiêu (≤ 180MB) và dự kiến sẽ giảm thêm 20-40 MB khi xuất bản bản Release chính thức (strip binary + thin LTO + tối ưu CSS/JS).
3. **Cập nhật ADR-0001:** Bổ sung phần kết quả nghiệm thu Gate 0 vào tài liệu kiến trúc.

**Lý do:** Đáp ứng trọn vẹn tiêu chuẩn về RAM, hiệu năng và kiến trúc phân lớp Clean Architecture.

**Hệ quả:** Hoàn tất toàn bộ Phase 0. Đủ điều kiện khởi động **Phase 1: Chat MVP**.

**Trạng thái:** Đã áp dụng

---

## [2026-10-02] Dọn dẹp Tài liệu & Cập nhật Cấu hình Dự án

**Bối cảnh:** Sau khi tổng hợp thành công `MASTER_PLAN.md` và xây dựng nền tảng Phase 0, các file dự thảo kế hoạch cũ (`Plan_01.md`, `Plan_02.md`, `Plan_03.md`) không còn cần thiết và có thể gây nhiễu cho các Agent/Dev mới. Cần làm sạch thư mục `docs/` và bổ sung danh sách context files trong `.antigravity/config.json`.

**Quyết định:**
1. **Xoá bỏ các bản kế hoạch cũ:** Đã loại bỏ `docs/Plan_01.md`, `docs/Plan_02.md`, `docs/Plan_03.md`. Nguồn sự thật duy nhất về kế hoạch hiện tại là `docs/MASTER_PLAN.md`.
2. **Cập nhật `.antigravity/config.json`:** Khai báo toàn bộ các tài liệu cốt lõi (`MASTER_PLAN.md`, `DECISION_LOG.md`, `ARCHITECTURE.md`, `TESTING_GUIDE.md`, `AI_WORKFLOW.md`, `THREAT_MODEL.md`) làm `context_files` mặc định.
3. **Mở rộng command allowlist:** Bổ sung các lệnh `just`, `cargo nextest/deny`, `npx tauri` vào danh sách tự động duyệt an toàn.

**Lý do:** Giữ thư mục tài liệu tinh gọn, chính xác; giúp các Agent AI mới khi truy cập dự án luôn nạp đúng ngữ cảnh chuẩn hóa mà không bị phân tán bởi các bản nháp cũ.

**Hệ quả:** Thư mục `docs/` chỉ chứa tài liệu chính thức đang có hiệu lực.

**Trạng thái:** Đã áp dụng

---

## [2026-10-02] Hoàn thành Phase 0 Nền móng & Chuẩn bị Spike Gate 0

**Bối cảnh:** Cần dựng nền tảng monorepo (Rust workspace + Tauri 2 React app), hệ thống tài liệu tiêu chuẩn, và ứng dụng Gate 0 Spike để xác thực các tiêu chí kỹ thuật cốt lõi (gõ tiếng Việt, RAM WebView2, tải 1.000 tin nhắn, Milkdown editor).

**Quyết định:**
1. **Khởi tạo Rust Workspace:** Gồm 11 crates (`domain`, `policy`, `agent`, `llm`, `tools`, `mcp`, `store`, `workspace`, `vault`, `app`, `testkit`) và ứng dụng shell `hubbub-desktop` (Tauri 2).
2. **Thiết lập quy chuẩn code:** Khóa toolchain `rust-toolchain.toml`, `rustfmt.toml`, cấu hình `deny.toml` chặn copyleft/vulnerability, lints nghiêm ngặt ở cấp workspace (`forbid(unsafe_code)`, `deny(unwrap_used, expect_used, panic)`).
3. **Hoàn thiện tài liệu:** `ARCHITECTURE.md`, `TESTING_GUIDE.md` (chú trọng common edge cases), `AI_WORKFLOW.md`, `THREAT_MODEL.md`, và 7 bản ADR (`0001` đến `0007`).
4. **Xây dựng Gate 0 Spike App:** Ứng dụng tương tác đầy đủ 5 bài test (Gõ tiếng Việt, Virtualized 1.000 tin nhắn với `@tanstack/react-virtual`, Stream mô phỏng 50t/s, Milkdown WYSIWYG editor trên nền ProseMirror, Scorecard tổng hợp).
5. **Cấu hình CI & Task Runner:** Tạo `justfile` và workflow `.github/workflows/ci.yml`.

**Lý do:** Đảm bảo toàn bộ workspace compile sạch sẽ, cấu trúc phân lớp Clean Architecture được tuân thủ tuyệt đối, sẵn sàng chạy kiểm thử thực tế trên máy người dùng trước khi code tính năng Phase 1.

**Hệ quả:** Có thể khởi chạy `npm run tauri dev` ngay để thực hiện các bài test Gate 0.

**Trạng thái:** Đã áp dụng

---

## [2026-10-02] Chốt Master Plan v1.0

**Bối cảnh:** Dự án Hubbub cần một kế hoạch phát triển tổng thể thống nhất. Đã có 3 bản kế hoạch riêng biệt (Plan_01, Plan_02, Plan_03) với các hướng tiếp cận khác nhau.

**Quyết định:** Tạo MASTER_PLAN.md tổng hợp, kế thừa:
- Kiến trúc Clean/Hexagonal sâu từ Plan 02
- Tính thực tế, Gate 0, và quy trình AI-assisted từ Plan 03
- Loại bỏ kiến trúc Python sidecar từ Plan 01

**Các quyết định con:**
1. **Stack chính:** Tauri 2 + React + Rust Core (Gate 0 validate, fallback Flutter)
2. **Agent framework:** Tự viết bằng Rust (không dùng LangGraph/LangChain)
3. **Database:** SQLite + FTS5 (không vector DB ở v1)
4. **Voice:** Loại bỏ hoàn toàn khỏi v1 — tránh phức tạp không cần thiết
5. **Testing:** Test-first, ưu tiên common edge cases, danh sách cụ thể trong TESTING_GUIDE
6. **Decision Log:** Bắt buộc cập nhật mỗi khi có quyết định/thay đổi quan trọng
7. **Bug-first:** Luôn fix bug/vấn đề tiềm ẩn trước khi phát triển tính năng mới

**Lý do:** Cần kế hoạch khả thi cho 1 developer với AI hỗ trợ, kiến trúc tốt ngay từ đầu, dễ mở rộng, có tài liệu đầy đủ cho Dev/Agent mới tiếp cận.

**Hệ quả:** Bắt đầu Phase 0 (Foundation + Gate 0). Mọi thay đổi kiến trúc/stack phải viết ADR.

**Trạng thái:** Đã áp dụng
