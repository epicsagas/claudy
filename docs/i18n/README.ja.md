<h1 align="center">claudy</h1>

<p align="center"><b>一つのコマンド。すべてのプロバイダー。Claude CLIの完全な制御。</b></p>

<p align="center">
環境変数と設定ファイルの管理に悩むのはもう終わり。<br/>
Claudyを使えば、Anthropic、Z.AI、OpenRouter、Ollama、カスタムエンドポイントを1つのコマンドで切り替えられます — 認証情報、設定モード、Claudeフレームワークをプロファイルごとにきれいに分離します。
</p>

<p align="center">
<b>マルチプロバイダー · 設定分離 · チャネルブリッジ · ローカルエージェントブリッジ · 使用量分析</b>
</p>

---

<p align="center">
  <a href="../../README.md">🇺🇸 English</a> •
  <a href="README.ko.md">🇰🇷 한국어</a> •
  <a href="README.zh-Hans.md">🇨🇳 中文</a> •
  <a href="README.de.md">🇩🇪 Deutsch</a> •
  <a href="README.fr.md">🇫🇷 Français</a> •
  <a href="README.es.md">🇪🇸 Español</a> •
  <a href="README.hi.md">🇮🇳 हिन्दी</a> •
  <a href="README.pt-BR.md">🇧🇷 Português</a> •
  <a href="README.id.md">🇮🇩 Bahasa</a> •
  <a href="README.ar.md">🇸🇦 العربية</a>
</p>

<p align="center">
    <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/rust-1.92%2B-orange.svg" alt="rust-lang" /></a>
    <a href="https://crates.io/crates/claudy"><img src="https://img.shields.io/crates/v/claudy.svg" alt="crates.io" /></a>
    <a href="https://crates.io/crates/claudy"><img src="https://img.shields.io/crates/d/claudy.svg" alt="Downloads" /></a>
    <a href="../../LICENSE"><img src="https://img.shields.io/badge/License-Apache%202.0-blue.svg" alt="License" /></a>
    <a href="https://buymeacoffee.com/epicsaga"><img src="https://img.shields.io/badge/Buy%20Me%20a%20Coffee-FFDD00?style=flat&logo=buy-me-a-coffee&logoColor=black" alt="Buy Me a Coffee" /></a>
    <a href="https://github.com/epicsagas/claudy/actions/workflows/ci.yml"><img src="https://github.com/epicsagas/claudy/actions/workflows/ci.yml/badge.svg" alt="CI" /></a>
</p>

---

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../assets/features-2048.png">
  <img alt="Claudy 機能紹介" src="../assets/features-2048.png" width="100%">
</picture>

## なぜClaudyが必要か

| | 機能 | なぜ重要か |
|--|------|-----------|
| 🔄 | マルチプロバイダー起動 | Anthropic、Z.AI、OpenRouter、Ollama、カスタムエンドポイントを1つのコマンドで切り替え |
| 📦 | 設定モード | `CLAUDE.md`、設定、スキル、エージェントをモードごとに分離 — 相互汚染なし |
| 🔗 | エージェントMCPブリッジ | Claude Codeからagy、Codex、Aiderなど20以上のエージェントにタスクを委任 |
| 💬 | チャネルブリッジ | Telegram、Slack、Discordボットをインタラクティブな権限プロンプト付きで実行 |
| 📊 | 使用量分析 | トークン使用量、コスト、ツールパターンをローカルTauriダッシュボードで追跡 |
| 🔐 | 安全なプロセス制御 | SIGINT/SIGTERM転送、アトミック設定書き込み、0600認証情報ストレージ |
| 🔀 | クロスプロバイダーセッション継続性 | Z.AI/GLMで作成したセッションをAnthropic APIで引き継いで作業できるよう自動修復 |
| 🛠️ | 運用UX | インストール、更新、アンインストール、診断、ping — すべて1つのバイナリから |

## サポートプロバイダー

> ClaudyはClaude CLI用のGoベースのマルチプロバイダーランチャーである[Clother](https://github.com/jolehuit/clother)にインスパイアされました。Z.AIが最も徹底的にテストされたプロバイダーです。他のプロバイダーに問題がある場合は、[イシューを開いてください](https://github.com/epicsagas/claudy/issues)。

| プロバイダー | ステータス | 備考 |
|---|---|---|
| ビルトイン (Anthropic) | ✅ テスト済み | デフォルト |
| Z.AI | ✅ テスト済み | |
| OpenRouterエイリアス | ⚠️ 実験的 | 完全にテストされていません — GitHubにイシューを報告してください |
| Ollama | ⚠️ 実験的 | 完全にテストされていません — GitHubにイシューを報告してください |
| カスタムエンドポイント | ⚠️ 実験的 | 完全にテストされていません — GitHubにイシューを報告してください |

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../assets/demo.gif">
  <img alt="デモ" src="../assets/demo.gif" width="100%">
</picture>

## クイックスタート

**1. インストール**

macOS / Linux:

```bash
brew install epicsagas/tap/claudy
```

Homebrewがない場合、インストーラースクリプトを使用:

```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/epicsagas/claudy/releases/latest/download/claudy-installer.sh | sh
```

Windows:

```powershell
irm https://github.com/epicsagas/claudy/releases/latest/download/claudy-installer.ps1 | iex
```

Rustツールチェーン経由:

```bash
cargo binstall claudy   # プレビルドバイナリ (高速)
cargo install claudy    # ソースからビルド
```

**2. 設定**

```bash
claudy install                        # ディレクトリ、設定、シークレットを初期化
echo 'ANTHROPIC_API_KEY=your-key' >> ~/.claudy/secrets.env
```

**3. 起動**

```bash
claudy                                # デフォルトプロバイダー
claudy zai                            # Z.AIプロバイダー
claudy openrouter sonnet              # OpenRouterエイリアス
```

**4. 更新**

```bash
brew upgrade claudy          # Homebrew
claudy update                # 内蔵アップデーター
# またはインストーラースクリプトを再実行 / cargo binstall claudy@latest
claudy --version
```

<details>
<summary>プロバイダー認証情報</summary>

| 変数 | プロバイダー |
|---|---|
| `ANTHROPIC_API_KEY` | Anthropic (ネイティブ) |
| `ZAI_API_KEY` | Z.AI |
| `ZAI_CN_API_KEY` | Z.AI 中国 |
| `MINIMAX_API_KEY` | MiniMax |
| `MINIMAX_CN_API_KEY` | MiniMax 中国 |
| `KIMI_API_KEY` | Kimi K2 |
| `MOONSHOT_API_KEY` | Moonshot AI |
| `ARK_API_KEY` | VolcEngine |
| `DEEPSEEK_API_KEY` | DeepSeek |
| `MIMO_API_KEY` | Xiaomi MiMo |
| `ALIBABA_API_KEY` | Alibaba Coding Plan |
| `OPENROUTER_API_KEY` | OpenRouter (全エイリアス) |

カスタムプロバイダーは `custom_providers` エントリで定義された `api_key_env` 変数を使用します。

</details>

<details>
<summary>config.yamlスキーマ</summary>

すべての設定は `~/.claudy/config.yaml` にあります。必要なセクションのみ追加してください — 省略された項目にはデフォルト値が使用されます。

> 全リファレンス: [docs/config.md](../config.md)

```yaml
# プロバイダーオーバーライド — プロバイダーごとにデフォルトモデルとモデルティアをオーバーライド
provider_overrides:
  zai:
    model: "glm-5.1"
    model_tiers:
      haiku: "glm-4.7"                # → ANTHROPIC_DEFAULT_HAIKU_MODEL
      sonnet: "glm-5.1"               # → ANTHROPIC_DEFAULT_SONNET_MODEL
      opus: "glm-5"                   # → ANTHROPIC_DEFAULT_OPUS_MODEL

# OpenRouterエイリアス — claudy <エイリアス> で起動
openrouter_aliases:
  kimi: "moonshotai/kimi-k2.5"
  sonnet: "anthropic/claude-sonnet-4"

# カスタムAnthropic互換プロバイダー — claudy <slug> で起動
custom_providers:
  my-llm:
    name: "my-llm"
    display_name: "My Custom LLM"
    base_url: "https://my-llm.com/api/anthropic"
    api_key_env: "MY_LLM_API_KEY"
    default_model: "my-model-v1"

# 圧縮ポリシー
compaction:
  auto_compact: true                   # デフォルト: true
  threshold: 0.8                       # 0.0–1.0、デフォルト: 0.8

# モデルごとのコンテキストウィンドウオーバーライド
model_settings:
  deepseek-chat:
    max_context_tokens: 64000

# チャネルブリッジ — `claudy channel add`の非対話型代替
channel:
  enabled_platforms: ["telegram"]
  listen_addr: "127.0.0.1:3456"
  default_profile: "zai"
  platform_profiles:
    telegram: "zai"
  platform_allowed_users:
    telegram: ["user_id_1"]
  max_concurrent_sessions: 0           # 0 = 無制限
  stream_timeout_secs: 1800

# エージェントオーバーライド
agents:
  aider:
    binary: "aider"
    args: ["--message", "{prompt}"]
    timeout: 300
```

</details>

---

## コアコンセプト

### プロファイル

ビルトインプロバイダー、OpenRouterエイリアス、またはカスタムプロバイダーのメタデータ + 認証戦略を解決する起動ターゲットです。

### モード

`~/.claudy/modes/<name>/`にある名前付きClaude設定ディレクトリです。

次のように実行すると:

```bash
claudy <profile> <mode> [args...]
```

Claudyは次を設定します:

```bash
CLAUDE_CONFIG_DIR=~/.claudy/modes/<mode>/
```

これにより、Claudeはモード固有の設定ファイルを読み取ります。

モードは、独自の `CLAUDE.md`、スキル、エージェント、設定を提供する**専用Claudeフレームワークやツールキット**にも適しています — 例: [gstack](https://github.com/garrytan/gstack)、[superpowers](https://github.com/obra/superpowers)、[ecc](https://github.com/affaan-m/everything-claude-code)、独自の [epic-harness](https://github.com/epicsagas/epic-harness)(自己進化する Claude Code プラグイン)、その他のカスタムハーネス。デフォルト設定を汚染する代わりに、各フレームワークを独自のモードで分離してください:

```bash
# フレームワーク用の専用モードを作成
claudy mode create gstack

# フレームワークの設定をモードディレクトリにコピーまたはシンボリックリンク
cp -r /path/to/gstack/.claude/. ~/.claudy/modes/gstack/

# そのフレームワークがアクティブな状態でClaudeを起動
claudy <profile> gstack
```

各モードディレクトリは自己完結型の `CLAUDE_CONFIG_DIR` であり、フレームワーク間で競合することはありません。

> **[epic-harness](https://github.com/epicsagas/epic-harness)との相性が抜群です。** Claudyは運用層を担い(プロバイダー切替、設定の分離、チャネル/エージェントブリッジ)、epic-harness(3つのコマンド、26の自動トリガースキル、失敗パターンから自己進化)がエージェントの知性を加えます。同じ `epicsagas` ファミリーで、モード単位で関心を明確に分離します。

<details>
<summary>コマンドリファレンス</summary>

## コマンドリファレンス

### メインコマンド

- `claudy ls` (エイリアス: `list`): 設定済み/解決済みプロファイルを一覧表示。
- `claudy setup [provider]` (エイリアス: `config`): インタラクティブなプロバイダー設定。
- `claudy show <profile>` (エイリアス: `info`): 解決済みプロバイダーの詳細を表示。
- `claudy ping [profile]` (エイリアス: `test`): プロバイダー接続をテスト。
- `claudy doctor` (エイリアス: `status`): バージョン、パス、プロファイル数を表示。
- `claudy sync` (エイリアス: `install`): claudyバイナリをインストール/同期。
- `claudy update`: claudyを更新。
- `claudy uninstall`: インストール済みファイルを削除。
- `claudy mode <action> [name]`: Claude設定モードを管理。
- `claudy channel <subcommand>`: チャネルブリッジを管理。
- `claudy mcp`: エージェントブリッジ用MCPサーバーとして実行。
- `claudy analytics <subcommand>`: 使用量分析ダッシュボード。
- `claudy session sanitize`: 非Anthropicプロバイダーによる無効なthinkingブロックを持つセッションを修復します。

### モードコマンド

```bash
claudy mode create <name>
claudy mode ls
claudy mode remove <name>
```

モード名ルール: `[a-z0-9][a-z0-9_-]*` (`mode`は予約語)。

### チャネルコマンド (オプションブリッジ)

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

`channel add`はボットトークン、許可ユーザー、プロファイル、モードマッピングを案内します。

#### サポートプラットフォーム

| プラットフォーム | インジェスション | インタラクティブボタン | 備考 |
|----------|-----------|-------------------|-------|
| Telegram | ロングポーリング + Webhook | インラインキーボード | 最も完成度が高い |
| Slack | イベントサブスクリプションWebhook | Block Kitアクション | HMAC-SHA256検証 |
| Discord | インタラクションWebhook | アクションローコンポーネント | Ed25519検証 |

#### チャネルボットコマンド

実行中、ボットはチャットで次のコマンドに応答します:

- `/help` — 利用可能なコマンドを表示
- `/cancel` — 現在のタスクをキャンセル
- `/model` — Claudeモデルを変更 (インタラクティブボタン)
- `/yolo` — 自動許可パーミッションをトグル
- `/status` — セッションステータス、プロファイル、モード、gitブランチ、トークン使用量を表示
- `/sessions` — 最近のClaudeセッションを一覧表示 (切替ボタン付き)
- `/projects` — プロジェクトを一覧表示 (閲覧ボタン付き)
- `/new` — 新しいセッションを開始
- `/history` — 最近のセッション履歴を表示

その他のテキストを送信すると、Claudeと直接対話できます。

#### 権限プロンプト

Claudeがツール使用の承認を要求したとき (コマンドの実行、ファイルの編集など)、
ボットはチャットにインタラクティブな許可/拒否プロンプトを送信します。ボタンをタップすると、
応答がClaudeに返送され、処理が自動的に継続されます。

#### シークレット

チャネル認証情報を `~/.claudy/secrets.env` に保存します (完全な形式は[プロバイダー認証情報](#プロバイダー認証情報-secretsenv)を参照):

```env
TELEGRAM_BOT_TOKEN=...
SLACK_BOT_TOKEN=xoxb-...
SLACK_SIGNING_SECRET=...
DISCORD_BOT_TOKEN=...
DISCORD_APPLICATION_ID=...
DISCORD_PUBLIC_KEY=...
```

</details>

## エージェントMCPブリッジ

`claudy mcp`を実行すると、Claude Codeがローカルにインストールされた他のAIコーディングエージェントにタスクを委任できるstdioベースのMCPサーバーが起動します。

```bash
claudy mcp run        # MCPサーバーを起動 (Claude Codeが呼び出し)
claudy mcp install    # Claude Code設定にMCPサーバーとして登録
claudy mcp uninstall  # Claude Code MCP設定から削除
```

`claudy mcp install`は自動的に `~/.claude/settings.json` に登録します。`claudy mode create <name>`でモードを作成すると、そのモードの設定ファイルにも登録されます。手動設定は不要です。

手動で登録する場合 (またはプロジェクトレベルの `.claude/settings.json` に):

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

Claude Codeはインストールされたすべてのエージェントを公開する `ask_agent` ツールを認識します。

### 使用例

登録されると、Claude Codeは次のようにタスクを委任できます:

```
> Ask agy to review the error handling in src/api.rs
> Ask codex to write unit tests for the parser module
> Ask aider to refactor the database layer
```

Claude Codeが適切なエージェントを選択し、プロンプトを渡し、結果を返します。作業ディレクトリを指定することもできます:

```json
{ "agent": "agy", "prompt": "Explain this module", "working_directory": "/path/to/project" }
```

### MCP登録の確認

```bash
# claudyが登録されているか確認
cat ~/.claude/settings.json | grep -A3 claudy

# MCPサーバーを手動テスト
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | claudy mcp run
```

### サポートエージェント (PATHから自動検出)

| エージェント | バイナリ | ヘッドレスコマンド |
|-------|--------|-----------------|
| Antigravity | `agy` | `agy -p "..."` |
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

### カスタムエージェント

`~/.claudy/config.yaml`の `agents` キーにエージェントを追加します (完全なスキーマは[設定](#configyamlスキーマ)を参照):

```yaml
agents:
  my-agent:
    binary: "my-agent"
    args: ["--prompt", "{prompt}", "--no-interactive"]
    description: "My custom agent"
    timeout: 180
```

ビルトインエージェントと同じキーを使用するとデフォルトをオーバーライドします。`args`の `{prompt}` は実際のタスクに置換されます。

## 使用量分析

> **注**: 分析機能はまだ開発中です。トークン数、コスト推定、その他のメトリクスが完全に正確でない場合があります。今後のリリースで改善予定です。

```bash
claudy analytics dashboard         # ローカル分析ダッシュボードを開く (Tauri 2)
claudy analytics ingest            # ~/.claude/projects/からセッションデータを取り込み
claudy analytics ingest --full     # すべてのファイルを再取り込み (チェックポイントを無視)
claudy analytics ingest --project my-project  # 特定のプロジェクトを取り込み
claudy analytics recommend         # CLIで使用量の推奨事項を表示
claudy analytics export            # 分析データをエクスポート (JSON、デフォルト30日)
claudy analytics export --format csv --days 7  # 過去7日間をCSVでエクスポート
claudy analytics sync-pricing      # models.devとAnthropic価格ページからモデル価格を同期
claudy analytics recalculate       # 最新の価格データですべてのコストを再計算
claudy analytics insights          # コンパクトなJSONインサイトサマリーを生成 (デフォルト: 7日)
claudy analytics insights --days 14  # 過去14日間を分析
claudy analytics insights --from 2026-04-01 --to 2026-04-30  # 特定の日付範囲
claudy analytics insights --project my-project  # プロジェクトでフィルター
```

### Claude Code内で: `/analytics-insights`

使用量を分析する最速の方法は、Claude Code内で直接行うことです。`analytics-insights`スキルは自動的に利用可能です — 自然に聞いてください:

```
> /analytics-insights
> /analytics-insights last 2 weeks
> analyze my usage patterns
> 사용 패턴 분석해줘
```

Claudeが `claudy analytics insights` を実行し、JSONを分析して構造化されたレポートを返します:

- **コストトレンド** — 日/週の支出とスパイク検出
- **モデル分布** — 使用しているモデルとセッションあたりのコスト
- **ツールパターン** — 最も使用されるツール、エラー率、効率性の観察
- **キャッシュパフォーマンス** — ヒット率と推定節約額
- **実行可能な推奨事項** — "簡単なタスクをturboにルーティング"などの具体的な提案と推定節約額

出力例 (生データは [`docs/examples/analytics-insights-sample.json`](../examples/analytics-insights-sample.json) を参照):

```
#### サマリー
81セッション、合計$481、1日平均$68.7。コストが急激に
上昇傾向 — 直近3営業日の平均は1日$97。

#### 推奨事項
1. 単純なタスクをglm-5-turboにルーティング — 推定節約額: ~$90/月
2. $1.91/ターンの外れ値セッションを調査 (平均コスト/ターンの6倍)
3. harnessオーバーヘッドを削減 — TaskCreate/Updateが約1,000回呼び出し
```

手動コマンドなし、コンテキスト切り替えなし。Claudeに使用量について聞くだけで、すぐに回答が得られます。

### 分析が追跡する項目

- **トークン**: 過去30日間の入力、出力、キャッシュトークンの詳細なトレンド (モデルと日付別グループ化)
- **ツール**: Claudeが最も頻繁に使用するツールの分布分析 (呼び出し回数、エラー率、平均実行時間を含む)
- **コスト**: 実際のトークン価格に基づくリアルタイム使用コスト推定 (日/週/月予測とトレンド検出を含む)
- **ヒント (推奨事項)**: 高コストセッションの検出、簡単なタスクへのHaikuの提案、コンテキスト要約が有益な長い会話の識別など、データ駆動の最適化アドバイス
- **プロジェクト**: 暗号化されたセッションUUIDを読みやすいプロジェクトフォルダ名に自動マッピング

データは `~/.claudy/analytics/` のローカルSQLiteデータベースに保存されます。ダッシュボードは高性能なローカルTauri 2 + Svelteアプリとして実行されます。ダッシュボードの **[Sync]** ボタンを使用して、Claude CLI履歴から即座にデータを更新してください。

### 分析ダッシュボード
```bash
claudy analytics dashboard
```
<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../assets/analytics-dashboard.png">
  <img alt="分析ダッシュボード" src="../assets/analytics-dashboard.png" width="100%">
</picture>

---

## クロスプロバイダーセッション継続性

Z.AI / GLMなど非AnthropicプロバイダーによるセッションのJSONLファイルには、空のsignatureを持つthinkingブロックが記録されます。そのセッションをAnthropic APIで再開すると、次のエラーが発生します:

```
API Error: 400 Invalid `signature` in `thinking` block
```

Claudyは2つの方法でこの問題を処理します:

**自動 (チャネルブリッジ):** チャネルサーバーがセッションを再開する際、空のsignatureを持つthinkingブロックを自動的に通常のテキストブロックに変換します。追加の操作は不要です。

**手動 (CLI):** `claude --resume`で直接再開する前に、`claudy session sanitize`でセッションを修復します:

```bash
# インタラクティブ — 問題のあるセッションリストから選択
claudy session sanitize

# プロジェクト名でフィルタリング
claudy session sanitize --project book-forge

# すべての問題セッションを一括処理
claudy session sanitize --all --yes
```

**変換の仕組み:** 空のsignatureを持つthinkingブロックが通常のテキストブロックに書き換えられます。推論内容はテキストとして保持され、セッションファイルはアトミックに更新されます。有効なAnthropic signatureを持つブロックは変更されません。

**制限事項:** セッション継続性は会話履歴の互換性に依存します。セッション中にプロバイダーを切り替えると、sanitization後も微妙なコンテキストの変化が生じる可能性があります。

---

## ファイルとディレクトリ構成

デフォルトでは、Claudyは次の場所にデータを保存します:

```text
~/.claudy/
```

重要なファイル/ディレクトリ:

- `config.yaml`: プロバイダー + チャネル + エージェント設定。
- `secrets.env`: プロバイダー/ボット認証情報。
- `launchers.json`: ランチャー/シンボリックマニフェスト。
- `modes/`: Claude設定モード。
- `session-patches/`: セッションパッチストレージ。
- `channel/`: チャネルランタイム状態 (`pid`、セッション、監査ログ)。
- `analytics/`: 分析SQLiteデータベースとチェックポイント。
- `cache/update.json`: 更新メタデータキャッシュ。

## 環境変数

- `CLAUDY_HOME`: Claudyホームディレクトリをオーバーライド (デフォルト: `~/.claudy`)。
- `CLAUDE_CONFIG_DIR`: モードで起動時にClaudyが自動設定。

## 一般的なワークフロー

### プロバイダーの設定と起動

```bash
claudy setup
claudy <profile>
```

### モードとプロバイダーの組み合わせ

```bash
claudy mode create work
claudy <profile> work --yolo
```

> `--yolo`はclaudyの `--dangerously-skip-permissions` のショートハンドです。

### 専用Claudeフレームワークを独自モードで実行

gstack、superpowers、ecc、または独自の [epic-harness](https://github.com/epicsagas/epic-harness) などのフレームワークは独自の `CLAUDE.md`、スキル、エージェントを提供します。分離して保持してください:

```bash
# 一度だけ: モードを作成しフレームワーク設定をシード
claudy mode create gstack
cp -r /path/to/gstack/.claude/. ~/.claudy/modes/gstack/

# 日常使用: フレームワークがアクティブな状態でClaudeを起動
claudy <profile> gstack
```

デフォルト設定を変更せずにフレームワーク間を切り替え:

```bash
claudy <profile> gstack      # gstackフレームワークがアクティブ
claudy <profile> superpowers # superpowersフレームワークがアクティブ
claudy <profile>             # デフォルト設定、変更なし
```

### MCP経由で他のエージェントにタスクを委任

```bash
# 1) MCPが登録されていることを確認 (初回 `claudy mcp` 時に自動登録)
claudy mcp

# 2) Claude Codeでインストール済みエージェントにタスクを委任:
#    "Ask agy to analyze this error"
#    "Ask aider to refactor the auth module"
```

### インストール/設定状態の診断

```bash
claudy doctor
claudy ping
```

## トラブルシューティング

- **`profile not recognized`**: `claudy ls`を実行し、リストされたプロファイルIDを選択してください。
- **`not configured`プロファイル**: `claudy setup <provider>`を実行して認証情報を追加してください。
- **チャネルステータス異常**: `claudy channel status`を実行し、`claudy channel stop`と `claudy channel start`で再起動してください。
- **チャネルボットが応答しない**: `~/.claudy/channel/logs/server.log`でエラーを確認してください。`~/.claudy/secrets.env`のボットトークンと、`allowed_users`にチャットユーザーIDが含まれているか確認してください。
- **権限プロンプトが表示されない**: Claude CLIが `--dangerously-skip-permissions` で実行されていないことを確認してください。プロンプトはClaudeがツール使用に明示的な承認を必要とする場合にのみトリガーされます。
- **インストール後にバイナリが見つからない**: [確認](#verify)セクションのPATHに関する注意を参照してください。
- **エージェントがMCPに表示されない**: エージェントバイナリが `PATH` にあることを確認してください (`which agy`)。インストール済みエージェントのみが `tools/list` に表示されます。
- **エージェントタイムアウト**: `config.yaml`のagentsフィールドでタイムアウトを増やしてください (デフォルト: 120秒)。
- **MCPが登録されない**: `claudy mcp`を手動で1回実行するか、`~/.claude/settings.json`で `mcpServers.claudy` エントリを確認してください。
- **エージェント出力が切り詰められる**: エージェントstdoutは10MBに制限されています。大きな出力の場合は、エージェントがファイルに書き込むようにリダイレクトしてください。
- **分析データが欠落**: `claudy analytics ingest`を実行して `~/.claude/projects/` からデータを取り込んでください。`--full`を使用してすべてを再取り込みしてください。
- **セッション再開時に `400 Invalid signature in thinking block`**: Z.AIなど非AnthropicプロバイダーによるセッションはAnthropicのAPIで無効なthinkingブロックが含まれています。`claudy session sanitize`を実行して変換した後、通常通り再開してください。

## 開発

```bash
cargo build
cargo test
cargo fmt
cargo clippy -- -D warnings

# 分析バックエンドのテスト (ローカルDBを使用)
cargo run --example test_dashboard --features analytics-ui

# 分析ダッシュボードの起動 (analytics-ui機能が必要)
cargo run --features analytics-ui -- analytics dashboard
```

## 貢献

貢献を歓迎します！始め方:

1. リポジトリをフォークし、機能ブランチを作成してください。
2. 適切なテストとともに変更を加えてください。
3. 提出前に `cargo test && cargo clippy -- -D warnings` を実行してください。
4. https://github.com/epicsagas/claudy でプルリクエストを開いてください。

バグレポートと機能リクエストは [GitHub Issues](https://github.com/epicsagas/claudy/issues) からお願いします。

## 謝辞

このプロジェクトはClaude CLI用のGoベースマルチプロバイダーランチャーである[Clother](https://github.com/jolehuit/clother)にインスパイアされました。Claudyは独立したRust実装として、RAIIベースのセッションガード、シグナル転送、ランチャーシンボリックリンクを含めゼロから再設計され、**フル機能チャネルブリッジ**(Telegram/Slack/Discord)、クロスエージェント委任のための**エージェントMCPブリッジ**、Tauri 2で構築された**高性能分析ダッシュボード**を含む深いエコシステム統合を提供します。これらの追加機能は、Claudyが単なるランチャーからClaude CLIユーザーのための包括的な運用ツールキットへと進化したことを反映しています。

## ライセンス

[Apache-2.0](../../LICENSE)
