param(
    [Parameter(Position = 0, Mandatory = $true)]
    [ValidateSet("dev", "staging", "prod", "help")]
    [string]$Environment,

    [Parameter(Position = 1)]
    [ArgumentCompleter({
        param($commandName, $parameterName, $wordToComplete, $commandAst, $fakeBoundParameters)

        $environment = $fakeBoundParameters['Environment']
        switch ($environment)
        {
            'dev' {
                @('up', 'down', 'help') | Where-Object { $_ -like "$wordToComplete*" }
            }
            'staging' {
                @('up', 'down', 'update_app', 'help') | Where-Object { $_ -like "$wordToComplete*" }
            }
            'prod' {
                @('up', 'down', 'update_app', 'help') | Where-Object { $_ -like "$wordToComplete*" }  # nocache is commented out
                # @('up', 'down', 'nocache', 'help') | Where-Object { $_ -like "$wordToComplete*" }
            }
            default {
                @('up', 'down', 'update_app', 'help') | Where-Object { $_ -like "$wordToComplete*" }  # nocache is commented out
                # @('up', 'down', 'nocache', 'help') | Where-Object { $_ -like "$wordToComplete*" }
            }
        }
    })]
    [string]$Command = "up"
)

function Show-Help
{
    Write-Host "🛠️ Management Script for Development, Staging and Production" -ForegroundColor Green
    Write-Host ""
    Write-Host "Usage:" -ForegroundColor Yellow
    Write-Host "  .\mng.ps1 [environment] [command]" -ForegroundColor White
    Write-Host ""
    Write-Host "Environments:" -ForegroundColor Yellow
    Write-Host "  dev     - Development environment" -ForegroundColor White
    Write-Host "  staging - Staging environment (builds locally)" -ForegroundColor White
    Write-Host "  prod    - Production environment (pulls from GHCR)" -ForegroundColor White
    Write-Host ""
    Write-Host "Commands:" -ForegroundColor Yellow
    Write-Host "  up         - Start services (default)" -ForegroundColor White
    Write-Host "  down       - Stop services" -ForegroundColor White
    Write-Host "  update_app - Update app container only (staging/prod only)" -ForegroundColor White
    # Write-Host "  nocache - Build without cache and start (prod only)" -ForegroundColor White
    Write-Host ""
    Write-Host "Examples:" -ForegroundColor Yellow
    Write-Host "  .\mng.ps1 dev up" -ForegroundColor White
    Write-Host "  .\mng.ps1 staging up" -ForegroundColor White
    Write-Host "  .\mng.ps1 staging update_app" -ForegroundColor White
    Write-Host "  .\mng.ps1 prod update_app" -ForegroundColor White
    Write-Host "  .\mng.ps1 prod down" -ForegroundColor White
    # Write-Host "  .\mng.ps1 prod nocache" -ForegroundColor White
}

function Start-DevDatabase
{
    # .env.dbファイルの存在確認
    if (-not (Test-Path ".env.db"))
    {
        Write-Host "❌ Warning: .env.db file not found!" -ForegroundColor Red
        Write-Host "Please create .env.db file based on .env.db.example" -ForegroundColor Yellow
        exit 1
    }

    # 環境変数ファイルの読み込み
    Get-Content ".env.db" | ForEach-Object {
        if ($_ -match "^([^=]+)=(.*)$")
        {
            [Environment]::SetEnvironmentVariable($matches[1], $matches[2], "Process")
        }
    }

    # 環境変数を取得（未設定時はデフォルト値を使用）
    $DBUSER = if ([string]::IsNullOrEmpty($env:POSTGRES_USER)) { "postgres" } else { $env:POSTGRES_USER }
    $DBPASSWORD = if ([string]::IsNullOrEmpty($env:POSTGRES_PASSWORD)) { "postgres" } else { $env:POSTGRES_PASSWORD }
    $DBDATABASE = if ([string]::IsNullOrEmpty($env:POSTGRES_DB)) { "postgres" } else { $env:POSTGRES_DB }
    $DBHOST = if ([string]::IsNullOrEmpty($env:DB_HOST)) { "localhost" } else { $env:DB_HOST }
    $DBPORT = if ([string]::IsNullOrEmpty($env:DB_PORT)) { "5432" } else { $env:DB_PORT }

    # コンテナが存在するか確認
    $runningContainer = docker ps -q -f name=dev-db
    if (-not $runningContainer)
    {
        $exitedContainer = docker ps -aq -f status=exited -f name=dev-db
        Write-Host "🚀 Starting development database..." -ForegroundColor Green
        # Dockerイメージのビルド
        docker build -t dev-db-image -f Dockerfile.db .
        # コンテナの起動
        docker run -d `
            --name dev-db `
            -v pgdata:/var/lib/postgresql/data `
            -e POSTGRES_USER="$DBUSER" `
            -e POSTGRES_PASSWORD="$DBPASSWORD" `
            -e POSTGRES_DB="$DBDATABASE" `
            -p "${DBPORT}:5432" `
            dev-db-image
    }
    else
    {
        Write-Host "✨ Database is already running" -ForegroundColor Yellow
    }

    # データベースの接続確認
    Write-Host "🔍 Checking database connection..." -ForegroundColor Cyan
    do
    {
        $isReady = docker exec dev-db pg_isready -U $DBUSER
        if ($LASTEXITCODE -ne 0)
        {
            Write-Host "🕐 Waiting for database to be ready..." -ForegroundColor Yellow
            Start-Sleep -Seconds 2
        }
    } while ($LASTEXITCODE -ne 0)

    Write-Host "✅ Database is ready!" -ForegroundColor Green
    Write-Host "Connection info:" -ForegroundColor White
    Write-Host "Host: $DBHOST" -ForegroundColor White
    Write-Host "Port: $DBPORT" -ForegroundColor White
    Write-Host "User: $DBUSER" -ForegroundColor White
    Write-Host "Database: $DBDATABASE" -ForegroundColor White
}

function Stop-DevDatabase
{
    Write-Host "🛑 Stopping development database..." -ForegroundColor Yellow
    docker stop dev-db 2> $null
    Write-Host "✅ Development database stopped!" -ForegroundColor Green
}

function Start-ProdServices
{
    Write-Host "🚀 サービスを起動しています..." -ForegroundColor Green
    docker compose up -d
}

function Stop-ProdServices
{
    Write-Host "🛑 サービスを停止しています..." -ForegroundColor Yellow
    docker compose down
}

function Start-StagingServices
{
    Write-Host "🚀 検証環境のサービスを起動しています（appコンテナを再ビルド）..." -ForegroundColor Green
    docker compose -f docker-compose.staging.yml build app
    docker compose -f docker-compose.staging.yml up -d
}

function Stop-StagingServices
{
    Write-Host "🛑 検証環境のサービスを停止しています..." -ForegroundColor Yellow
    docker compose -f docker-compose.staging.yml down
}

function Update-StagingApp
{
    Write-Host "🔄 検証環境のappコンテナを更新しています..." -ForegroundColor Cyan
    docker compose -f docker-compose.staging.yml build app
    docker compose -f docker-compose.staging.yml up -d app
    Write-Host "✅ appコンテナの更新が完了しました" -ForegroundColor Green
}

function Update-ProdApp
{
    Write-Host "🔄 本番環境のappコンテナを更新しています..." -ForegroundColor Cyan
    docker compose pull app
    docker compose up -d app
    Write-Host "✅ appコンテナの更新が完了しました" -ForegroundColor Green
}

# function Start-ProdServicesNoCache (commented out - no longer needed as production pulls pre-built images)
# {
#     Write-Host "🔄 キャッシュなしでビルドしています..." -ForegroundColor Cyan
#     docker compose build --no-cache
#     if ($LASTEXITCODE -eq 0)
#     {
#         Write-Host "🚀 サービスを起動しています..." -ForegroundColor Green
#         docker compose up -d
#     }
#     else
#     {
#         Write-Host "❌ ビルド中にエラーが発生しました" -ForegroundColor Red
#         exit 1
#     }
# }

# ヘルプの表示
if ($Environment -eq "help")
{
    Show-Help
    exit
}

# コマンドの検証と実行
switch ($Environment)
{
    "dev" {
        # devで利用可能なコマンドの検証
        if ($Command -notin @("up", "down", "help"))
        {
            Write-Host "❌ Invalid command for dev: $Command" -ForegroundColor Red
            Write-Host "Available commands for dev: up, down" -ForegroundColor Yellow
            Show-Help
            exit 1
        }

        switch ($Command)
        {
            "up" {
                Start-DevDatabase
            }
            "down" {
                Stop-DevDatabase
            }
            "help" {
                Show-Help
            }
        }
    }
    "staging" {
        # stagingで利用可能なコマンドの検証
        if ($Command -notin @("up", "down", "update_app", "help"))
        {
            Write-Host "❌ Invalid command for staging: $Command" -ForegroundColor Red
            Write-Host "Available commands for staging: up, down, update_app" -ForegroundColor Yellow
            Show-Help
            exit 1
        }

        switch ($Command)
        {
            "up" {
                Start-StagingServices
            }
            "down" {
                Stop-StagingServices
            }
            "update_app" {
                Update-StagingApp
            }
            "help" {
                Show-Help
            }
        }
    }
    "prod" {
        # prodで利用可能なコマンドの検証 (nocache is commented out)
        if ($Command -notin @("up", "down", "update_app", "help"))
        # if ($Command -notin @("up", "down", "nocache", "help"))
        {
            Write-Host "❌ Invalid command for prod: $Command" -ForegroundColor Red
            Write-Host "Available commands for prod: up, down, update_app" -ForegroundColor Yellow  # nocache is commented out
            # Write-Host "Available commands for prod: up, down, nocache" -ForegroundColor Yellow
            Show-Help
            exit 1
        }

        switch ($Command)
        {
            "up" {
                Start-ProdServices
            }
            "down" {
                Stop-ProdServices
            }
            "update_app" {
                Update-ProdApp
            }
            # "nocache" {
            #     Start-ProdServicesNoCache
            # }
            "help" {
                Show-Help
            }
        }
    }
}

Write-Host "✅ Process completed!" -ForegroundColor Green
