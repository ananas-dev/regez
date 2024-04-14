#include "test.h"
#include <stdio.h>

int main(int argc, char** argv) {
    if (matches(argv[1])) {
        printf("string matches\n");
    } else {
        printf("string doesn't match\n");
    }

    return 0;
}