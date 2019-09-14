drop trigger if exists queued_jobs_set_state_after_delete on queued_jobs;
drop function if exists queued_jobs_set_pending_or_finished_state;

drop trigger if exists queued_jobs_set_state_after_insert on queued_jobs;
drop function if exists queued_jobs_set_processing_state;
drop function if exists queued_jobs_bind_schedules_for_task;

drop table queued_jobs;
drop table tasks;
drop table schedules;
