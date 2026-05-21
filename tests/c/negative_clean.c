struct foo {
	int value;
};

typedef int spinlock_t;
struct mutex;

void spin_lock(spinlock_t *lock);
void spin_unlock(spinlock_t *lock);
void mutex_lock(struct mutex *lock);
void mutex_unlock(struct mutex *lock);
void rcu_read_lock(void);
void rcu_read_unlock(void);
struct foo *rcu_dereference(struct foo *ptr);

int demo(struct foo *ptr, spinlock_t *spin, struct mutex *lock)
{
	int ret = 0;
	mutex_lock(lock);
	if (!ptr)
		goto out;
	rcu_read_lock();
	ptr = rcu_dereference(ptr);
	ret = ({ struct foo *safe = rcu_dereference(ptr); safe->value; });
	rcu_read_unlock();
	spin_lock(spin);
	spin_unlock(spin);
out:
	mutex_unlock(lock);
	return ret;
}
