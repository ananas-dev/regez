#include "stack.h"

int accepting[] = {1,1,0};

int matches(char *input) {
	int state;
	char c;
	int cursor = 0;
	Stack stack = {};
	stack_init(&stack);
start:
	push(&stack, -1);
	goto s2;
s0:
	state = 0;
	if ((c = input[cursor++]) == '\0') goto end;
	clear(&stack);
	push(&stack, 0);
	goto end;
s1:
	state = 1;
	if ((c = input[cursor++]) == '\0') goto end;
	clear(&stack);
	push(&stack, 1);
	if ((c == 'c')) goto s0;
	goto end;
s2:
	state = 2;
	if ((c = input[cursor++]) == '\0') goto end;
	push(&stack, 2);
	if ((c == 'a') || (c == 'b')) goto s1;
	goto end;
end:
	return accepting[state];
}

