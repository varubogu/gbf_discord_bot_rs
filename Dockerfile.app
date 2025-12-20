# ================================
# ビルドステージ
# ================================
FROM rust:latest AS builder

WORKDIR /build

# 依存関係キャッシング用にCargo関連ファイルとmigrationをコピー
COPY Cargo.toml Cargo.lock ./
COPY migration ./migration

# ダミーソースで依存関係をビルド（キャッシュ層として機能）
# 次回以降、ソースコード変更時は依存関係のビルドをスキップできる
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src target/release/gbf_discord_bot_rs*

# 実際のソースコードをコピーしてリビルド
# 依存関係は既にキャッシュされているため、アプリケーションコードのみビルドされる
COPY src ./src
RUN cargo build --release

# ================================
# ランタイムステージ
# ================================
FROM debian:bookworm-slim

# PostgreSQLクライアントライブラリとCA証明書をインストール
# - ca-certificates: HTTPS通信（Google Sheets API等）に必要
# - libpq5: PostgreSQL接続に必要
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    libpq5 && \
    rm -rf /var/lib/apt/lists/*

# セキュリティのため非rootユーザーで実行
RUN useradd -m -u 1001 botuser

WORKDIR /app

# ビルド済みバイナリをコピー
COPY --from=builder /build/target/release/gbf_discord_bot_rs .

# 権限設定
RUN chown -R botuser:botuser /app
USER botuser

# Bot起動
CMD ["./gbf_discord_bot_rs"]
