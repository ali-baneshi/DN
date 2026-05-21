struct foo {
	int value;
};

int demo(struct foo *foo)
{
	int value = foo->value;
	if (!foo)
		return -1;
	return value;
}
