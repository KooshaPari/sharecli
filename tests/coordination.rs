use sharecli::coordination::{
    CommandLockStore, LockStatus, PriorityTaskQueue, QueuePriority, TaskStatus,
};

#[test]
fn command_lock_acquire_release_and_reacquire_by_same_pid() {
    let temp = tempfile::tempdir().unwrap();
    let store = CommandLockStore::new(temp.path().join("locks.json"));

    let first = store.acquire("cmd-hash", 1234, Some("out.log")).unwrap();
    assert_eq!(first.cmd_hash, "cmd-hash");
    assert_eq!(first.pid, 1234);
    assert_eq!(first.status, LockStatus::Locked);
    assert!(first.is_locked());

    let second = store.acquire("cmd-hash", 1234, Some("new.log")).unwrap();
    assert_eq!(second.output_path.as_deref(), Some("new.log"));
    assert!(second.is_locked());

    store.release("cmd-hash", 1234).unwrap();
    let released = store.get("cmd-hash").unwrap().unwrap();
    assert_eq!(released.pid, 0);
    assert_eq!(released.status, LockStatus::Unlocked);
    assert!(!released.is_locked());

    let reacquired = store.acquire("cmd-hash", 1234, None).unwrap();
    assert!(reacquired.is_locked());
}

#[test]
fn command_lock_rejects_other_pid_until_owner_releases() {
    let temp = tempfile::tempdir().unwrap();
    let store = CommandLockStore::new(temp.path().join("locks.json"));

    store.acquire("cmd-hash", 1111, None).unwrap();

    let err = store.acquire("cmd-hash", 2222, None).unwrap_err();
    assert!(err.to_string().contains("already locked"));

    let release_err = store.release("cmd-hash", 2222).unwrap_err();
    assert!(release_err.to_string().contains("cannot release"));
}

#[test]
fn priority_queue_lists_and_dequeues_by_priority() {
    let temp = tempfile::tempdir().unwrap();
    let queue = PriorityTaskQueue::new(temp.path().join("queue.json"));

    queue.enqueue("low", QueuePriority::Low).unwrap();
    queue.enqueue("normal", QueuePriority::Normal).unwrap();
    queue.enqueue("critical", QueuePriority::Critical).unwrap();
    queue.enqueue("high", QueuePriority::High).unwrap();

    let listed = queue.list_all().unwrap();
    let commands: Vec<_> = listed.iter().map(|item| item.command.as_str()).collect();
    assert_eq!(commands, ["critical", "high", "normal", "low"]);
    assert!(listed.iter().all(|item| item.status == TaskStatus::Pending));

    let next = queue.dequeue().unwrap().unwrap();
    assert_eq!(next.command, "critical");
    assert_eq!(next.status, TaskStatus::Dequeued);

    let remaining = queue.list_all().unwrap();
    let commands: Vec<_> = remaining.iter().map(|item| item.command.as_str()).collect();
    assert_eq!(commands, ["high", "normal", "low"]);
}
