[← English](../../README.md)

<h1 align="center">claudy</h1>

<p align="center"><b>Satu perintah. Provider apa pun. Kendali penuh atas Claude CLI.</b></p>

---

<p align="center">
Tidak perlu lagi pusing mengurus variabel lingkungan dan file konfigurasi.<br/>
Claudy memungkinkan Anda beralih antara Anthropic, Z.AI, OpenRouter, Ollama, dan endpoint kustom hanya dengan satu perintah — menjaga credential, mode konfigurasi, dan framework Claude tetap terisolasi dengan rapi per profil.
</p>

<p align="center">
<b>Multi-provider · Isolasi config · Channel bridge · Bridge agen lokal · Analytics penggunaan</b>
</p>

---

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

<img src="../assets/features-2048.png" alt="Why Claudy" width="100%" />

## Mengapa Claudy?

- **Launch multi-provider**: beralih antara built-in, Z.AI, OpenRouter alias, Ollama, dan endpoint kustom yang kompatibel dengan Anthropic.
- **Config modes**: isolasi konfigurasi Claude (`CLAUDE.md`, `settings.json`, skills/plugins/agents) per Mode.
- **Resolusi Provider Profile**: menyatukan built-in providers, custom providers, dan OpenRouter aliases.
- **Perilaku proses yang aman**: meneruskan SIGINT/SIGTERM ke proses Claude anak.
- **UX operasional**: perintah install/update/uninstall, pemeriksaan status, dan uji konektivitas.
- **Channel bridge opsional**: jalankan jembatan bot lokal untuk Telegram, Slack, dan Discord dengan prompt izin interaktif.
- **Agent MCP bridge**: delegasikan tugas dari Claude Code ke agen AI lokal lainnya (Gemini, Codex, Aider, dll.) melalui MCP.
- **Usage analytics**: serap data sesi dari `~/.claude/projects/`, lacak penggunaan token dan biaya per sesi/proyek, lihat dashboard lokal dengan rekomendasi.

## Provider yang Didukung

> Claudy terinspirasi dari [Clother](https://github.com/jolehuit/clother), launcher multi-provider berbasis Go untuk Claude CLI. Z.AI adalah provider yang paling banyak diuji. Jika Anda menemukan masalah dengan provider lain, silakan [buka issue](https://github.com/epicsagas/claudy/issues).

| Provider | Status | Catatan |
|---|---|---|
| Built-in (Anthropic) | ✅ Diuji | Default |
| Z.AI | ✅ Diuji | |
| OpenRouter alias | ⚠️ Eksperimental | Belum sepenuhnya diuji — laporkan masalah di GitHub |
| Ollama | ⚠️ Eksperimental | Belum sepenuhnya diuji — laporkan masalah di GitHub |
| Custom endpoint | ⚠️ Eksperimental | Belum sepenuhnya diuji — laporkan masalah di GitHub |

<img src="../assets/demo.gif" alt="demo" width="100%" />

## Persyaratan

- macOS atau Linux
- Rust toolchain (`cargo`) untuk build/install dari sumber
- Claude CLI terinstal dan tersedia di `PATH`

## Instalasi

### macOS / Linux (satu baris)

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/epicsagas/claudy/releases/latest/download/claudy-installer.sh | sh
```

### macOS Homebrew

```bash
brew tap epicsagas/tap
brew install claudy
```

### Windows PowerShell

```powershell
irm https://github.com/epicsagas/claudy/releases/latest/download/claudy-installer.ps1 | iex
```

### crates.io

**Binary siap pakai (cepat, tanpa kompilasi)**

```
cargo install cargo-binstall
cargo binstall claudy
```

**Platform apa pun — build dari sumber**

```
cargo install claudy
```

### Instal dari sumber Git

```bash
git clone https://github.com/epicsagas/claudy.git
cd claudy
cargo install --path .
```

## Penyiapan / Konfigurasi

```bash
claudy install
echo 'ZAI_API_KEY=your-key-here' >> ~/.claudy/secrets.env
claudy --version
claudy zai
```

## Mulai Cepat

<img src="docs/assets/demo.gif" alt="Quick Start" width="100%" />

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
claudy channel serve [--profile <profile>] [--listen <host:port>]
claudy channel start [--profile <profile>] [--listen <host:port>]
claudy channel stop
claudy channel restart [--profile <profile>] [--listen <host:port>]
claudy channel status
claudy channel add <telegram|slack|discord>
claudy channel remove <telegram|slack|discord>
claudy channel enable
claudy channel disable
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
claudy mcp run        # Mulai server MCP (dipanggil oleh Claude Code)
claudy mcp install    # Daftarkan claudy sebagai MCP server di pengaturan Claude Code
claudy mcp uninstall  # Hapus claudy dari pengaturan MCP Claude Code
```

`claudy mcp install` secara otomatis mendaftarkan dirinya di `~/.claude/settings.json`. Saat Anda membuat Mode dengan `claudy mode create <name>`, ia juga mendaftar di file pengaturan Mode. Tidak diperlukan konfigurasi manual.

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
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | claudy mcp run
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
claudy analytics sync-pricing      # Sinkronkan harga model dari models.dev dan halaman harga Anthropic
claudy analytics recalculate       # Hitung ulang semua biaya menggunakan data harga terbaru
claudy analytics insights          # Buat ringkasan JSON kompak untuk analisis LLM (default: 7 hari)
claudy analytics insights --days 14  # Analisis 14 hari terakhir
claudy analytics insights --from 2026-04-01 --to 2026-04-30  # Rentang tanggal spesifik
claudy analytics insights --project my-project  # Filter berdasarkan proyek
```

### Inside Claude Code: `/analytics-insights`

Cara tercepat untuk menganalisis penggunaan Anda adalah langsung di dalam Claude Code. Skill `analytics-insights` tersedia secara otomatis — cukup tanyakan secara alami:

```
> /analytics-insights
> /analytics-insights last 2 weeks
> analyze my usage patterns
> 사용 패턴 분석해줘
```

Claude menjalankan `claudy analytics insights`, menganalisis JSON, dan mengembalikan laporan terstruktur dengan:

- **Tren biaya** — pengeluaran harian/mingguan dengan deteksi lonjakan
- **Distribusi model** — model mana yang Anda gunakan dan berapa biayanya per sesi
- **Pola alat** — alat yang paling sering digunakan, tingkat kesalahan, observasi efisiensi
- **Performa cache** — rasio hit dan penghematan yang diperkirakan
- **Rekomendasi yang dapat ditindaklanjuti** — saran spesifik seperti "arahkan tugas sederhana ke turbo" dengan estimasi penghematan dalam dolar

Contoh output (lihat [`docs/examples/analytics-insights-sample.json`](docs/examples/analytics-insights-sample.json) untuk data mentah):

```
#### Summary
81 sessions, $481 total spend at an average of $68.7/day. Costs trending
sharply upward — last 3 weekdays averaged $97/day.

#### Recommendations
1. Route simple tasks to glm-5-turbo — est. savings: ~$90/month
2. Investigate $1.91/turn outlier session (6x average cost-per-turn)
3. Reduce harness overhead — TaskCreate/Update accounted for ~1,000 calls
```

Tanpa perintah manual, tanpa berpindah konteks. Tanyakan kepada Claude tentang penggunaan Anda dan dapatkan jawaban secara instan.

Analytics melacak:

- **Tokens**: Tren terperinci token input, output, dan cache selama 30 hari terakhir, dikelompokkan berdasarkan model dan tanggal.
- **Tools**: Analisis distribusi yang menunjukkan alat mana yang paling sering digunakan Claude, termasuk jumlah panggilan, tingkat kesalahan, dan waktu eksekusi rata-rata.
- **Cost**: Estimasi real-time biaya penggunaan berdasarkan harga token aktual, termasuk perkiraan harian/mingguan/bulanan dan deteksi tren (increasing/stable/decreasing).
- **Tips (Recommendations)**: Saran optimasi berbasis data, seperti mendeteksi sesi berbiaya tinggi, menyarankan Haiku untuk tugas sederhana, dan mengidentifikasi percakapan panjang yang bisa mendapat manfaat dari ringkasan konteks.
- **Projects**: Secara otomatis memetakan UUID sesi yang tidak mudah dibaca ke nama folder proyek yang dapat dibaca manusia untuk konteks yang lebih baik.

Data disimpan dalam database SQLite lokal di `~/.claudy/analytics/`. Dashboard berjalan sebagai aplikasi Tauri 2 + Svelte lokal berkinerja tinggi. Gunakan tombol **[Sync]** di dashboard untuk langsung menyegarkan data dari riwayat Claude CLI Anda.

<img src="../assets/analytics-dashboard.png" alt="Analytics Dashboard" width="100%" />

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
