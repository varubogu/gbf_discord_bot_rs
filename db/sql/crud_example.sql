-- 未来のスケジュール一覧
select * from worker.scheduled_tasks where schedule_datetime >= now() order by schedule_datetime;
select * from worker.notifications where schedule_datetime >= now() order by schedule_datetime;

-- 通知処理（task_type = 1）の未来のスケジュール一覧
select * from worker.scheduled_tasks st
    left join worker.notifications n on st.id = n.task_id
where task_type = 1
order by st.schedule_datetime