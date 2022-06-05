#include <stdio.h>
#include <string>
#include <iostream>
#include <vector>
#include <fstream>
#include <math.h>

//#define DEBUG 1

#ifdef DEBUG
#define PRINT(...) do{ fprintf( stderr, __VA_ARGS__ ); } while( false )
#else
#define PRINT(...) do{ } while ( false )
#endif

static char CONVFMT[] = "%.6g";

union Value {
  double float_value;
  char* str_value;
};

static char empty_string[] = "";
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
extern "C" void free_string(char tag, double value) {
  PRINT("Free string called tag:%d value:%g", tag, value);
  if (tag == 1) {
    union Value myVal;
    myVal.float_value = value;
    free( (void*) myVal.str_value );
  } else {
    printf("\tllawk compiler bug: tried to free a non-string value!");
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
  if (files.empty()) {
    PRINT("\t files empty return");
    return;
  }
  std::string current = files.at(files.size() - 1);
  files.pop_back();
  PRINT("\tsetting current file %s", current.c_str());
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
  if (!std::getline(current_file, full_line, RS)) {
    PRINT("\tGet line failed. Continuing as if current line empty\n");
    full_line.clear();
  }
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
extern "C" double column(char tag, double value) {
  PRINT("column call tag %d value %g\n", tag, value);
  union Value val;
  if (tag == 0) {
    if (value == 0) {
      PRINT("\tcolumn == 0 return full line\n");
      val.str_value= owned_string(full_line);
      return val.float_value;
    }
    if (value - 1 >= fields.size()) {
      PRINT("\tcolumn too large ret empty string\n");
      std::string empty = "";
      val.str_value= owned_string(empty);
      return val.float_value;
    }
    val.str_value= owned_string(fields.at(value-1));
    PRINT("\tcolumn normal return fields[col-1] %s\n", fields.at(value-1).c_str());
    return val.float_value;
  } else {
    PRINT("\tCannot get column from tag %d returning $0\n", tag);
    val.str_value= owned_string(full_line);
    return val.float_value;
  }
}

extern "C" void print_value(char tag, double value) {
  // Is it UB? Yes. Is it easy? Yes;
  union Value val;
  val.float_value = value;
  PRINT("Print value called tag %c value %g\n", tag, val.float_value);
  if (tag == 0) {
    if (ceilf(value) == value) {
      int64_t int_value = static_cast<int>(value);
      printf("%lld\n", int_value);
    } else {
      printf("%g\n", val.float_value);
    }
    PRINT("\t Tag is == 0 DONE\n");
  } else if (tag == 1) {
    PRINT("\t Tag is == 1\n");
    printf("%s\n", val.str_value);
  }
}

extern "C" double string_to_number(char tag, double value) {
  // TODO: Strings that can’t be interpreted as valid numbers convert to zero.
  union Value val;
  val.float_value = value;
  PRINT("string_to_number called tag %d value %g\n", tag, value);

  // TODO: This is UB if the string is not representable as a double.
  return atof(val.str_value);
}

extern "C" double number_to_string(char tag, double value) {
  union Value val;
  val.float_value = value;
  PRINT("number_to_string called tag %d value %g\n", tag, value);

  char* result = (char*) malloc(64);
  int bytes_needed_init = snprintf(result, 64, &CONVFMT[0], val.float_value);
  if (bytes_needed_init < 0) {
    printf("FAILURE converting number to string %g tag(%d)\n", value, tag);
  }
  if (bytes_needed_init > 64) {
    free(result);
    result = (char*) malloc(bytes_needed_init);
    int bytes_needed_realloc = snprintf(result, 64, &CONVFMT[0], val.float_value);
    if (bytes_needed_realloc > bytes_needed_init) {
      printf("FAILURE converting number to string %g tag(%d)\n", value, tag);
    }
  }
  val.str_value = result;
  return val.float_value;
}

// something llvm will probably not optimize out. Handy at times to see full IR.
extern "C" double get_float() {
  return 2.2;
}