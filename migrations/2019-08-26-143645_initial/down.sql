drop trigger if exists queued_tasks_set_state_after_delete on queued_tasks;
drop function if exists queued_tasks_set_pending_or_finished_state;

drop trigger if exists queued_tasks_set_state_after_insert on queued_tasks;
drop function if exists queued_tasks_set_processing_state;

drop table queued_tasks;
drop table tasks;
drop table schedules;
