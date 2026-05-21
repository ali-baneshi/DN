struct foo {
	int value;
};

int demo(struct foo *ptr)
{
	rcu_read_lock();
	if (ptr->value)
		return ptr->value;
	rcu_read_unlock();
	return 0;
}
