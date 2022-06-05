; ModuleID = 'llvm-awk'
source_filename = "llvm-awk"
target datalayout = "e-m:o-p270:32:32-p271:32:32-p272:64:64-i64:64-f80:128-n8:16:32:64-S128"

declare extern_weak void @print_value(i8, double)

declare extern_weak double @get_float()

declare extern_weak i64 @next_line()

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
}

define i64 @main() {
init_bb:
  %"file_path string alloc" = alloca [201 x i8], align 1
  store [201 x i8] c"/var/folders/6n/rwqzdntn1hd46w7f_wrcff0c0000gn/T/.tmpyfYaTk/temp_file\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00\00", [201 x i8]* %"file_path string alloc", align 1
  call void @add_file([201 x i8]* %"file_path string alloc")
  call void @init()
  br label %while_test

while_test:                                       ; preds = %while_body, %init_bb
  %get_next_line = call i64 @next_line()
  %cast-int-to-float = bitcast i64 %get_next_line to double
  %value_is_zero_f64 = fcmp oeq double %cast-int-to-float, 0.000000e+00
  %predicate = xor i1 %value_is_zero_f64, true
  br i1 %predicate, label %while_body, label %while_continue

while_body:                                       ; preds = %while_test
  %phi_a_tag = phi i8 [ 0, %init_bb ], [ 0, %while_body ]
  %phi_a_value = phi double [ 0.000000e+00, %init_bb ], [ 1.100000e+00, %while_body ]
  call void @free_if_string(i8 0, double 0.000000e+00)
  br label %while_test

while_continue:                                   ; preds = %while_test
  call void @print_value(i8 %phi_a_tag, double %phi_a_value)
  ret i64 0
}
