# マルチ募集v2 - Discord Components v2 対応計画

## 概要

Discord の Components v2 が serenity で正式サポートされたら、マルチ募集v2コマンドのUIを改善する。
現在、属性選択ボタンは画面下部に集中しているが、Components v2 を使用することで、各属性の横にボタンを配置できるようになり、視認性が向上する。

## 現在の実装（Components v1）

### ボタン配置
```
行1: [🔥 火] [💧 水] [🌍 土]
行2: [💨 風] [☀️ 光] [🌙 闇]
行3: [🌈 全属性可能] [❌ 全て取り消し]
```

### 参加者一覧表示（Embed内）
```
🔥 火: @user1 @user2
💧 水: @user3
🌍 土: なし
💨 風: @user4 @user5 @user6
☀️ 光: なし
🌙 闇: なし
🌈 全属性可能: @user7
```

### 問題点
- ボタンとテキストが分離しているため、どのボタンがどの属性か直感的に分かりにくい
- 画面下部にボタンが集中し、スクロールが必要になる場合がある

## Components v2 対応後の実装

### 新しいUI（Section使用）

6属性はそれぞれ Section を使って、ボタンとテキストを同じ行に配置：

```
[参加] 🔥 火: @user1 @user2
[参加] 💧 水: @user3
[参加] 🌍 土: なし
[参加] 💨 風: @user4 @user5 @user6
[参加] ☀️ 光: なし
[参加] 🌙 闇: なし
[🌈 全属性可能] [❌ 全て取り消し]
```

**注**: ボタンとテキストの順序（[ボタン] テキスト or テキスト [ボタン]）は、実装時に Discord の仕様を確認して決定

### 実装方法

#### 1. 依存関係の更新

**Cargo.toml**
```toml
# serenity を next ブランチ（または正式リリース後のバージョン）に更新
serenity = { git = "https://github.com/serenity-rs/serenity", branch = "next" }
# または正式リリース後
serenity = "0.13"  # バージョンは正式リリース時に確認
```

#### 2. 実装ファイル

**対象ファイル**: `src/services/recruitment/new.rs`

**修正対象関数**: `create_recruitment_buttons`

**新規実装内容**:

```rust
use poise::serenity_prelude::{
    CreateSection,
    CreateSectionComponent,
    CreateSectionAccessory,
    CreateTextDisplay,
    CreateButton,
    ButtonStyle
};

/// 募集用セクションを作成する（Components v2版）
pub fn create_recruitment_sections(battle_style_name: &str) -> Vec<CreateSection> {
    use crate::types::{ALL_ELEMENTS_EMOJI, ELEMENT_EMOJIS, ELEMENT_NAMES};

    if battle_style_name == "6属性" {
        let mut sections = Vec::new();

        // 6属性それぞれにセクションを作成
        for (i, (emoji, name)) in ELEMENT_EMOJIS.iter().zip(ELEMENT_NAMES.iter()).enumerate() {
            let text = CreateTextDisplay::new(format!("{} {}: なし", emoji, name));
            let button = CreateButton::new(format!("recruit_join_{}", i + 1))
                .label("参加")
                .style(ButtonStyle::Primary);

            let section = CreateSection::new(
                vec![CreateSectionComponent::TextDisplay(text)],
                CreateSectionAccessory::Button(button)
            );
            sections.push(section);
        }

        // 全属性可能と全て取り消しは従来通りのActionRowで配置
        // （または別のセクションとして実装）

        sections
    } else {
        // シンプル参加の場合は従来のボタンのまま
        // または同様に Section で実装
        vec![]
    }
}
```

#### 3. メッセージ送信処理の更新

**対象ファイル**: `src/services/recruitment/new.rs`

**修正対象関数**: `send_recruitment_message_with_buttons`

- `CreateReply::components()` に Section を渡すように変更
- Components v2 のメッセージ構造に合わせて調整

#### 4. 参加者一覧の更新処理

**対象ファイル**: `src/events/handlers/component_interaction.rs` または関連ファイル

- ボタンクリック時に対応する Section のテキストを更新
- `EditMessage` または `EditInteractionResponse` で Section を更新する処理を実装

### 制限事項

#### Discord API の制限
- **Section は最大3つのコンポーネント + 1つのアクセサリー**をサポート
- **全コンポーネントで4000文字制限を共有**（参加者が多い場合は注意）
- **メッセージあたりのコンポーネント数に制限あり**（最大40個）

#### 実装上の注意
- 6属性 + 全属性 + 全て取り消し = 8個のコンポーネント（制限内）
- 参加者が多い場合、文字数制限に注意
- `next` ブランチを使う場合、API変更のリスクあり

## 実装スケジュール

### Phase 1: 正式リリース待機
- [ ] serenity の Components v2 正式リリースを待つ
- [ ] リリースノートで API の安定性を確認
- [ ] poise の対応状況も確認

### Phase 2: 開発環境での検証
- [ ] 開発環境で serenity を更新
- [ ] 小規模なテストコードで Components v2 の動作確認
- [ ] ボタンとテキストの配置順序を検証

### Phase 3: 実装
- [ ] `create_recruitment_sections` 関数を実装
- [ ] メッセージ送信処理を更新
- [ ] 参加者一覧更新処理を実装
- [ ] 既存の `create_recruitment_buttons` との互換性を考慮

### Phase 4: テストと検証
- [ ] 単体テスト作成
- [ ] 統合テスト実施
- [ ] 本番環境でのベータテスト

### Phase 5: デプロイ
- [ ] 本番環境にデプロイ
- [ ] 既存の募集メッセージとの互換性確認
- [ ] ユーザーフィードバックの収集

## 参考情報

### 公式ドキュメント
- [serenity PR #3123 - ComponentsV2 support](https://github.com/serenity-rs/serenity/pull/3123)
- [CreateSection documentation](https://serenity-rs.github.io/serenity/next/serenity/builder/struct.CreateSection.html)
- [CreateTextDisplay documentation](https://serenity-rs.github.io/serenity/next/serenity/builder/struct.CreateTextDisplay.html)
- [CreateSectionAccessory documentation](https://serenity-rs.github.io/serenity/next/serenity/builder/enum.CreateSectionAccessory.html)

### Discord Components v2 について
- [Components V2 | Umbra's Rantings (discord.py)](https://about.abstractumbra.dev/discord.py/2025/08/17/components-v2.html)
- [Components V2 - D++ (C++)](https://dpp.dev/components_v2.html)
- [Discord Library Support Status](https://libs.advaith.io/)

### Components v2 の特徴
- Section コンポーネントでテキストとボタンを同じ行に配置可能
- よりモダンで柔軟なUIレイアウト
- Container、MediaGallery、Separator など新しいコンポーネントタイプ

## 備考

- 正式リリース時期は未定（2024年11月時点で serenity 0.12.4 が最新、Components v2 は `next` ブランチのみ）
- poise の対応も必要になる可能性あり
- 実装前に必ず最新のドキュメントとサンプルコードを確認すること
