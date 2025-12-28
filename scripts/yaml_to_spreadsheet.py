#!/usr/bin/env python3
"""
YAMLメッセージファイルをGoogleスプレッドシート形式（TSV）に変換するスクリプト

使用方法:
    python scripts/yaml_to_spreadsheet.py

出力:
    locales/messages.tsv - Googleスプレッドシートにコピペ可能なタブ区切り形式
"""

import re
from pathlib import Path


def parse_yaml_manually(content: str) -> dict:
    """
    YAMLファイルを手動でパースする（標準ライブラリのみ使用）

    Args:
        content: YAMLファイルの内容

    Returns:
        パース結果の辞書
    """
    lines = content.split('\n')
    result = {}
    current_path = []

    for line in lines:
        # コメントと空行をスキップ
        if not line.strip() or line.strip().startswith('#'):
            continue

        # インデントレベルを計算（2スペース単位）
        indent = len(line) - len(line.lstrip(' '))
        level = indent // 2

        # キー:値のペアを抽出
        if ':' in line:
            key_value = line.strip()
            if key_value.startswith('_'):
                # _version などはスキップ
                continue

            parts = key_value.split(':', 1)
            key = parts[0].strip()
            value = parts[1].strip() if len(parts) > 1 else ''

            # 値がクォートで囲まれている場合は除去
            if value:
                # ダブルクォートまたはシングルクォートを除去
                if (value.startswith('"') and value.endswith('"')) or \
                   (value.startswith("'") and value.endswith("'")):
                    value = value[1:-1]

            # 現在のパスを更新
            current_path = current_path[:level] + [key]

            # 値がある場合は結果に追加
            if value:
                path_key = '.'.join(current_path)
                # エスケープシーケンスを実際の文字に変換
                value = value.replace('\\n', '\n').replace('\\t', '\t')
                result[path_key] = value

    return result


def flatten_messages(parsed_data: dict) -> list[tuple[str, str, str]]:
    """
    パースされたYAMLデータを平坦化してスプレッドシート形式に変換

    Args:
        parsed_data: パース結果の辞書

    Returns:
        (id, message_jp, message_en)のタプルのリスト
    """
    messages = {}

    for key, value in parsed_data.items():
        # キーが .ja または .en で終わる場合
        if key.endswith('.ja'):
            base_key = key[:-3]  # .ja を除去
            if base_key not in messages:
                messages[base_key] = {'jp': '', 'en': ''}
            messages[base_key]['jp'] = value
        elif key.endswith('.en'):
            base_key = key[:-3]  # .en を除去
            if base_key not in messages:
                messages[base_key] = {'jp': '', 'en': ''}
            messages[base_key]['en'] = value

    # タプルのリストに変換（IDでソート）
    result = []
    for msg_id in sorted(messages.keys()):
        result.append((
            msg_id,
            messages[msg_id]['jp'],
            messages[msg_id]['en']
        ))

    return result


def escape_tsv_field(field: str) -> str:
    """
    TSVフィールドをエスケープする

    改行やタブを含む場合はダブルクォーテーションで囲む

    Args:
        field: エスケープするフィールド

    Returns:
        エスケープされたフィールド
    """
    # 改行、タブ、ダブルクォーテーションを含む場合はクォートで囲む
    if '\n' in field or '\t' in field or '"' in field:
        # ダブルクォーテーションを2つ重ねてエスケープ
        escaped = field.replace('"', '""')
        return f'"{escaped}"'
    return field


def write_tsv(output_path: Path, data: list[tuple[str, str, str]]):
    """
    TSVファイルを書き込む

    Args:
        output_path: 出力ファイルパス
        data: (id, message_jp, message_en)のタプルのリスト
    """
    with open(output_path, 'w', encoding='utf-8') as f:
        # ヘッダー行
        f.write('id\tmessage_jp\tmessage_en\n')

        # データ行
        for msg_id, msg_jp, msg_en in data:
            # 各フィールドをエスケープ
            msg_id_escaped = escape_tsv_field(msg_id)
            msg_jp_escaped = escape_tsv_field(msg_jp)
            msg_en_escaped = escape_tsv_field(msg_en)

            f.write(f'{msg_id_escaped}\t{msg_jp_escaped}\t{msg_en_escaped}\n')


def main():
    """メイン処理"""
    # パスの設定
    project_root = Path(__file__).parent.parent
    input_file = project_root / 'locales' / 'messages.yml'
    output_file = project_root / 'locales' / 'messages.tsv'

    print(f'入力ファイル: {input_file}')
    print(f'出力ファイル: {output_file}')

    # YAMLファイルを読み込み
    if not input_file.exists():
        print(f'エラー: {input_file} が見つかりません')
        return

    with open(input_file, 'r', encoding='utf-8') as f:
        content = f.read()

    # YAMLをパース
    print('YAMLファイルをパース中...')
    parsed_data = parse_yaml_manually(content)
    print(f'パース完了: {len(parsed_data)}個のエントリを検出')

    # スプレッドシート形式に変換
    print('スプレッドシート形式に変換中...')
    spreadsheet_data = flatten_messages(parsed_data)
    print(f'変換完了: {len(spreadsheet_data)}個のメッセージ')

    # TSVファイルに書き込み
    print('TSVファイルに書き込み中...')
    write_tsv(output_file, spreadsheet_data)
    print(f'✅ 完了: {output_file}')
    print()
    print('Googleスプレッドシートへのインポート方法:')
    print('1. Googleスプレッドシートを開く')
    print('2. ファイル > インポート > アップロード')
    print(f'3. {output_file.name} をアップロード')
    print('4. 「タブ」を区切り文字として選択')
    print('または、TSVファイルの内容をコピーしてスプレッドシートに直接貼り付けることもできます')


if __name__ == '__main__':
    main()
