# 募集機能拡張計画

## 1. 概要

`docs/develop/features/quest_recruitment.md` に記載された現行仕様とは別に、募集内容変更フローなど将来的に追加を検討している機能計画を整理します。

## 2. 募集内容変更フロー（計画）

```mermaid
sequenceDiagram
    participant U as User
    participant C as Command
    participant F as Facade
    participant S as Service
    participant R as Repository
    participant D as Discord

    U->>C: /recruit_change message recruit quest event_date
    C->>F: change_recruitment_information()
    F->>F: authorize(actor, message_owner, has_gbf_bot_control)
    F->>R: get_recruitment_by_message()
    R-->>F: recruitment
    F->>S: regenerate_recruit_content()
    S->>D: edit_message()
    D-->>S: ok
    S->>D: send_update_notification()
    D-->>S: notification_id
    S->>R: update_owner_if_needed()
    S->>R: update_recruitment()
    R-->>S: ok
    S-->>F: ok
    F-->>C: success
    C-->>U: 更新完了通知
```

## 3. 運用メモ

- フロー実装時は Discord 権限確認とメッセージ編集 API の制約を再確認する。  
- 実装が進んだ場合は機能設計書を最新化し、本計画の該当部分を完了扱いに更新する。
