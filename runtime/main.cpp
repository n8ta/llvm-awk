extern "C" void print_value(char tag, double value);

union Value {
  double float_value;
  char* str_value;
};

int main() {
  double v = 1.2;
  union Value val;
  val.str_value= (char*) &v;
  print_value(0, val.float_value);
}
