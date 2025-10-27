#!/bin/bash

# 色設定
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
WHITE='\033[0;37m'
NC='\033[0m' # No Color

# ヘルプメッセージの表示
show_help() {
    echo -e "${GREEN}🛠️ Management Script for Development and Production${NC}"
    echo ""
    echo -e "${YELLOW}Usage:${NC}"
    echo -e "${WHITE}  ./mng.sh [environment] [command]${NC}"
    echo ""
    echo -e "${YELLOW}Environments:${NC}"
    echo -e "${WHITE}  dev  - Development environment${NC}"
    echo -e "${WHITE}  prod - Production environment${NC}"
    echo ""
    echo -e "${YELLOW}Commands:${NC}"
    echo -e "${WHITE}  up      - Start services (default)${NC}"
    echo -e "${WHITE}  down    - Stop services${NC}"
    echo -e "${WHITE}  nocache - Build without cache and start (prod only)${NC}"
    echo ""
    echo -e "${YELLOW}Examples:${NC}"
    echo -e "${WHITE}  ./mng.sh dev up${NC}"
    echo -e "${WHITE}  ./mng.sh prod down${NC}"
    echo -e "${WHITE}  ./mng.sh prod nocache${NC}"
}

# 入力補完機能
_mng_completion() {
    local cur prev opts
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    
    # 第1引数の場合
    if [[ ${COMP_CWORD} -eq 1 ]]; then
        opts="dev prod"
        if [[ ${cur} == -* ]] ; then
            COMPREPLY=( $(compgen -W "-h --help" -- ${cur}) )
            return 0
        fi
        COMPREPLY=( $(compgen -W "${opts}" -- ${cur}) )
        return 0
    fi
    
    # 第2引数の場合
    if [[ ${COMP_CWORD} -eq 2 ]]; then
        case "${COMP_WORDS[1]}" in
            dev)
                opts="up down"
                ;;
            prod)
                opts="up down nocache"
                ;;
            *)
                opts=""
                ;;
        esac
        COMPREPLY=( $(compgen -W "${opts}" -- ${cur}) )
        return 0
    fi
}

# 入力補完を登録
complete -F _mng_completion mng.sh
complete -F _mng_completion ./mng.sh

# 引数の処理
ENVIRONMENT=${1:-""}
COMMAND=${2:-"up"}

# ヘルプオプションの処理
if [[ "$1" == "-h" || "$1" == "--help" ]]; then
    show_help
    exit 0
fi

# 環境の検証
case $ENVIRONMENT in
    dev|prod)
        # 有効な環境
        ;;
    *)
        echo -e "${RED}❌ Invalid environment: $ENVIRONMENT${NC}"
        echo -e "${YELLOW}Available environments: dev, prod${NC}"
        show_help
        exit 1
        ;;
esac

# コマンドの検証
case $ENVIRONMENT in
    dev)
        case $COMMAND in
            up|down|help)
                # 有効なコマンド
                ;;
            *)
                echo -e "${RED}❌ Invalid command for dev: $COMMAND${NC}"
                echo -e "${YELLOW}Available commands for dev: up, down${NC}"
                show_help
                exit 1
                ;;
        esac
        ;;
    prod)
        case $COMMAND in
            up|down|nocache|help)
                # 有効なコマンド
                ;;
            *)
                echo -e "${RED}❌ Invalid command for prod: $COMMAND${NC}"
                echo -e "${YELLOW}Available commands for prod: up, down, nocache${NC}"
                show_help
                exit 1
                ;;
        esac
        ;;
esac

# dev up の処理
dev_up() {
    # .envファイルの存在確認
    if [ ! -f ".env" ]; then
        echo -e "${RED}❌ Warning: .env file not found!${NC}"
        echo -e "${YELLOW}Please create .env file based on .env.example${NC}"
        exit 1
    fi

    # 環境変数の読み込み
    set -a
    source .env
    set +a

    # コンテナが存在するか確認
    if [ ! "$(docker ps -q -f name=dev-db)" ]; then
        if [ "$(docker ps -aq -f status=exited -f name=dev-db)" ]; then
            # 停止中のコンテナがある場合は削除
            docker rm dev-db
        fi
        echo -e "${GREEN}🚀 Starting development database...${NC}"
        # Dockerイメージのビルド
        docker build -t dev-db-image -f Dockerfile.db .
        # コンテナの起動
        docker run -d \
            --name dev-db \
            -v pgdata:/var/lib/postgresql \
            -e POSTGRES_USER=${DB_USER} \
            -e POSTGRES_PASSWORD=${DB_PASSWORD} \
            -e POSTGRES_DB=${DB_NAME} \
            -p ${DB_PORT}:5432 \
            dev-db-image
    else
        echo -e "${YELLOW}✨ Database is already running${NC}"
    fi

    # データベースの接続確認
    echo -e "${CYAN}🔍 Checking database connection...${NC}"
    until docker exec dev-db pg_isready -U ${DB_USER}
    do
        echo -e "${YELLOW}🕐 Waiting for database to be ready...${NC}"
        sleep 2
    done

    echo -e "${GREEN}✅ Database is ready!${NC}"
    echo -e "${WHITE}Connection info:${NC}"
    echo -e "${WHITE}Host: ${DB_HOST}${NC}"
    echo -e "${WHITE}Port: ${DB_PORT}${NC}"
    echo -e "${WHITE}User: ${DB_USER}${NC}"
    echo -e "${WHITE}Database: ${DB_NAME}${NC}"
}

# dev down の処理
dev_down() {
    echo -e "${YELLOW}🛑 Stopping development database...${NC}"
    docker stop dev-db 2>/dev/null
    echo -e "${GREEN}✅ Development database stopped!${NC}"
}

# prod up の処理
prod_up() {
    echo -e "${GREEN}🚀 サービスを起動しています...${NC}"
    docker compose --env-file config/.env up -d
}

# prod down の処理
prod_down() {
    echo -e "${YELLOW}🛑 サービスを停止しています...${NC}"
    docker compose down
}

# prod nocache の処理
prod_nocache() {
    echo -e "${CYAN}🔄 キャッシュなしでビルドしています...${NC}"
    docker compose --env-file config/.env build --no-cache
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}🚀 サービスを起動しています...${NC}"
        docker compose --env-file config/.env up -d
    else
        echo -e "${RED}❌ ビルド中にエラーが発生しました${NC}"
        exit 1
    fi
}

# メイン処理
case $ENVIRONMENT in
    "dev")
        case $COMMAND in
            "up")
                dev_up
                ;;
            "down")
                dev_down
                ;;
            "help")
                show_help
                ;;
        esac
        ;;
    "prod")
        case $COMMAND in
            "up")
                prod_up
                ;;
            "down")
                prod_down
                ;;
            "nocache")
                prod_nocache
                ;;
            "help")
                show_help
                ;;
        esac
        ;;
esac

# 結果の確認
if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Process completed!${NC}"
else
    echo -e "${RED}❌ An error occurred during the process${NC}"
    exit 1
fi