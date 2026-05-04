[← English](../../README.md)

<p align="center">
  <a href="README_ko.md">🇰🇷 한국어</a> •
  <a href="README_zh.md">🇨🇳 中文</a> •
  <a href="README_ja.md">🇯🇵 日本語</a> •
  <a href="README_de.md">🇩🇪 Deutsch</a> •
  <a href="README_fr.md">🇫🇷 Français</a> •
  <a href="README_es.md">🇪🇸 Español</a> •
  <a href="README_hi.md">🇮🇳 हिन्दी</a> •
  <a href="README_pt.md">🇧🇷 Português</a> •
  <a href="README_id.md">🇮🇩 Bahasa</a> •
  <a href="README_ar.md">🇸🇦 العربية</a>
</p>

<h1 align="center">claudy</h1>

<p align="center"><b>Launcher multi-provider modern untuk Claude CLI.</b></p>

---

<p align="center">
Claudy membantu Anda menjalankan Claude terhadap berbagai provider dengan satu antarmuka perintah yang konsisten, sekaligus menjaga credential provider dan overlay konfigurasi Claude tetap terorganisir di bawah satu direktori home.
</p>

<p align="center">
    <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/rust-1.92%2B-orange.svg" alt="rust-lang" /></a>
    <a href="https://crates.io/crates/claudy"><img src="https://img.shields.io/crates/v/claudy.svg" alt="crates.io" /></a>
    <a href="https://crates.io/crates/claudy"><img src="https://img.shields.io/crates/d/claudy.svg" alt="Downloads" /></a>
    <a href="../../LICENSE"><img src="https://img.shields.io/badge/License-Apache%202.0-blue.svg" alt="License" /></a>
    <a href="https://buymeacoffee.com/epicsaga"><img src="https://img.shields.io/badge/Buy%20Me%20a%20Coffee-FFDD00?style=flat&logo=buy-me-a-coffee&logoColor=black" alt="Buy Me a Coffee" /></a>
</p>

---

<img src="../../assets/features-2048.png" alt="Why Claudy" width="100%" />

## Mengapa Claudy?

- **Launch multi-provider**: beralih antara built-in, Z.AI, OpenRouter alias, Ollama, dan endpoint kustom yang kompatibel dengan Anthropic.
- **Config modes**: isolasi konfigurasi Claude (`CLAUDE.md`, `settings.json`, skills/plugins/agents) per Mode.
- **Resolusi Provider Profile**: menyatukan built-in providers, custom providers, dan OpenRouter aliases.
- **Perilaku proses yang aman**: meneruskan SIGINT/SIGTERM ke proses Claude anak.
- **UX operasional**: perintah install/update/uninstall, pemeriksaan status, dan uji konektivitas.
- **Channel bridge opsional**: jalankan jembatan bot lokal untuk Telegram, Slack, dan Discord dengan prompt izin interaktif.
- **Agent MCP bridge**: delegasikan tugas dari Claude Code ke agen AI lokal lainnya (Gemini, Codex, Aider, dll.) melalui MCP.
- **Usage analytics**: serap data sesi dari `~/.claude/projects/`, lacak penggunaan token dan biaya per sesi/proyek, lihat dashboard lokal dengan rekomendasi.

## Status Provider

> Claudy terinspirasi dari [Clother](https://github.com/jolehuit/clother), launcher multi-provider berbasis Go untuk Claude CLI. Hanya **Z.AI provider yang telah diuji sepenuhnya**. Semua provider alternatif lainnya bersifat eksperimental dan belum diuji — gunakan dengan risiko Anda sendiri.

| Provider | Status | Catatan |
|---|---|---|
| Built-in (Anthropic) | ✅ Diuji | Default |
| Z.AI | ✅ Diuji | Tervalidasi penuh |
| OpenRouter alias | ⚠️ Eksperimental | Belum diuji — gunakan dengan risiko sendiri |
| Ollama | ⚠️ Eksperimental | Belum diuji — gunakan dengan risiko sendiri |
| Custom endpoint | ⚠️ Eksperimental | Belum diuji — gunakan dengan risiko sendiri |

## Persyaratan

- macOS atau Linux
- Rust toolchain (`cargo`) untuk build/install dari sumber
- Claude CLI terinstal dan tersedia di `PATH`

## Instalasi

### Instal dari crates.io

**Binary siap pakai (cepat, tanpa kompilasi)**

```
cargo install cargo-binstall
cargo binstall claudy
```

**Platform apa pun — build dari sumber**

```
cargo install claudy
```

**MacOS homebrew**

```bash
brew tap epicsagas/tap
brew install claudy
```

### Instal dari sumber lokal

```bash
git clone https://github.com/epicsagas/claudy.git
cd claudy
cargo install --path .
```

### Verifikasi

```bash
claudy --help
claudy --version
```

## Mulai Cepat

```bash
# 1) Tampilkan daftar profiles yang tersedia/terselesaikan
claudy ls

# 2) Konfigurasikan credential secara interaktif
claudy setup

# 3) Periksa detail satu Profile
claudy show <profile>

# 4) Jalankan Claude dengan sebuah Profile
claudy <profile> [claude-args...]
```

## Konsep Inti

### Profile

Target peluncuran yang menyelesaikan metadata provider + strategi autentikasi (built-in provider, OpenRouter alias, atau custom provider).

### Mode

Direktori konfigurasi Claude bernama di `~/.claudy/modes/<name>/`.

Ketika Anda menjalankan:

```bash
claudy <profile> <mode> [args...]
```

Claudy menetapkan:

```bash
CLAUDE_CONFIG_DIR=~/.claudy/modes/<mode>/
```

sehingga Claude membaca file konfigurasi khusus Mode.

Mode juga sangat cocok untuk menjalankan **framework dan toolkit Claude khusus** yang membawa `CLAUDE.md`, skill, agen, atau pengaturan mereka sendiri — seperti [gstack](https://github.com/garrytan/gstack), [superpowers](https://github.com/obra/superpowers), [ecc](https://github.com/affaan-m/everything-claude-code), atau harness kustom apa pun. Daripada mengotori konfigurasi default, isolasi setiap framework di Mode-nya sendiri:

```bash
# Buat Mode khusus untuk framework
claudy mode create gstack

# Salin atau buat symlink konfigurasi framework ke direktori Mode
cp -r /path/to/gstack/.claude/. ~/.claudy/modes/gstack/

# Jalankan Claude dengan framework tersebut aktif
claudy <profile> gstack
```

Setiap direktori Mode adalah `CLAUDE_CONFIG_DIR` yang mandiri, sehingga framework tidak pernah saling berkonflik satu sama lain maupun dengan konfigurasi default Anda.

## Referensi Perintah

### Perintah Utama

- `claudy ls` (alias: `list`): tampilkan daftar profiles yang dikonfigurasi/terselesaikan.
- `claudy setup [provider]` (alias: `config`): pengaturan provider secara interaktif.
- `claudy show <profile>` (alias: `info`): tampilkan detail provider yang terselesaikan.
- `claudy ping [profile]` (alias: `test`): uji konektivitas provider.
- `claudy doctor` (alias: `status`): tampilkan versi, path, dan jumlah Profile.
- `claudy sync` (alias: `install`): instal/sinkronkan binary claudy.
- `claudy update`: perbarui claudy.
- `claudy uninstall`: hapus file yang terinstal.
- `claudy mode <action> [name]`: kelola Claude config modes.
- `claudy channel <subcommand>`: kelola Channel bridge.
- `claudy mcp`: jalankan sebagai server MCP untuk jembatan agen.
- `claudy analytics <subcommand>`: dashboard analytics penggunaan.

### Perintah Mode

```bash
claudy mode create <name>
claudy mode ls
claudy mode rm <name>
```

Aturan nama Mode: `[a-z0-9][a-z0-9_-]*` (`mode` dicadangkan).

### Perintah Channel (jembatan opsional)

```bash
claudy channel start [--profile <profile>] [--listen <host:port>]
claudy channel stop
claudy channel restart
claudy channel status
claudy channel add <telegram|slack|discord>
claudy channel remove <telegram|slack|discord>
claudy channel enable <telegram|slack|discord>
claudy channel disable <telegram|slack|discord>
```

`channel add` memandu Anda melalui bot token, pengguna yang diizinkan, pemetaan Profile dan Mode.

#### Platform yang didukung

| Platform | Penyerapan | Tombol interaktif | Catatan |
|----------|-----------|-------------------|---------|
| Telegram | Long-polling + webhook | Inline keyboard | Paling lengkap |
| Slack | Event subscription webhook | Block Kit actions | Terverifikasi HMAC-SHA256 |
| Discord | Interaction webhook | Action row components | Terverifikasi Ed25519 |

#### Perintah bot Channel

Setelah berjalan, bot merespons perintah-perintah ini dalam obrolan:

- `/help` — Tampilkan perintah yang tersedia
- `/cancel` — Batalkan tugas saat ini
- `/model` — Ganti model Claude (tombol interaktif)
- `/yolo` — Toggle izin auto-allow
- `/status` — Tampilkan status sesi, Profile, Mode, cabang git, dan penggunaan token
- `/sessions` — Tampilkan daftar sesi Claude terbaru (dengan tombol switch)
- `/projects` — Tampilkan daftar proyek (dengan tombol telusuri)
- `/new` — Mulai sesi baru
- `/history` — Tampilkan riwayat sesi terbaru

Kirim teks lain apa pun untuk berbicara langsung dengan Claude.

#### Prompt izin (Permission prompts)

Saat Claude meminta persetujuan untuk menggunakan alat (menjalankan perintah, mengedit file, dll.), bot mengirimkan prompt Allow/Deny interaktif ke obrolan Anda. Mengetuk tombol mengirimkan respons kembali ke Claude dan pemrosesan berlanjut secara otomatis.

#### Secrets

Simpan credential di `~/.claudy/secrets.env`:

```env
TELEGRAM_BOT_TOKEN=...
SLACK_BOT_TOKEN=xoxb-...
SLACK_SIGNING_SECRET=...
DISCORD_BOT_TOKEN=...
DISCORD_APPLICATION_ID=...
DISCORD_PUBLIC_KEY=...
```

### Agent MCP bridge

Jalankan `claudy mcp` untuk memulai server MCP berbasis stdio yang memungkinkan Claude Code mendelegasikan tugas ke agen AI coding lokal lainnya.

```bash
claudy mcp
```

Pada pertama kali dijalankan, claudy secara otomatis mendaftarkan dirinya di `~/.claude/settings.json`. Saat Anda membuat Mode dengan `claudy mode create <name>`, ia juga mendaftar di file pengaturan Mode. Tidak diperlukan konfigurasi manual.

Untuk mendaftar secara manual (atau di `.claude/settings.json` tingkat proyek):

```json
{
  "mcpServers": {
    "claudy": {
      "command": "claudy",
      "args": ["mcp"]
    }
  }
}
```

Claude Code akan melihat alat `ask_agent` yang mengekspos semua agen yang terinstal.

#### Contoh penggunaan

Setelah terdaftar, Claude Code dapat mendelegasikan tugas seperti ini:

```
> Ask gemini to review the error handling in src/api.rs
> Ask codex to write unit tests for the parser module
> Ask aider to refactor the database layer
```

Claude Code memilih agen yang tepat, meneruskan prompt, dan mengembalikan hasilnya. Anda juga dapat menentukan direktori kerja:

```json
{ "agent": "gemini", "prompt": "Explain this module", "working_directory": "/path/to/project" }
```

#### Verifikasi registrasi MCP

```bash
# Periksa apakah claudy sudah terdaftar
cat ~/.claude/settings.json | grep -A3 claudy

# Uji server MCP secara manual
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | claudy mcp
```

#### Agen yang didukung (auto-detected dari PATH)

| Agent | Binary | Headless command |
|-------|--------|-----------------|
| Gemini CLI | `gemini` | `gemini -p "..." --output-format text` |
| Codex CLI | `codex` | `codex exec "..."` |
| Cursor Agent | `agent` | `agent -p "..." --output-format text` |
| GitHub Copilot | `copilot` | `copilot -p "..."` |
| OpenCode | `opencode` | `opencode run "..."` |
| Cline | `cline` | `cline -y "..."` |
| Aider | `aider` | `aider --message "..."` |
| Goose | `goose` | `goose run "..."` |
| Amp | `amp` | `amp --non-interactive "..."` |
| Droid | `droid` | `droid exec "..."` |
| Kiro | `kiro-cli` | `kiro-cli chat --no-interactive --trust-all-tools "..."` |
| Junie | `junie` | `junie "..."` |
| Kimi Code | `kimi` | `kimi "..."` |
| Mistral Vibe | `vibe` | `vibe "..."` |
| Qwen Code | `qwen-code` | `qwen-code "..."` |
| Crush | `crush` | `crush "..."` |
| Groq Code | `groq-code` | `groq-code --prompt "..."` |
| Plandex | `plandex` | `plandex tell "..."` |
| Kilo Code | `kilo` | `kilo "..."` |
| OpenHands | `openhands` | `openhands "..."` |

#### Agen kustom

Tambahkan agen di `~/.claudy/config.yaml`:

```json
{
  "agents": {
    "my-agent": {
      "binary": "my-agent",
      "args": ["--prompt", "{prompt}", "--no-interactive"],
      "description": "My custom agent",
      "timeout": 180
    }
  }
}
```

Kunci yang sama dengan agen bawaan akan menimpa nilai default-nya. `{prompt}` dalam `args` diganti dengan tugas yang sebenarnya.

### Perintah Analytics

> **Catatan**: Fitur analytics masih dalam pengembangan. Jumlah token, estimasi biaya, dan metrik lainnya mungkin belum sepenuhnya akurat. Perbaikan diharapkan pada rilis mendatang.

```bash
claudy analytics dashboard         # Buka dashboard analytics lokal (Tauri 2)
claudy analytics ingest            # Serap data sesi dari ~/.claude/projects/
claudy analytics ingest --full     # Serap ulang semua file (abaikan checkpoints)
claudy analytics ingest --project my-project  # Serap proyek tertentu
claudy analytics recommend         # Tampilkan rekomendasi penggunaan di CLI
claudy analytics export            # Ekspor data analytics (JSON, default 30 hari)
claudy analytics export --format csv --days 7  # Ekspor sebagai CSV untuk 7 hari terakhir
```

Analytics melacak:

- **Tokens**: Tren terperinci token input, output, dan cache selama 30 hari terakhir, dikelompokkan berdasarkan model dan tanggal.
- **Tools**: Analisis distribusi yang menunjukkan alat mana yang paling sering digunakan Claude, termasuk jumlah panggilan, tingkat kesalahan, dan waktu eksekusi rata-rata.
- **Cost**: Estimasi real-time biaya penggunaan berdasarkan harga token aktual, termasuk perkiraan harian/mingguan/bulanan dan deteksi tren (increasing/stable/decreasing).
- **Tips (Recommendations)**: Saran optimasi berbasis data, seperti mendeteksi sesi berbiaya tinggi, menyarankan Haiku untuk tugas sederhana, dan mengidentifikasi percakapan panjang yang bisa mendapat manfaat dari ringkasan konteks.
- **Projects**: Secara otomatis memetakan UUID sesi yang tidak mudah dibaca ke nama folder proyek yang dapat dibaca manusia untuk konteks yang lebih baik.

Data disimpan dalam database SQLite lokal di `~/.claudy/analytics/`. Dashboard berjalan sebagai aplikasi Tauri 2 + Svelte lokal berkinerja tinggi. Gunakan tombol **[Sync]** di dashboard untuk langsung menyegarkan data dari riwayat Claude CLI Anda.

<img src="../../assets/analytics-dashboard.png" alt="Analytics Dashboard" width="100%" />

## Tata Letak File dan Direktori

Secara default, Claudy menyimpan data di:

```text
~/.claudy/
```

File/direktori penting:

- `config.yaml`: konfigurasi provider + channel + agent.
- `secrets.env`: credential provider/bot.
- `launchers.json`: manifest launcher/symlink.
- `modes/`: Claude config modes.
- `session-patches/`: penyimpanan patch sesi.
- `channel/`: status runtime channel (`pid`, sesi, log audit).
- `analytics/`: database SQLite analytics dan checkpoints.
- `cache/update.json`: cache metadata pembaruan.

## Variabel Lingkungan

- `CLAUDY_HOME`: timpa direktori home Claudy (default: `~/.claudy`).
- `CLAUDE_CONFIG_DIR`: diatur otomatis oleh Claudy saat meluncurkan dengan Mode.

## Alur Kerja Umum

### Konfigurasi dan jalankan provider

```bash
claudy setup
claudy <profile>
```

### Gunakan Mode dengan provider

```bash
claudy mode create work
claudy <profile> work --yolo
```

> `--yolo` adalah singkatan claudy untuk `--dangerously-skip-permissions`.

### Menjalankan Framework Claude Khusus dalam Mode-nya Sendiri

Framework seperti gstack, superpowers, atau ecc membawa `CLAUDE.md`, skill, dan agen mereka sendiri. Jalankan secara terisolasi:

```bash
# Pengaturan sekali: buat Mode dan muat konfigurasi framework
claudy mode create gstack
cp -r /path/to/gstack/.claude/. ~/.claudy/modes/gstack/

# Penggunaan harian: jalankan Claude dengan framework aktif
claudy <profile> gstack
```

Beralih antar framework tanpa mengubah konfigurasi default:

```bash
claudy <profile> gstack      # framework gstack aktif
claudy <profile> superpowers # framework superpowers aktif
claudy <profile>             # konfigurasi default, tidak berubah
```

### Delegasikan tugas ke agen lain melalui MCP

```bash
# 1) Pastikan MCP sudah terdaftar (terjadi otomatis pada `claudy mcp` pertama)
claudy mcp

# 2) Di Claude Code, minta untuk mendelegasikan ke agen yang terinstal:
#    "Ask gemini to analyze this error"
#    "Ask aider to refactor the auth module"
```

### Diagnosis status instalasi/konfigurasi

```bash
claudy doctor
claudy ping
```

## Pemecahan Masalah

- **`profile not recognized`**: jalankan `claudy ls` dan pilih ID Profile yang terdaftar.
- **Profile dengan status `not configured`**: jalankan `claudy setup <provider>` untuk menambahkan credential.
- **Channel status tidak sehat**: jalankan `claudy channel status`, lalu restart dengan `claudy channel stop` dan `claudy channel start`.
- **Bot Channel tidak merespons**: periksa `~/.claudy/channel/logs/server.log` untuk kesalahan. Verifikasi bot token di `~/.claudy/secrets.env` dan pastikan `allowed_users` mencakup ID pengguna obrolan Anda.
- **Permission prompt tidak muncul**: pastikan Claude CLI tidak berjalan dengan `--dangerously-skip-permissions`. Prompt hanya terpicu saat Claude membutuhkan persetujuan eksplisit untuk penggunaan alat.
- **Binary tidak ditemukan setelah instalasi**: pastikan direktori bin Claudy ada di `PATH`, lalu restart shell Anda.
- **Agen tidak muncul di MCP**: pastikan binary agen ada di `PATH` (`which gemini`). Hanya agen yang terinstal yang muncul di `tools/list`.
- **Agent timeout**: tingkatkan timeout di field agents `config.yaml` (default: 120s).
- **MCP belum terdaftar**: jalankan `claudy mcp` sekali secara manual, atau periksa entri `mcpServers.claudy` di `~/.claude/settings.json`.
- **Output agen terpotong**: stdout agen dibatasi 10MB. Untuk output besar, arahkan agen untuk menulis ke file.
- **Data Analytics hilang**: jalankan `claudy analytics ingest` untuk mengisi dari `~/.claude/projects/`. Gunakan `--full` untuk menyerap ulang semuanya.

## Pengembangan

```bash
cargo build
cargo test
cargo fmt
cargo clippy -- -D warnings

# Uji analytics backend (menggunakan DB lokal)
cargo run --example test_dashboard --features analytics-ui

# Luncurkan analytics dashboard (memerlukan fitur analytics-ui)
cargo run --features analytics-ui -- analytics dashboard
```

## Berkontribusi

Kontribusi sangat disambut! Berikut cara memulainya:

1. Fork repositori dan buat branch fitur.
2. Lakukan perubahan Anda dengan pengujian yang sesuai.
3. Jalankan `cargo test && cargo clippy -- -D warnings` sebelum mengirimkan.
4. Buka Pull Request di https://github.com/epicsagas/claudy.

Laporan bug dan permintaan fitur disambut melalui [GitHub Issues](https://github.com/epicsagas/claudy/issues).

## Penghargaan

Proyek ini terinspirasi dari [Clother](https://github.com/jolehuit/clother), launcher multi-provider berbasis Go untuk Claude CLI. Claudy adalah implementasi Rust yang independen, dirancang ulang dari awal dengan RAII-based session guards, penerusan sinyal, launcher symlinks, dan integrasi ekosistem yang mendalam termasuk **Channel Bridge berfitur lengkap** (Telegram/Slack/Discord), **Agent MCP Bridge** untuk delegasi lintas agen, dan **Analytics Dashboard berkinerja tinggi** yang dibangun dengan Tauri 2. Penambahan-penambahan ini mencerminkan transisi Claudy dari launcher sederhana menjadi toolkit operasional komprehensif bagi pengguna Claude CLI.

## Lisensi

[Apache-2.0](../../LICENSE)
