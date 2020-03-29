drop trigger if exists queued_jobs_associate_with_task_after_insert on queued_jobs;
drop function if exists queued_jobs_associate_with_task;

drop trigger if exists schedules_increment_update_count_after_update on schedules;
drop function if exists schedules_increment_update_count;

drop trigger if exists queued_jobs_increment_queued_count_after_delete on queued_jobs;
drop function if exists queued_jobs_increment_queued_count;

drop function if exists queued_jobs_bind_schedules_for_task;

drop table queued_jobs;
drop table tasks;
drop table schedules;
