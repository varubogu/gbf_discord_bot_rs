# 定期募集スケジュールのオフセット決定ロジック 設計書

## 概要

定期募集スケジュール作成時の募集開始日オフセット（`recruit_start_day_offset`）のデフォルト値決定ロジックを定義します。

## 背景

定期募集スケジュールでは、クエスト開始時刻と募集開始時刻の2つの時刻を指定します。募集開始日オフセットは、募集を開始する日がクエスト開始日の何日前かを示す値です。

- `recruit_start_day_offset = 0`: 当日募集（募集開始日とクエスト開始日が同じ）
- `recruit_start_day_offset = 1`: 前日募集（募集開始日がクエスト開始日の1日前）
- `recruit_start_day_offset = 2`: 二日前募集（募集開始日がクエスト開始日の2日前）

## オフセットのデフォルト値決定ロジック

### 基本原則

募集開始日時は常にクエスト開始日時より前である必要があります。
ユーザーがオフセットを指定しない場合、システムが自動的に適切なオフセット値を決定します。

### 決定ロジック

**入力なし（デフォルト）の場合:**

時刻部分のみを比較してオフセット値を決定します。

1. **募集開始時刻 < クエスト開始時刻の場合**: オフセット = 0
   - 理由: 当日の早い時刻に募集を開始し、遅い時刻にクエストを開始する自然な流れが成立
   - 例: 募集開始時刻 20:00、クエスト開始時刻 22:00
     - 同じ日の20:00に募集開始 → 同じ日の22:00にクエスト開始

2. **募集開始時刻 ≧ クエスト開始時刻の場合**: オフセット = 1
   - 理由: 当日だと募集開始がクエスト開始と同時または後になってしまうため、前日募集扱いとする
   - 例: 募集開始時刻 22:00、クエスト開始時刻 20:00
     - 前日の22:00に募集開始 → 翌日の20:00にクエスト開始
   - 例: 募集開始時刻 20:00、クエスト開始時刻 20:00
     - 前日の20:00に募集開始 → 翌日の20:00にクエスト開始

**入力あり（ユーザー指定）の場合:**

ユーザーが指定したオフセット値をそのまま使用します（0〜7の範囲で指定可能）。

### ユースケース例

#### ケース1: 当日募集（自動判定）
```
クエスト開始時刻: 22:00
募集開始時刻: 20:00
オフセット指定: なし
→ システム判定: オフセット = 0（当日募集）

実行例（火曜日の場合）:
- 火曜日 20:00 に募集開始
- 火曜日 22:00 にクエスト開始
```

#### ケース2: 前日募集（自動判定）
```
クエスト開始時刻: 20:00
募集開始時刻: 22:00
オフセット指定: なし
→ システム判定: オフセット = 1（前日募集）

実行例（火曜日の場合）:
- 月曜日 22:00 に募集開始
- 火曜日 20:00 にクエスト開始
```

#### ケース3: 同時刻（自動判定）
```
クエスト開始時刻: 21:00
募集開始時刻: 21:00
オフセット指定: なし
→ システム判定: オフセット = 1（前日募集）

実行例（火曜日の場合）:
- 月曜日 21:00 に募集開始
- 火曜日 21:00 にクエスト開始
```

#### ケース4: 手動指定
```
クエスト開始時刻: 22:00
募集開始時刻: 20:00
オフセット指定: 2
→ ユーザー指定: オフセット = 2（二日前募集）

実行例（火曜日の場合）:
- 日曜日 20:00 に募集開始
- 火曜日 22:00 にクエスト開始
```

## 実装仕様

### コマンドパラメータ

```rust
#[name_localized("ja", "募集開始日オフセット")]
#[description = "Recruitment start day offset (0=same day, 1=day before, default: auto)"]
#[description_localized(
    "ja",
    "募集開始日オフセット（0=当日、1=前日、2=二日前、省略時は自動判定）"
)]
#[min = 0]
#[max = 7]
recruit_start_day_offset: Option<i64>
```

### デフォルト値決定関数

```rust
/// 募集開始日オフセットのデフォルト値を決定
///
/// # 引数
/// - `recruit_start_time`: 募集開始時刻
/// - `quest_start_time`: クエスト開始時刻
///
/// # 戻り値
/// - 0: 当日募集（募集開始時刻 < クエスト開始時刻）
/// - 1: 前日募集（募集開始時刻 ≧ クエスト開始時刻）
fn determine_default_offset(
    recruit_start_time: NaiveTime,
    quest_start_time: NaiveTime,
) -> i32 {
    if recruit_start_time < quest_start_time {
        0 // 当日募集
    } else {
        1 // 前日募集
    }
}
```

### 使用箇所

1. **イベント層（`recruitment_schedule_create.rs`）**
   ```rust
   let default_offset = if recruit_start_day_offset.is_none() {
       // パース後の時刻を使って判定
       let recruit_time = time_parser.parse_time_string(&recruit_start_time)?;
       let quest_time = time_parser.parse_time_string(&quest_start_time)?;
       determine_default_offset(recruit_time, quest_time)
   } else {
       recruit_start_day_offset.unwrap() as i32
   };

   facade.create_recruitment_schedule(
       // ...
       default_offset,
       // ...
   ).await?;
   ```

2. **データベースモデル（`battle_recruitment_schedules.rs`）**

   ActiveModelBehaviorのデフォルト値は削除します（デフォルト値はコマンド層で決定されるため）:
   ```rust
   impl ActiveModelBehavior for ActiveModel {
       fn new() -> Self {
           let now = chrono::Utc::now();
           Self {
               // ...
               recruit_start_day_offset: sea_orm::NotSet, // デフォルト値を削除
               // ...
           }
       }
   }
   ```

## バリデーション

### 時刻の整合性チェック

オフセットが0の場合のみ、募集開始時刻がクエスト開始時刻より前であることを確認します。

```rust
// service/schedule/recruitment_schedule_service.rs
pub fn validate_schedule_input(
    &self,
    day_of_weeks: &[i32],
    quest_start_time: NaiveTime,
    recruit_start_day_offset: i32,
    recruit_start_time: Option<NaiveTime>,
) -> Result<()> {
    // ... 既存のバリデーション ...

    // 募集開始時刻とクエスト開始時刻の整合性チェック
    if let Some(recruit_time) = recruit_start_time {
        if recruit_start_day_offset == 0 && recruit_time >= quest_start_time {
            return Err(crate::types::AppError::Business {
                message:
                    "当日募集の場合、募集開始時刻はクエスト開始時刻より前である必要があります"
                        .to_string(),
            });
        }
    }

    Ok(())
}
```

## テストケース

### 単体テスト

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveTime;

    #[test]
    fn test_determine_default_offset_same_day() {
        // 募集開始時刻 < クエスト開始時刻 → オフセット0
        let recruit_time = NaiveTime::from_hms_opt(20, 0, 0).unwrap();
        let quest_time = NaiveTime::from_hms_opt(22, 0, 0).unwrap();
        assert_eq!(determine_default_offset(recruit_time, quest_time), 0);
    }

    #[test]
    fn test_determine_default_offset_previous_day() {
        // 募集開始時刻 > クエスト開始時刻 → オフセット1
        let recruit_time = NaiveTime::from_hms_opt(22, 0, 0).unwrap();
        let quest_time = NaiveTime::from_hms_opt(20, 0, 0).unwrap();
        assert_eq!(determine_default_offset(recruit_time, quest_time), 1);
    }

    #[test]
    fn test_determine_default_offset_equal_time() {
        // 募集開始時刻 == クエスト開始時刻 → オフセット1
        let recruit_time = NaiveTime::from_hms_opt(21, 0, 0).unwrap();
        let quest_time = NaiveTime::from_hms_opt(21, 0, 0).unwrap();
        assert_eq!(determine_default_offset(recruit_time, quest_time), 1);
    }

    #[test]
    fn test_validate_same_day_valid() {
        // 当日募集で募集開始 < クエスト開始 → OK
        let service = RecruitmentScheduleService::new();
        let result = service.validate_schedule_input(
            &[1, 3, 5],
            NaiveTime::from_hms_opt(22, 0, 0).unwrap(),
            0,
            Some(NaiveTime::from_hms_opt(20, 0, 0).unwrap()),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_same_day_invalid() {
        // 当日募集で募集開始 >= クエスト開始 → エラー
        let service = RecruitmentScheduleService::new();
        let result = service.validate_schedule_input(
            &[1, 3, 5],
            NaiveTime::from_hms_opt(20, 0, 0).unwrap(),
            0,
            Some(NaiveTime::from_hms_opt(22, 0, 0).unwrap()),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_previous_day_valid() {
        // 前日募集で募集開始 > クエスト開始 → OK
        let service = RecruitmentScheduleService::new();
        let result = service.validate_schedule_input(
            &[1, 3, 5],
            NaiveTime::from_hms_opt(20, 0, 0).unwrap(),
            1,
            Some(NaiveTime::from_hms_opt(22, 0, 0).unwrap()),
        );
        assert!(result.is_ok());
    }
}
```

### 統合テスト

実際のコマンド実行をシミュレートして、オフセット決定ロジックが正しく動作することを確認します。

## 移行対応

### 既存データへの影響

既存の定期募集スケジュールには影響しません。
新規作成時のみ、新しいロジックが適用されます。

### 後方互換性

コマンドのインターフェースは変更されません。
オフセットパラメータは引き続きオプショナルです。

## 関連ドキュメント

- [スケジュール処理システム設計書](./schedule_processing_system.md)
- [定期募集機能設計書](../../features/recruitment_notification_schedule.md)
