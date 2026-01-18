-- 未来のスケジュール一覧
select * from worker.scheduled_tasks where schedule_datetime >= now() order by schedule_datetime;
select * from worker.notifications where schedule_datetime >= now() order by schedule_datetime;

-- 通知処理（task_type = 1）の未来のスケジュール一覧
select * from worker.scheduled_tasks st
    left join worker.notifications n on st.id = n.task_id
where task_type = 1
order by st.schedule_datetime;

-- ギルドチャンネル一覧
select * from guild_master.guild_channels gc
    left join guild_master.guilds g on gc.guild_id = g.guild_id
order by gc.guild_id, gc.channel_type;

-- ギルドチャンネル1件更新
update guild_master.guild_channels set
  channel_id = :channel_id
where guild_id = :guild_id and channel_type = :channel_type;

-- ギルドチャンネル1件削除
delete from guild_master.guild_channels
where guild_id = :guild_id and channel_type = :channel_type;