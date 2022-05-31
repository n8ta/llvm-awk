#include <stdio.h>
#include <iostream>


union Value {
  double float_value;
  long long int int_value;
};
static char FS = ' ';
static char RS = '\n';

extern "C" void print_value(char tag, long long int value) {
  // Is it UB? Yes. Is it easy? Yes;
  union Value val;
  val.int_value = value;
#ifdef DEBUG
  printf("to_bool_i64 tag:%d value:%lld value:%lf\n", (int) tag, val.int_value, val.float_value);
#endif
  if (tag == 0) {
    printf("%lld\n", val.int_value);
  } else if (tag == 1) {
    printf("%g\n", (double) val.float_value);
  }
}


extern "C" long int to_bool_i64(char tag, long long int value) {
  union Value val;
  val.int_value = value;
#ifdef DEBUG
  printf("to_bool_i64 tag:%d value:%lld value:%lf\n", (int) tag, val.int_value, val.float_value);
#endif
  if (tag == 0) {
    return val.int_value == 0 ? 0 : 1;
  } else if (tag == 1) {
    return val.float_value == 0.0 ? 0 : 1;
  }
  return 1;
}

extern "C" void print_mismatch() {
  printf("integer float mismatch\n");
}

extern "C" double get_float() {
  return 2.2;
}