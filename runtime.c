#include <stdio.h>

void print_value(char tag, long long int value) {
//    printf("print_value tag:%d value:%lld\n", (int) tag, value);
    if (tag == 0) {
        printf("%lld\n", value);
    } else if (tag == 1) {
        printf("%f\n", (double) value);
    }
}

long int to_bool_i64(char tag, long long int value) {
//    printf("to_bool_i64 tag:%d value:%lld\n", (int) tag, value);
    if (tag == 0) {
        return value == 0 ? 0 : 1;
    } else if (tag == 1) {
        return (0.0 == (double) value) ? 0 : 1;
    }
    return 1;
}
void print_mismatch() {
    printf("integer float mismatch\n");
}
double get_float() {
    return 2.2;
}