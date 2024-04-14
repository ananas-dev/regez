#include <stdio.h>

// Define the maximum size of the stack.
#define MAX_SIZE 100

// Stack structure for DFA state storage
typedef struct {
    int items[MAX_SIZE];
    int top;
} Stack;

// Function to initialize the stack
void stack_init(Stack *s) {
    s->top = -1;
}

// Function to check if the stack is full
int isFull(Stack *s) {
    return s->top == MAX_SIZE - 1;
}

// Function to check if the stack is empty
int isEmpty(Stack *s) {
    return s->top == -1;
}

// Function to add an element to the stack
void push(Stack *s, int state) {
    if (isFull(s)) {
        printf("Error: Stack is full.\n");
    } else {
        s->items[++s->top] = state;
    }
}

// Function to remove an element from the stack
int pop(Stack *s) {
    if (isEmpty(s)) {
        printf("Error: Stack is empty.\n");
        return -1; // Return -1 or any invalid state to indicate empty stack
    } else {
        int state = s->items[s->top--];
        return state;
    }
}

// Function to clear the stack
void clear(Stack *s) {
    s->top = -1;
}