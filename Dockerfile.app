# ================================
# ビルドステージ
# ================================
FROM rust:1.93-bookworm AS builder

WORKDIR /build

# 依存関係キャッシング用にCargo関連ファイルとmigrationをコピー
COPY Cargo.toml Cargo.lock ./
COPY migration ./migration

# ビルドスクリプトをコピー（スキーマユーティリティ生成に必要）
COPY build.rs ./

# ロケールファイルをコピー（ビルド時最適化のため必要）
COPY locales ./locales

# ダミーソースで依存関係をビルド（キャッシュ層として機能）
# schema_lintは開発用ツールのため本番イメージでは除外
RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release --bin gbf_discord_bot_rs || true

# ビルド成果物のうち、次のビルドで再利用されないものを削除
# target/release/build/ は build.rs の成果物が入っているが、
# srcディレクトリが変更されると無効になるため削除
RUN rm -rf src target/release/build/

# 実際のソースコードをコピーしてリビルド
# build/ ディレクトリが存在しないため、build.rsが確実に実行される
COPY src ./src
RUN cargo build --release --bin gbf_discord_bot_rs

# ================================
# ランタイムステージ
# ================================
FROM debian:bookworm-slim

# CA証明書をインストール（HTTPS通信に必要）
# 注: PostgreSQL接続はsqlxの純粋Rust実装を使用するためlibpqは不要
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# セキュリティのため非rootユーザーで実行
RUN useradd -m -u 1001 botuser

WORKDIR /app

# ビルド済みバイナリをコピー
COPY --from=builder /build/target/release/gbf_discord_bot_rs .

# ロケールファイルをコピー
COPY locales ./locales

# 権限設定
RUN chown -R botuser:botuser /app
USER botuser

# Bot起動
CMD ["./gbf_discord_bot_rs"]
