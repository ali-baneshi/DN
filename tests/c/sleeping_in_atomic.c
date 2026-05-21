typedef int spinlock_t;
void spin_lock(spinlock_t *lock);
void msleep(int timeout);

void demo(spinlock_t *lock)
{
	spin_lock(lock);
	msleep(10);
}
