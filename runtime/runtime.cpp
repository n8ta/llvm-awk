#include <stdio.h>
#include <string>
#include <iostream>
#include <vector>
#include <fstream>

#ifdef DEBUG
#define PRINT(...) do{ fprintf( stderr, __VA_ARGS__ ); } while( false )
#else
#define PRINT(...) do{ } while ( false )
#endif

union Value {
  double float_value;
  int64_t int_value;
  char* str_value;
};

static char FS = ' ';
static char RS = '\n';

static std::string full_line;
static std::vector<std::string> fields;
static std::vector<std::string> files;
static std::ifstream current_file;

// Returns a malloc'ed C style null terminated string to be passed across
// ffi to llvm program. llvm program is responsible for calling free_string
// when it is done with it.
char* owned_string(std::string data) {
  size_t allocation_size = data.length()+1;
  char* pointer_to_existing = (char*) data.c_str();
  char* new_string = (char*) malloc(allocation_size);
  memcpy(new_string, pointer_to_existing, allocation_size);
  return new_string;
}

// Frees a string created by owned_string
extern "C" void free_string(char tag, int64_t value) {
  PRINT("Free string called tag:%d value:%lld", tag, value);
  if (tag == 2) {
    free((void*) value);
  } else {
    printf("\tCOMPILER BUG tried to free a non-string value!");
  }
}

extern "C" void add_file(void *path) {
  char *path_str = (char *) path;
  PRINT("adding file %s\n", path_str);
  files.push_back(std::string(path_str));
}

// Called when done adding files;
extern "C" void init() {
  PRINT("Init called\n");
  std::string current = files.at(files.size() - 1);
  files.pop_back();
  current_file.open(current);
}

int next_file() {
  PRINT("Next file called\n");
  if (files.size() == 0) {
    PRINT("\tThere is no next file\n");
    return 0;
  }
  std::string next_file = files.at(files.size() - 1);
  PRINT("\tNext file is: %s\n", next_file.c_str());
  current_file.close();
  current_file.open(next_file);
  files.pop_back();
  return 1;
}

extern "C" int64_t next_line() {
  fields.clear();
  PRINT("Next line called\n");
  std::getline(current_file, full_line, RS);
  PRINT("\tDone calling first getline %s\n", full_line.c_str());
//  getline(current_file, full_line, RS);
  if (full_line.length() == 0) {
    PRINT("\tLine was empty trying next file\n");

    int found = 0;
    while (next_file()) {
      PRINT("\tGetting line from next file\n");
      getline(current_file, full_line, RS);
      if (full_line.length() == 0) {
        PRINT("\tLine was empty trying next file\n");
        continue;
      } else {
        found = 1;
        PRINT("\tGot something from this file\n");
        break;
      }
    }
    if (!found) {
      PRINT("\tOut of files return false 0\n");
      return 0;
    }
  }
  size_t start = 0;
  for (size_t i = 0; i < full_line.length(); i++) {
    if (full_line[i] == FS) {
      std::string substr = full_line.substr(start, i-start);
      fields.push_back(substr);
      PRINT("\tat %zu to %zu adding substr '%s'\n", start, i, substr.c_str());
      start = i + 1;
    }
    if (i == full_line.length() - 1) {

      std::string substr = full_line.substr(start, i-start+1);
      fields.push_back(substr);
      PRINT("\tadding trail off from %zu to %zu '%s'\n", start, i, substr.c_str());
    }
  }
  PRINT("next line returns 1-true\n");
  return 1;
}

// Returns a pointer to a c_string that the caller now is responsible for
// freeing or 0 if column is too large.
extern "C" int64_t column(char tag, int64_t value) {
  PRINT("column call tag %d value %lld\n", tag, value);
  if (tag == 0) {
    if (value - 1 >= fields.size()) {
      PRINT("\tcolumn to large ret empty string\n");
      std::string empty = "";
      return (int64_t) owned_string(empty); //empty string is repr by 0
    }
    if (value == 0) {
      PRINT("\tcolumn == 0 return full line\n");
      return (int64_t) owned_string(full_line);
    }
    int64_t int_value = (int64_t) owned_string(fields.at(value-1));
    PRINT("\tcolumn normal return fields[col-1] %s int: %lld\n", fields.at(value-1).c_str(), int_value);
    return int_value;
  } else {
    printf("\tCannot get column from tag %d returning $0\n", tag);
    return (int64_t) owned_string(full_line);
  }
}

extern "C" void print_value(char tag, int64_t value) {
  // Is it UB? Yes. Is it easy? Yes;
  union Value val;
  val.int_value = value;
  PRINT("Print value called tag %c value %lld\n", tag, value);
  if (tag == 0) {
    printf("%lld\n", val.int_value);
  } else if (tag == 1) {
    printf("%g\n", (double) val.float_value);
  } else if (tag == 2) {
    printf("%s\n", val.str_value);
  }
}


// 0 is TRUE, but only here. Sorry about that...
extern "C" long int to_bool_i64(char tag, int64_t value) {
  union Value val;
  val.int_value = value;
  PRINT("to_bool_i64 tag:%d value:%lld value:%lf\n", (int) tag, val.int_value, val.float_value);
  if (tag == 0) {
    return val.int_value == 0 ? 0 : 1;
  } else if (tag == 1) {
    return val.float_value == 0.0 ? 0 : 1;
  } else if (tag == 2) {
    PRINT("\tstring is %s", val.str_value);
    return strlen(val.str_value) == 0 ? 1 : 0;
  }
  return 1;
}

extern "C" void print_mismatch() {
  printf("integer float mismatch\n");
}

// something llvm will probably not optimize out. Handy at times to see full IR.
extern "C" double get_float() {
  return 2.2;
}