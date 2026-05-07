[← English](../../README.md)

<h1 align="center">claudy</h1>

<p align="center"><b>1つのコマンド。あらゆる Provider。Claude CLI の完全な制御。</b></p>

---

<p align="center">
環境変数や設定ファイルの管理から解放されましょう。<br/>
Claudy を使えば、Anthropic、Z.AI、OpenRouter、Ollama、カスタムエンドポイントを1つのコマンドで切り替えられ、認証情報・設定モード・Claude フレームワークを Profile ごとにクリーンに分離して管理できます。
</p>

<p align="center">
<b>マルチ Provider · 設定分離 · Channel ブリッジ · ローカルエージェントブリッジ · 使用状況分析</b>
</p>

---

<p align="center"><b>Claude CLI 向けのモダンなマルチ Provider ランチャー。</b></p>

---

<p align="center">
Claudy は、統一されたコマンドインターフェースで複数の Provider に対して Claude を実行できるようにし、Provider の認証情報と Claude の設定オーバーレイを単一のホームディレクトリ配下で整理して管理します。
</p>

<p align="center">
    <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/rust-1.92%2B-orange.svg" alt="rust-lang" /></a>
    <a href="https://crates.io/crates/claudy"><img src="https://img.shields.io/crates/v/claudy.svg" alt="crates.io" /></a>
    <a href="https://crates.io/crates/claudy"><img src="https://img.shields.io/crates/d/claudy.svg" alt="Downloads" /></a>
    <a href="../../LICENSE"><img src="https://img.shields.io/badge/License-Apache%202.0-blue.svg" alt="License" /></a>
    <a href="https://buymeacoffee.com/epicsaga"><img src="https://img.shields.io/badge/Buy%20Me%20a%20Coffee-FFDD00?style=flat&logo=buy-me-a-coffee&logoColor=black" alt="Buy Me a Coffee" /></a>
</p>

---

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../assets/features-2048.png">
  <img alt="Claudy 機能紹介" src="../assets/features-2048.png" width="100%">
</picture>

## Claudy を使う理由

| | 機能 | なぜ重要か |
|--|------|----------|
| 🔄 | マルチ Provider 起動 | Anthropic、Z.AI、OpenRouter、Ollama、カスタムエンドポイントを1コマンドで切り替え |
| 📦 | Config Mode | `CLAUDE.md`、設定、スキル、エージェントをモードごとに分離 — 相互汚染なし |
| 🔗 | エージェント MCP ブリッジ | Claude CodeからGemini、Codex、Aiderなど20+のエージェントへタスク委譲 |
| 💬 | Channel ブリッジ | インタラクティブな権限プロンプト付きでTelegram、Slack、Discordボットを実行 |
| 📊 | 使用状況分析 | ローカルTauriダッシュボードでトークン使用量、コスト、ツールパターンを追跡 |
| 🔐 | 安全なプロセス制御 | SIGINT/SIGTERM転送、アトミック設定書き込み、0600資格情報ストレージ |
| 🛠️ | 運用 UX | インストール、更新、アンインストール、診断、接続テスト — 1つのバイナリで |

## 対応 Provider

> Claudy は Go ベースの Claude CLI マルチ Provider ランチャーである [Clother](https://github.com/jolehuit/clother) からインスピレーションを得ています。Z.AI Provider が最も十分にテストされています。他の Provider で問題が発生した場合は、[Issue を開いてください](https://github.com/epicsagas/claudy/issues)。

| Provider | ステータス | 備考 |
|---|---|---|
| ビルトイン (Anthropic) | ✅ テスト済み | デフォルト |
| Z.AI | ✅ テスト済み | |
| OpenRouter エイリアス | ⚠️ 実験的 | 十分にテストされていません — GitHub で Issue を報告してください |
| Ollama | ⚠️ 実験的 | 十分にテストされていません — GitHub で Issue を報告してください |
| カスタムエンドポイント | ⚠️ 実験的 | 十分にテストされていません — GitHub で Issue を報告してください |

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../assets/demo.gif">
  <img alt="demo" src="../assets/demo.gif" width="100%">
</picture>

## 要件

- macOS または Linux
- ソースからビルド/インストールするには Rust ツールチェーン（`cargo`）が必要
- Claude CLI がインストール済みで `PATH` から利用可能であること

## インストール

プラットフォームを選択してください:

**macOS / Linux (ワンライナー)**
```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/epicsagas/claudy/releases/latest/download/claudy-installer.sh | sh
```

**macOS (Homebrew)**
```bash
brew tap epicsagas/tap
brew install claudy
```

**Windows (PowerShell)**
```powershell
irm https://github.com/epicsagas/claudy/releases/latest/download/claudy-installer.ps1 | iex
```

**Cargo (全プラットフォーム)**
```bash
# ビルド済みバイナリ
cargo binstall claudy
# またはソースからビルド
cargo install claudy
```

**Gitからビルド**
```bash
git clone https://github.com/epicsagas/claudy.git
cd claudy
cargo install --path .
```

## セットアップ＆起動

```bash
# 初期化
claudy install
# APIキーの設定
echo 'ZAI_API_KEY=your-key-here' >> ~/.claudy/secrets.env
# 確認
claudy --version
# 起動
claudy zai
```

## クイックスタート

<img src="docs/assets/demo.gif" alt="Quick Start" width="100%" />

```bash
# 1) 利用可能な/解決済みの Profile を一覧表示
claudy ls

# 2) 対話形式で認証情報を設定
claudy setup

# 3) 1 つの Profile の詳細を確認
claudy show <profile>

# 4) Profile を使って Claude を起動
claudy <profile> [claude-args...]
```

## 主要概念

### Profile

Provider のメタデータ + 認証戦略（ビルトイン Provider、OpenRouter エイリアス、またはカスタム Provider）を解決する起動ターゲット。

### Mode

`~/.claudy/modes/<name>/` に配置された名前付きの Claude 設定ディレクトリ。

以下を実行すると:

```bash
claudy <profile> <mode> [args...]
```

Claudy が設定します:

```bash
CLAUDE_CONFIG_DIR=~/.claudy/modes/<mode>/
```

これにより Claude は Mode 固有の設定ファイルを読み込みます。

Mode は、独自の `CLAUDE.md`、スキル、エージェント、設定を備えた **専用の Claude フレームワーク・ツールキット** を実行する場合にも最適です — 例: [gstack](https://github.com/garrytan/gstack)、[superpowers](https://github.com/obra/superpowers)、[ecc](https://github.com/affaan-m/everything-claude-code) やカスタムハーネスなど。デフォルト設定を汚さず、各フレームワークを専用の Mode に分離できます:

```bash
# フレームワーク専用の Mode を作成
claudy mode create gstack

# フレームワークの設定を Mode ディレクトリにコピーまたはシンボリックリンク
cp -r /path/to/gstack/.claude/. ~/.claudy/modes/gstack/

# そのフレームワークをアクティブにして Claude を起動
claudy <profile> gstack
```

各 Mode ディレクトリは独立した `CLAUDE_CONFIG_DIR` であるため、フレームワーク同士、またはデフォルト設定と競合しません。

## コマンドリファレンス

### メインコマンド

- `claudy ls`（エイリアス: `list`）: 設定済み/解決済みの Profile を一覧表示。
- `claudy setup [provider]`（エイリアス: `config`）: 対話形式での Provider セットアップ。
- `claudy show <profile>`（エイリアス: `info`）: 解決済みの Provider 詳細を表示。
- `claudy ping [profile]`（エイリアス: `test`）: Provider の接続テスト。
- `claudy doctor`（エイリアス: `status`）: バージョン、パス、Profile 数を表示。
- `claudy sync`（エイリアス: `install`）: claudy バイナリのインストール/同期。
- `claudy update`: claudy の更新。
- `claudy uninstall`: インストール済みファイルを削除。
- `claudy mode <action> [name]`: Claude 設定 Mode の管理。
- `claudy channel <subcommand>`: Channel ブリッジの管理。
- `claudy mcp`: エージェントブリッジ用の MCP サーバーとして実行。
- `claudy analytics <subcommand>`: 使用状況分析ダッシュボード。

### Mode コマンド

```bash
claudy mode create <name>
claudy mode ls
claudy mode rm <name>
```

Mode 名のルール: `[a-z0-9][a-z0-9_-]*`（`mode` は予約語）。

### Channel コマンド（オプションのブリッジ）

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

`channel add` はボットトークン、許可ユーザー、Profile、Mode のマッピング設定をガイドします。

#### 対応プラットフォーム

| プラットフォーム | 受信方式 | インタラクティブボタン | 備考 |
|----------|-----------|-------------------|-------|
| Telegram | ロングポーリング + Webhook | インラインキーボード | 最も完全 |
| Slack | イベントサブスクリプション Webhook | Block Kit アクション | HMAC-SHA256 検証 |
| Discord | インタラクション Webhook | Action row コンポーネント | Ed25519 検証 |

#### Channel ボットコマンド

起動後、ボットはチャット内で以下のコマンドに応答します:

- `/help` — 利用可能なコマンドを表示
- `/cancel` — 現在のタスクをキャンセル
- `/model` — Claude モデルを変更（インタラクティブボタン）
- `/yolo` — 自動許可権限のトグル
- `/status` — セッションステータス、Profile、Mode、git ブランチ、トークン使用量を表示
- `/sessions` — 最近の Claude セッションを一覧表示（切替ボタン付き）
- `/projects` — プロジェクトを一覧表示（参照ボタン付き）
- `/new` — 新しいセッションを開始
- `/history` — 最近のセッション履歴を表示

その他のテキストを送信すると Claude と直接会話できます。

#### 権限プロンプト

Claude がツールの使用（コマンド実行、ファイル編集など）の承認を要求すると、ボットがチャットにインタラクティブな許可/拒否プロンプトを送信します。ボタンをタップすると応答が Claude に送信され、処理が自動的に継続されます。

#### シークレット

`~/.claudy/secrets.env` に認証情報を保存してください:

```env
TELEGRAM_BOT_TOKEN=...
SLACK_BOT_TOKEN=xoxb-...
SLACK_SIGNING_SECRET=...
DISCORD_BOT_TOKEN=...
DISCORD_APPLICATION_ID=...
DISCORD_PUBLIC_KEY=...
```

### エージェント MCP ブリッジ

`claudy mcp run` を実行すると、Claude Code が他のローカルにインストールされた AI コーディングエージェントにタスクを委譲できる stdio ベースの MCP サーバーが起動します。

```bash
claudy mcp run        # MCP サーバーを起動（Claude Code から呼び出される）
claudy mcp install    # claudy を Claude Code 設定に MCP サーバーとして登録
claudy mcp uninstall  # Claude Code の MCP 設定から claudy を削除
```

`claudy mcp install` を実行すると、自動的に `~/.claude/settings.json` に登録されます。`claudy mode create <name>` で Mode を作成すると、その Mode の設定ファイルにも登録されます。手動設定は不要です。

手動で登録する場合（またはプロジェクトレベルの `.claude/settings.json` に）:

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

Claude Code は、インストール済みのすべてのエージェントを公開する `ask_agent` ツールを確認できます。

#### 使用例

登録後、Claude Code は次のようにタスクを委譲できます:

```
> Ask gemini to review the error handling in src/api.rs
> Ask codex to write unit tests for the parser module
> Ask aider to refactor the database layer
```

Claude Code が適切なエージェントを選択し、プロンプトを渡して結果を返します。作業ディレクトリを指定することもできます:

```json
{ "agent": "gemini", "prompt": "Explain this module", "working_directory": "/path/to/project" }
```

#### MCP 登録の確認

```bash
# claudy が登録されているか確認
cat ~/.claude/settings.json | grep -A3 claudy

# MCP サーバーを手動でテスト
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | claudy mcp run
```

#### 対応エージェント（PATH から自動検出）

| エージェント | バイナリ | ヘッドレスコマンド |
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

#### カスタムエージェント

`~/.claudy/config.yaml` にエージェントを追加してください:

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

ビルトインエージェントと同じキーを使うとデフォルト値を上書きします。`args` 内の `{prompt}` は実際のタスクに置き換えられます。

### 分析コマンド

> **注意**: 分析機能はまだ開発中です。トークン数、コスト見積もり、その他の指標が完全に正確でない場合があります。今後のリリースで改善される予定です。

```bash
claudy analytics dashboard         # ローカル分析ダッシュボードを開く（Tauri 2）
claudy analytics ingest            # ~/.claude/projects/ からセッションデータを取り込む
claudy analytics ingest --full     # 全ファイルを再取り込み（チェックポイントを無視）
claudy analytics ingest --project my-project  # 特定のプロジェクトを取り込む
claudy analytics recommend         # CLI で使用状況の推奨事項を表示
claudy analytics export            # 分析データをエクスポート（JSON、デフォルト 30 日）
claudy analytics export --format csv --days 7  # 過去 7 日分を CSV でエクスポート
claudy analytics sync-pricing      # models.dev と Anthropic 価格ページからモデル価格を同期
claudy analytics recalculate       # 最新の価格データで全コストを再計算
claudy analytics insights          # LLM 分析用のコンパクト JSON サマリーを生成（デフォルト 7 日）
claudy analytics insights --days 14  # 過去 14 日間を分析
claudy analytics insights --from 2026-04-01 --to 2026-04-30  # 特定期間を分析
claudy analytics insights --project my-project  # プロジェクトでフィルタ
```

### Claude Code 内から: `/analytics-insights`

使用状況を分析する最も速い方法は、Claude Code 内から直接行うことです。`analytics-insights` スキルは自動的に利用可能です — 自然に質問するだけです:

```
> /analytics-insights
> /analytics-insights last 2 weeks
> analyze my usage patterns
> 使用パターンを分析して
```

Claude が `claudy analytics insights` を実行し、JSON を分析して、以下を含む構造化レポートを返します:

- **コストトレンド** — 日次/週次の支出とスパイク検出
- **モデル分布** — 使用しているモデルとセッションあたりのコスト
- **ツールパターン** — 最も使用頻度の高いツール、エラー率、効率性の観察
- **キャッシュパフォーマンス** — ヒット率と推定節約額
- **実用的な推奨事項** — 「シンプルなタスクは turbo にルーティング」のような具体的な提案と推定節約額

出力例（生データは [`docs/examples/analytics-insights-sample.json`](docs/examples/analytics-insights-sample.json) を参照）:

```
#### Summary
81 sessions, $481 total spend at an average of $68.7/day. Costs trending
sharply upward — last 3 weekdays averaged $97/day.

#### Recommendations
1. Route simple tasks to glm-5-turbo — est. savings: ~$90/month
2. Investigate $1.91/turn outlier session (6x average cost-per-turn)
3. Reduce harness overhead — TaskCreate/Update accounted for ~1,000 calls
```

手動コマンドもコンテキスト切替も不要。Claude に使用状況を聞くだけで、すぐに回答が得られます。

### 分析で追跡される項目

- **トークン**: モデルと日付でグループ化された過去 30 日間の入力・出力・キャッシュトークンの詳細な傾向。
- **ツール**: Claude が最もよく使うツールの分布分析（呼び出し回数、エラー率、平均実行時間を含む）。
- **コスト**: 実際のトークン価格に基づくリアルタイムの使用コスト見積もり（日次/週次/月次の予測とトレンド検出を含む）。
- **ヒント（推奨事項）**: 高コストセッションの検出、シンプルなタスクへの Haiku の提案、コンテキスト要約が効果的な長い会話の特定など、データ駆動の最適化アドバイス。
- **プロジェクト**: 難解なセッション UUID を人が読みやすいプロジェクトフォルダ名に自動マッピング。

データは `~/.claudy/analytics/` 配下のローカル SQLite データベースに保存されます。ダッシュボードは高性能なローカル Tauri 2 + Svelte アプリとして動作します。ダッシュボードの **[Sync]** ボタンを使用すると、Claude CLI の履歴からデータを即座に更新できます。

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../assets/analytics-dashboard.png">
  <img alt="分析ダッシュボード" src="../assets/analytics-dashboard.png" width="100%">
</picture>

## ファイルとディレクトリ構成

デフォルトでは、Claudy は以下の場所にデータを保存します:

```text
~/.claudy/
```

重要なファイル/ディレクトリ:

- `config.yaml`: Provider + Channel + エージェント設定。
- `secrets.env`: Provider/ボットの認証情報。
- `launchers.json`: ランチャー/シンボリックリンクのマニフェスト。
- `modes/`: Claude 設定 Mode。
- `session-patches/`: セッションパッチの保存場所。
- `channel/`: Channel ランタイム状態（`pid`、セッション、監査ログ）。
- `analytics/`: 分析用 SQLite データベースとチェックポイント。
- `cache/update.json`: 更新メタデータキャッシュ。

## 環境変数

- `CLAUDY_HOME`: Claudy ホームディレクトリの上書き（デフォルト: `~/.claudy`）。
- `CLAUDE_CONFIG_DIR`: Mode で起動する際に Claudy が自動設定。

## よくあるワークフロー

### Provider の設定と起動

```bash
claudy setup
claudy <profile>
```

### Provider と合わせて Mode を使用

```bash
claudy mode create work
claudy <profile> work --yolo
```

> `--yolo` は claudy における `--dangerously-skip-permissions` の省略形です。

### 専用の Claude フレームワークを Mode で実行

gstack、superpowers、ecc などのフレームワークは独自の `CLAUDE.md`、スキル、エージェントを提供します。それぞれを独立した Mode で実行:

```bash
# 初期設定: Mode を作成してフレームワークの設定を反映
claudy mode create gstack
cp -r /path/to/gstack/.claude/. ~/.claudy/modes/gstack/

# 日常使用: そのフレームワークをアクティブにして Claude を起動
claudy <profile> gstack
```

デフォルト設定を変えずにフレームワークを切り替え:

```bash
claudy <profile> gstack      # gstack フレームワーク有効
claudy <profile> superpowers # superpowers フレームワーク有効
claudy <profile>             # デフォルト設定のまま
```

### MCP を通じて他のエージェントにタスクを委譲

```bash
# 1) MCP が登録されていることを確認（初回 `claudy mcp` 実行時に自動登録）
claudy mcp

# 2) Claude Code でインストール済みのエージェントへの委譲をリクエスト:
#    "Ask gemini to analyze this error"
#    "Ask aider to refactor the auth module"
```

### インストール/設定状態の診断

```bash
claudy doctor
claudy ping
```

## トラブルシューティング

- **`profile not recognized`**: `claudy ls` を実行し、一覧表示された Profile ID を選択してください。
- **`not configured` Profile**: `claudy setup <provider>` を実行して認証情報を追加してください。
- **Channel ステータスが異常**: `claudy channel status` を実行した後、`claudy channel stop` と `claudy channel start` で再起動してください。
- **Channel ボットが応答しない**: `~/.claudy/channel/logs/server.log` でエラーを確認してください。`~/.claudy/secrets.env` のボットトークンと `allowed_users` にチャットユーザー ID が含まれているか確認してください。
- **権限プロンプトが表示されない**: Claude CLI が `--dangerously-skip-permissions` で実行されていないことを確認してください。プロンプトは Claude がツール使用の明示的な承認が必要な場合にのみトリガーされます。
- **インストール後にバイナリが見つからない**: Claudy の bin ディレクトリが `PATH` に含まれていることを確認し、シェルを再起動してください。
- **MCP にエージェントが表示されない**: エージェントのバイナリが `PATH` にあることを確認してください（`which gemini`）。インストール済みのエージェントのみ `tools/list` に表示されます。
- **エージェントのタイムアウト**: `config.yaml` の agents フィールドでタイムアウトを増やしてください（デフォルト: 120 秒）。
- **MCP が未登録**: `claudy mcp` を一度手動で実行するか、`~/.claude/settings.json` の `mcpServers.claudy` エントリを確認してください。
- **エージェントの出力が切り捨てられる**: エージェントの stdout は 10MB に制限されています。大きな出力の場合は、エージェントがファイルに書き出すようにリダイレクトしてください。
- **分析データがない**: `claudy analytics ingest` を実行して `~/.claude/projects/` からデータを取り込んでください。`--full` を使用するとすべてを再取り込みします。

## 開発

```bash
cargo build
cargo test
cargo fmt
cargo clippy -- -D warnings

# 分析バックエンドのテスト（ローカル DB を使用）
cargo run --example test_dashboard --features analytics-ui

# 分析ダッシュボードの起動（analytics-ui 機能が必要）
cargo run --features analytics-ui -- analytics dashboard
```

## コントリビューション

コントリビューションを歓迎します！開始方法:

1. リポジトリをフォークし、フィーチャーブランチを作成してください。
2. 適切な場合はテストとともに変更を加えてください。
3. 提出前に `cargo test && cargo clippy -- -D warnings` を実行してください。
4. https://github.com/epicsagas/claudy で Pull Request を開いてください。

バグ報告と機能リクエストは [GitHub Issues](https://github.com/epicsagas/claudy/issues) からお寄せください。

## 謝辞

このプロジェクトは、Go ベースの Claude CLI マルチ Provider ランチャーである [Clother](https://github.com/jolehuit/clother) からインスピレーションを受けています。Claudy はゼロから再設計された独立した Rust 実装であり、RAII ベースのセッションガード、シグナル転送、ランチャーシンボリックリンク、**フル機能の Channel ブリッジ**（Telegram/Slack/Discord）、クロスエージェント委譲のための**エージェント MCP ブリッジ**、Tauri 2 で構築された**高性能分析ダッシュボード**などの深いエコシステム統合が導入されました。これらの追加機能は、Claudy が単純なランチャーから Claude CLI ユーザー向けの総合的な運用ツールキットへと進化したことを反映しています。

## ライセンス

[Apache-2.0](../../LICENSE)
