; ModuleID = 'llvm-awk'
source_filename = "llvm-awk"
target datalayout = "e-m:o-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"

declare extern_weak void @print_value(i8, double)

declare extern_weak double @get_float()

declare extern_weak double @next_line()

declare extern_weak double @column(i8, double)

declare extern_weak void @add_file([201 x i8]*)

declare extern_weak void @init()

declare extern_weak void @free_string(i8, double)

declare extern_weak double @string_to_number(i8, double)

declare extern_weak double @number_to_string(i8, double)

define void @free_if_string(i8 %0, double %1) {
free_string_init:
  %cmp_with_str_tag = icmp eq i8 %0, 1
  br i1 %cmp_with_str_tag, label %free_string_ffi_call, label %free_string_ret

free_string_ffi_call:                             ; preds = %free_string_init
  call void @free_string(i8 %0, double %1)
  ret void

free_string_ret:                                  ; preds = %free_string_init
  ret void
  %zero_tag = alloca i8, align 1
  %one_tag = alloca i8, align 1
  %zero_value = alloca double, align 8
  %one_value = alloca double, align 8
  store i8 0, i8* %zero_tag, align 1
  store i8 0, i8* %one_tag, align 1
  store double 0.000000e+00, double* %zero_value, align 8
  store double 0.000000e+00, double* %one_value, align 8
}

define i64 @main() {
init_bb:
  %"file_path string alloc" = alloca [201 x i8], align 1
  store [201 x i8] c"data.txt\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00", [201 x i8]* %"file_path string alloc", align 1
  call void @add_file([201 x i8]* %"file_path string alloc")
  call void @init()
  br label %while_test

while_test:                                       ; preds = %while_body, %init_bb
  %get_next_line = call double @next_line()
  %tag = alloca i8, align 1
  %value = alloca double, align 8
  store i8 0, i8* %tag, align 1
  store double %get_next_line, double* %value, align 8
  %tag1 = load i8, i8* %tag, align 1
  %value2 = load double, double* %value, align 8
  %tag_is_zero = icmp eq i8 %tag1, 0
  %value_is_zero_f64 = fcmp oeq double %value2, 0.000000e+00
  %zero_f64 = and i1 %value_is_zero_f64, %tag_is_zero
  %predicate = xor i1 %zero_f64, true
  br i1 %predicate, label %while_body, label %while_continue

while_body:                                       ; preds = %while_test
  %tag3 = alloca i8, align 1
  %value4 = alloca double, align 8
  store i8 0, i8* %tag3, align 1
  store double 1.000000e+00, double* %value4, align 8
  %tag5 = load i8, i8* %tag3, align 1
  %value6 = load double, double* %value4, align 8
  %get_column = call double @column(i8 %tag5, double %value6)
  %tag7 = alloca i8, align 1
  %value8 = alloca double, align 8
  store i8 1, i8* %tag7, align 1
  store double %get_column, double* %value8, align 8
  %tag9 = load i8, i8* %tag7, align 1
  %value10 = load double, double* %value8, align 8
  call void @print_value(i8 %tag9, double %value10)
  br label %while_test

while_continue:                                   ; preds = %while_test
  ret i64 0

