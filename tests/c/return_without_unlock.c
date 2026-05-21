struct mutex;
void mutex_lock(struct mutex *lock);

int demo(struct mutex *lock)
{
	mutex_lock(lock);
	return 1;
}
