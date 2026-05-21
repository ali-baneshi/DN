#include <stdio.h>

// This line is way too long and exceeds the 80 character limit which is required by kernel coding style
int main() 
{
	int x = 10;
 	int y = 20;  
	
	if(x > 5) {
		printf("x is greater than 5\n");
	}
	
	for(int i = 0; i < 10; i++)
	{
		printf("%d\n", i);
	}
	
	return 0;
}
