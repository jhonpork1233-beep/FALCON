//! LLVM Backend Code Generator for Falcon (Dynamic Loading)
//! 
//! This module generates native code using LLVM via dynamic loading (libloading).
//! It bypasses compile-time linking by loading "LLVM-C.dll" at runtime.

#![cfg(feature = "llvm")]

use std::collections::HashMap;
use std::mem::ManuallyDrop;
use std::ffi::{CString, CStr, c_void, c_char, c_uint, c_longlong, c_ulonglong, c_double};
use std::path::Path;
use std::ptr;
use std::sync::Arc;

use libloading::{Library, Symbol};

use crate::ir::{IrModule, IrFunction, IrInstruction, IrValue, IrType, IrLiteral, IntType, FloatType};

// Opaque Types
type LLVMContextRef = *mut c_void;
type LLVMModuleRef = *mut c_void;
type LLVMBuilderRef = *mut c_void;
type LLVMTypeRef = *mut c_void;
type LLVMValueRef = *mut c_void;
type LLVMBasicBlockRef = *mut c_void;
type LLVMBool = i32;
type LLVMOpaqueTargetMachine = c_void;
type LLVMTargetMachineRef = *mut LLVMOpaqueTargetMachine;
type LLVMTargetRef = *mut c_void;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LLVMIntPredicate {
    LLVMIntEQ = 32,
    LLVMIntNE = 33,
    LLVMIntUGT = 34,
    LLVMIntUGE = 35,
    LLVMIntULT = 36,
    LLVMIntULE = 37,
    LLVMIntSGT = 38,
    LLVMIntSGE = 39,
    LLVMIntSLT = 40,
    LLVMIntSLE = 41,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LLVMRealPredicate {
    LLVMRealOEQ = 1,
    LLVMRealOGT = 2,
    LLVMRealOGE = 3,
    LLVMRealOLT = 4,
    LLVMRealOLE = 5,
    LLVMRealONE = 6,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LLVMTypeKind {
    LLVMVoidTypeKind = 0,
    LLVMHalfTypeKind,
    LLVMFloatTypeKind,
    LLVMDoubleTypeKind,
    LLVMX86_FP80TypeKind,
    LLVMFP128TypeKind,
    LLVMPPC_FP128TypeKind,
    LLVMLabelTypeKind,
    LLVMIntegerTypeKind,
    LLVMFunctionTypeKind,
    LLVMStructTypeKind,
    LLVMArrayTypeKind,
    LLVMPointerTypeKind,
    LLVMVectorTypeKind,
    LLVMMetadataTypeKind,
    LLVMX86_MMXTypeKind,
    LLVMTokenTypeKind,
    LLVMScalableVectorTypeKind,
    LLVMBFloatTypeKind,
    LLVMX86_AMXTypeKind,
}

// Function Signatures
type FnUnsafe0 = unsafe extern "C" fn() -> ();
type FnContextCreate = unsafe extern "C" fn() -> LLVMContextRef;
type FnContextDispose = unsafe extern "C" fn(LLVMContextRef);
type FnModuleCreate = unsafe extern "C" fn(*const c_char, LLVMContextRef) -> LLVMModuleRef;
type FnModuleDispose = unsafe extern "C" fn(LLVMModuleRef);
type FnCreateBuilder = unsafe extern "C" fn(LLVMContextRef) -> LLVMBuilderRef;
type FnDisposeBuilder = unsafe extern "C" fn(LLVMBuilderRef);
type FnVerifyModule = unsafe extern "C" fn(LLVMModuleRef, u32, *mut *mut c_char) -> LLVMBool;
type FnDisposeMessage = unsafe extern "C" fn(*mut c_char);

// Types
type FnInt64Type = unsafe extern "C" fn(LLVMContextRef) -> LLVMTypeRef;
type FnInt32Type = unsafe extern "C" fn(LLVMContextRef) -> LLVMTypeRef;
type FnInt16Type = unsafe extern "C" fn(LLVMContextRef) -> LLVMTypeRef;
type FnInt8Type = unsafe extern "C" fn(LLVMContextRef) -> LLVMTypeRef;
type FnInt1Type = unsafe extern "C" fn(LLVMContextRef) -> LLVMTypeRef;
type FnDoubleType = unsafe extern "C" fn(LLVMContextRef) -> LLVMTypeRef;
type FnVoidType = unsafe extern "C" fn(LLVMContextRef) -> LLVMTypeRef;
type FnPointerType = unsafe extern "C" fn(LLVMContextRef, c_uint) -> LLVMTypeRef;
type FnFunctionType = unsafe extern "C" fn(LLVMTypeRef, *mut LLVMTypeRef, c_uint, LLVMBool) -> LLVMTypeRef;
type FnStructType = unsafe extern "C" fn(LLVMContextRef, *mut LLVMTypeRef, c_uint, LLVMBool) -> LLVMTypeRef;
type FnTypeOf = unsafe extern "C" fn(LLVMValueRef) -> LLVMTypeRef;
type FnGetTypeKind = unsafe extern "C" fn(LLVMTypeRef) -> LLVMTypeKind;

// Values
type FnConstInt = unsafe extern "C" fn(LLVMTypeRef, c_ulonglong, LLVMBool) -> LLVMValueRef;
type FnConstReal = unsafe extern "C" fn(LLVMTypeRef, c_double) -> LLVMValueRef;
type FnAddFunction = unsafe extern "C" fn(LLVMModuleRef, *const c_char, LLVMTypeRef) -> LLVMValueRef;
type FnGetNamedFunction = unsafe extern "C" fn(LLVMModuleRef, *const c_char) -> LLVMValueRef;
type FnGetParam = unsafe extern "C" fn(LLVMValueRef, c_uint) -> LLVMValueRef;
type FnSetValueName2 = unsafe extern "C" fn(LLVMValueRef, *const c_char, usize);

// Builder
type FnAppendBasicBlock = unsafe extern "C" fn(LLVMContextRef, LLVMValueRef, *const c_char) -> LLVMBasicBlockRef;
type FnPositionBuilder = unsafe extern "C" fn(LLVMBuilderRef, LLVMBasicBlockRef);
type FnBuildRetVoid = unsafe extern "C" fn(LLVMBuilderRef) -> LLVMValueRef;
type FnBuildRet = unsafe extern "C" fn(LLVMBuilderRef, LLVMValueRef) -> LLVMValueRef;
type FnBuildAlloca = unsafe extern "C" fn(LLVMBuilderRef, LLVMTypeRef, *const c_char) -> LLVMValueRef;
type FnBuildStore = unsafe extern "C" fn(LLVMBuilderRef, LLVMValueRef, LLVMValueRef) -> LLVMValueRef;
type FnBuildLoad2 = unsafe extern "C" fn(LLVMBuilderRef, LLVMTypeRef, LLVMValueRef, *const c_char) -> LLVMValueRef;
type FnBuildAdd = unsafe extern "C" fn(LLVMBuilderRef, LLVMValueRef, LLVMValueRef, *const c_char) -> LLVMValueRef;
type FnBuildSub = unsafe extern "C" fn(LLVMBuilderRef, LLVMValueRef, LLVMValueRef, *const c_char) -> LLVMValueRef;
type FnBuildMul = unsafe extern "C" fn(LLVMBuilderRef, LLVMValueRef, LLVMValueRef, *const c_char) -> LLVMValueRef;
type FnBuildSDiv = unsafe extern "C" fn(LLVMBuilderRef, LLVMValueRef, LLVMValueRef, *const c_char) -> LLVMValueRef;
type FnBuildFAdd = unsafe extern "C" fn(LLVMBuilderRef, LLVMValueRef, LLVMValueRef, *const c_char) -> LLVMValueRef;
type FnBuildFSub = unsafe extern "C" fn(LLVMBuilderRef, LLVMValueRef, LLVMValueRef, *const c_char) -> LLVMValueRef;
type FnBuildFMul = unsafe extern "C" fn(LLVMBuilderRef, LLVMValueRef, LLVMValueRef, *const c_char) -> LLVMValueRef;
type FnBuildFDiv = unsafe extern "C" fn(LLVMBuilderRef, LLVMValueRef, LLVMValueRef, *const c_char) -> LLVMValueRef;
type FnBuildICmp = unsafe extern "C" fn(LLVMBuilderRef, LLVMIntPredicate, LLVMValueRef, LLVMValueRef, *const c_char) -> LLVMValueRef;
type FnBuildFCmp = unsafe extern "C" fn(LLVMBuilderRef, LLVMRealPredicate, LLVMValueRef, LLVMValueRef, *const c_char) -> LLVMValueRef;
type FnBuildCall2 = unsafe extern "C" fn(LLVMBuilderRef, LLVMTypeRef, LLVMValueRef, *mut LLVMValueRef, c_uint, *const c_char) -> LLVMValueRef;
type FnBuildBr = unsafe extern "C" fn(LLVMBuilderRef, LLVMBasicBlockRef) -> LLVMValueRef;
type FnBuildCondBr = unsafe extern "C" fn(LLVMBuilderRef, LLVMValueRef, LLVMBasicBlockRef, LLVMBasicBlockRef) -> LLVMValueRef;
type FnBuildGlobalStringPtr = unsafe extern "C" fn(LLVMBuilderRef, *const c_char, *const c_char) -> LLVMValueRef;
type FnBuildStructGEP2 = unsafe extern "C" fn(LLVMBuilderRef, LLVMTypeRef, LLVMValueRef, c_uint, *const c_char) -> LLVMValueRef;
type FnBuildGEP2 = unsafe extern "C" fn(LLVMBuilderRef, LLVMTypeRef, LLVMValueRef, *mut LLVMValueRef, c_uint, *const c_char) -> LLVMValueRef;
type FnBuildSExt = unsafe extern "C" fn(LLVMBuilderRef, LLVMValueRef, LLVMTypeRef, *const c_char) -> LLVMValueRef;
type FnBuildZExt = unsafe extern "C" fn(LLVMBuilderRef, LLVMValueRef, LLVMTypeRef, *const c_char) -> LLVMValueRef;
type FnBuildTrunc = unsafe extern "C" fn(LLVMBuilderRef, LLVMValueRef, LLVMTypeRef, *const c_char) -> LLVMValueRef;
type FnBuildIntToPtr = unsafe extern "C" fn(LLVMBuilderRef, LLVMValueRef, LLVMTypeRef, *const c_char) -> LLVMValueRef;
type FnBuildPtrToInt = unsafe extern "C" fn(LLVMBuilderRef, LLVMValueRef, LLVMTypeRef, *const c_char) -> LLVMValueRef;
type FnBuildFPToSI = unsafe extern "C" fn(LLVMBuilderRef, LLVMValueRef, LLVMTypeRef, *const c_char) -> LLVMValueRef;
type FnBuildSIToFP = unsafe extern "C" fn(LLVMBuilderRef, LLVMValueRef, LLVMTypeRef, *const c_char) -> LLVMValueRef;
type FnSetVolatile = unsafe extern "C" fn(LLVMValueRef, LLVMBool);
type FnArrayType = unsafe extern "C" fn(LLVMTypeRef, c_uint) -> LLVMTypeRef;
type FnGetIntTypeWidth = unsafe extern "C" fn(LLVMTypeRef) -> c_uint;
// Bitwise operations
type FnBuildAnd = unsafe extern "C" fn(LLVMBuilderRef, LLVMValueRef, LLVMValueRef, *const c_char) -> LLVMValueRef;
type FnBuildOr = unsafe extern "C" fn(LLVMBuilderRef, LLVMValueRef, LLVMValueRef, *const c_char) -> LLVMValueRef;
type FnBuildXor = unsafe extern "C" fn(LLVMBuilderRef, LLVMValueRef, LLVMValueRef, *const c_char) -> LLVMValueRef;
type FnBuildShl = unsafe extern "C" fn(LLVMBuilderRef, LLVMValueRef, LLVMValueRef, *const c_char) -> LLVMValueRef;
type FnBuildAShr = unsafe extern "C" fn(LLVMBuilderRef, LLVMValueRef, LLVMValueRef, *const c_char) -> LLVMValueRef;
type FnBuildNot = unsafe extern "C" fn(LLVMBuilderRef, LLVMValueRef, *const c_char) -> LLVMValueRef;

type FnGetInsertBlock = unsafe extern "C" fn(LLVMBuilderRef) -> LLVMBasicBlockRef;
type FnGetBasicBlockTerminator = unsafe extern "C" fn(LLVMBasicBlockRef) -> LLVMValueRef;
type FnGetFirstInstruction = unsafe extern "C" fn(LLVMBasicBlockRef) -> LLVMValueRef;
type FnPositionBuilderBefore = unsafe extern "C" fn(LLVMBuilderRef, LLVMValueRef);
type FnGetFirstBasicBlock = unsafe extern "C" fn(LLVMValueRef) -> LLVMBasicBlockRef;
type FnGetNextBasicBlock = unsafe extern "C" fn(LLVMBasicBlockRef) -> LLVMBasicBlockRef;
type FnGetBasicBlockName = unsafe extern "C" fn(LLVMBasicBlockRef) -> *const c_char;
type FnPrintModuleToString = unsafe extern "C" fn(LLVMModuleRef) -> *mut c_char;
type FnGetReturnType = unsafe extern "C" fn(LLVMTypeRef) -> LLVMTypeRef;
type FnCountParamTypes = unsafe extern "C" fn(LLVMTypeRef) -> c_uint;
type FnGetParamTypes = unsafe extern "C" fn(LLVMTypeRef, *mut LLVMTypeRef);

// Target
type FnInit = unsafe extern "C" fn();
type FnGetDefaultTriple = unsafe extern "C" fn() -> *mut c_char;
type FnGetTargetFromTriple = unsafe extern "C" fn(*const c_char, *mut LLVMTargetRef, *mut *mut c_char) -> LLVMBool;
type FnCreateTargetMachine = unsafe extern "C" fn(LLVMTargetRef, *const c_char, *const c_char, *const c_char, u32, u32, u32) -> LLVMTargetMachineRef;
type FnTargetMachineEmitToFile = unsafe extern "C" fn(LLVMTargetMachineRef, LLVMModuleRef, *mut c_char, u32, *mut *mut c_char) -> LLVMBool;

// PassBuilder (New Pass Manager)
type LLVMPassBuilderOptionsRef = *mut c_void;
type LLVMErrorRef = *mut c_void;
type FnCreatePassBuilderOptions = unsafe extern "C" fn() -> LLVMPassBuilderOptionsRef;
type FnDisposePassBuilderOptions = unsafe extern "C" fn(LLVMPassBuilderOptionsRef);
type FnRunPasses = unsafe extern "C" fn(LLVMModuleRef, *const c_char, LLVMTargetMachineRef, LLVMPassBuilderOptionsRef) -> LLVMErrorRef;
type FnGetErrorMessage = unsafe extern "C" fn(LLVMErrorRef) -> *mut c_char;
type FnConsumeError = unsafe extern "C" fn(LLVMErrorRef);

// ExecutionEngine / MCJIT for JIT execution
type LLVMExecutionEngineRef = *mut c_void;
type LLVMGenericValueRef = *mut c_void;
type FnLinkInMCJIT = unsafe extern "C" fn();
type FnCreateMCJITCompilerForModule = unsafe extern "C" fn(
    *mut LLVMExecutionEngineRef,
    LLVMModuleRef,
    *mut c_void, // MCJITCompilerOptions
    usize,       // SizeOfOptions
    *mut *mut c_char
) -> LLVMBool;
type FnDisposeExecutionEngine = unsafe extern "C" fn(LLVMExecutionEngineRef);
type FnGetFunctionAddress = unsafe extern "C" fn(LLVMExecutionEngineRef, *const c_char) -> u64;
type FnRunFunction = unsafe extern "C" fn(
    LLVMExecutionEngineRef,
    LLVMValueRef,
    c_uint,
    *mut LLVMGenericValueRef
) -> LLVMGenericValueRef;
type FnGenericValueToInt = unsafe extern "C" fn(LLVMGenericValueRef, LLVMBool) -> c_ulonglong;
type FnDisposeGenericValue = unsafe extern "C" fn(LLVMGenericValueRef);
type FnAddGlobalMapping = unsafe extern "C" fn(LLVMExecutionEngineRef, LLVMValueRef, *mut c_void);

// JIT Runtime functions (called from JIT-compiled code)
extern "C" fn jit_print_int(val: i64) {
    print!("{}", val);
    use std::io::Write;
    std::io::stdout().flush().ok();
}

extern "C" fn jit_print_i32(val: i32) {
    print!("{}", val);
    use std::io::Write;
    std::io::stdout().flush().ok();
}

extern "C" fn jit_println(s: *const c_char) {
    unsafe {
        if !s.is_null() {
            let cstr = CStr::from_ptr(s);
            println!("{}", cstr.to_string_lossy());
        } else {
            println!();
        }
    }
}

extern "C" fn jit_print(s: *const c_char) {
    unsafe {
        if !s.is_null() {
            let cstr = CStr::from_ptr(s);
            print!("{}", cstr.to_string_lossy());
            use std::io::Write;
            std::io::stdout().flush().ok();
        }
    }
}

extern "C" fn jit_bounds_check(index: i64, length: i64) {
    if index < 0 {
        eprintln!("PANIC: Array index out of bounds: negative index");
        std::process::abort();
    }
    if length >= 0 && index >= length {
        eprintln!("PANIC: Array index out of bounds: index exceeds length");
        std::process::abort();
    }
}


pub struct LLVMApi {
    lib: ManuallyDrop<Library>,
    
    // Symbols
    context_create: Symbol<'static, FnContextCreate>,
    context_dispose: Symbol<'static, FnContextDispose>,
    module_create: Symbol<'static, FnModuleCreate>,
    module_dispose: Symbol<'static, FnModuleDispose>,
    create_builder: Symbol<'static, FnCreateBuilder>,
    dispose_builder: Symbol<'static, FnDisposeBuilder>,
    verify_module: Symbol<'static, FnVerifyModule>,
    dispose_message: Symbol<'static, FnDisposeMessage>,
    print_module_to_string: Symbol<'static, FnPrintModuleToString>,
    
    int64_type: Symbol<'static, FnInt64Type>,
    int32_type: Symbol<'static, FnInt32Type>,
    int16_type: Symbol<'static, FnInt16Type>,
    int8_type: Symbol<'static, FnInt8Type>,
    int1_type: Symbol<'static, FnInt1Type>,
    double_type: Symbol<'static, FnDoubleType>,
    void_type: Symbol<'static, FnVoidType>,
    ptr_type: Symbol<'static, FnPointerType>,
    function_type: Symbol<'static, FnFunctionType>,
    struct_type: Symbol<'static, FnStructType>,
    type_of: Symbol<'static, FnTypeOf>,
    get_type_kind: Symbol<'static, FnGetTypeKind>,
    
    const_int: Symbol<'static, FnConstInt>,
    const_real: Symbol<'static, FnConstReal>,
    add_function: Symbol<'static, FnAddFunction>,
    get_named_function: Symbol<'static, FnGetNamedFunction>,
    get_param: Symbol<'static, FnGetParam>,
    set_value_name2: Symbol<'static, FnSetValueName2>,
    
    append_basic_block: Symbol<'static, FnAppendBasicBlock>,
    position_builder: Symbol<'static, FnPositionBuilder>,
    build_ret_void: Symbol<'static, FnBuildRetVoid>,
    build_ret: Symbol<'static, FnBuildRet>,
    build_alloca: Symbol<'static, FnBuildAlloca>,
    build_store: Symbol<'static, FnBuildStore>,
    build_load2: Symbol<'static, FnBuildLoad2>,
    build_add: Symbol<'static, FnBuildAdd>,
    build_sub: Symbol<'static, FnBuildSub>,
    build_mul: Symbol<'static, FnBuildMul>,
    build_sdiv: Symbol<'static, FnBuildSDiv>,
    build_fadd: Symbol<'static, FnBuildFAdd>,
    build_fsub: Symbol<'static, FnBuildFSub>,
    build_fmul: Symbol<'static, FnBuildFMul>,
    build_fdiv: Symbol<'static, FnBuildFDiv>,
    build_icmp: Symbol<'static, FnBuildICmp>,
    build_fcmp: Symbol<'static, FnBuildFCmp>,
    build_call2: Symbol<'static, FnBuildCall2>,
    build_br: Symbol<'static, FnBuildBr>,
    build_cond_br: Symbol<'static, FnBuildCondBr>,
    build_global_string_ptr: Symbol<'static, FnBuildGlobalStringPtr>,
    build_struct_gep2: Symbol<'static, FnBuildStructGEP2>,
    build_gep2: Symbol<'static, FnBuildGEP2>,
    build_sext: Symbol<'static, FnBuildSExt>,
    build_zext: Symbol<'static, FnBuildZExt>,
    build_trunc: Symbol<'static, FnBuildTrunc>,
    build_int_to_ptr: Symbol<'static, FnBuildIntToPtr>,
    build_ptr_to_int: Symbol<'static, FnBuildPtrToInt>,
    build_fptosi: Symbol<'static, FnBuildFPToSI>,
    build_sitofp: Symbol<'static, FnBuildSIToFP>,
    set_volatile: Symbol<'static, FnSetVolatile>,
    array_type: Symbol<'static, FnArrayType>,
    get_int_type_width: Symbol<'static, FnGetIntTypeWidth>,
    // Bitwise operations
    build_and: Symbol<'static, FnBuildAnd>,
    build_or: Symbol<'static, FnBuildOr>,
    build_xor: Symbol<'static, FnBuildXor>,
    build_shl: Symbol<'static, FnBuildShl>,
    build_ashr: Symbol<'static, FnBuildAShr>,
    build_not: Symbol<'static, FnBuildNot>,
    
    get_insert_block: Symbol<'static, FnGetInsertBlock>,
    get_basic_block_terminator: Symbol<'static, FnGetBasicBlockTerminator>,
    get_first_instruction: Symbol<'static, FnGetFirstInstruction>,
    position_builder_before: Symbol<'static, FnPositionBuilderBefore>,
    get_first_basic_block: Symbol<'static, FnGetFirstBasicBlock>,
    get_next_basic_block: Symbol<'static, FnGetNextBasicBlock>,
    get_basic_block_name: Symbol<'static, FnGetBasicBlockName>,
    get_return_type: Symbol<'static, FnGetReturnType>,
    count_param_types: Symbol<'static, FnCountParamTypes>,
    get_param_types: Symbol<'static, FnGetParamTypes>,
    
    // Target
    init_target_infos: Symbol<'static, FnInit>,
    init_targets: Symbol<'static, FnInit>,
    init_target_mcs: Symbol<'static, FnInit>,
    init_asm_printers: Symbol<'static, FnInit>,
    init_asm_parsers: Symbol<'static, FnInit>,
    
    // Optional additional targets
    init_aarch64_target_infos: Option<Symbol<'static, FnInit>>,
    init_aarch64_targets: Option<Symbol<'static, FnInit>>,
    init_aarch64_target_mcs: Option<Symbol<'static, FnInit>>,
    init_aarch64_asm_printers: Option<Symbol<'static, FnInit>>,
    init_aarch64_asm_parsers: Option<Symbol<'static, FnInit>>,
    
    init_riscv_target_infos: Option<Symbol<'static, FnInit>>,
    init_riscv_targets: Option<Symbol<'static, FnInit>>,
    init_riscv_target_mcs: Option<Symbol<'static, FnInit>>,
    init_riscv_asm_printers: Option<Symbol<'static, FnInit>>,
    init_riscv_asm_parsers: Option<Symbol<'static, FnInit>>,
    
    init_avr_target_infos: Option<Symbol<'static, FnInit>>,
    init_avr_targets: Option<Symbol<'static, FnInit>>,
    init_avr_target_mcs: Option<Symbol<'static, FnInit>>,
    init_avr_asm_printers: Option<Symbol<'static, FnInit>>,
    init_avr_asm_parsers: Option<Symbol<'static, FnInit>>,
    get_default_triple: Symbol<'static, FnGetDefaultTriple>,
    get_target_from_triple: Symbol<'static, FnGetTargetFromTriple>,
    create_target_machine: Symbol<'static, FnCreateTargetMachine>,
    emit_to_file: Symbol<'static, FnTargetMachineEmitToFile>,
    
    // PassBuilder (New Pass Manager)
    create_pass_builder_options: Symbol<'static, FnCreatePassBuilderOptions>,
    dispose_pass_builder_options: Symbol<'static, FnDisposePassBuilderOptions>,
    run_passes: Symbol<'static, FnRunPasses>,
    get_error_message: Symbol<'static, FnGetErrorMessage>,
    consume_error: Symbol<'static, FnConsumeError>,
    
    // MCJIT for JIT execution
    link_in_mcjit: Symbol<'static, FnLinkInMCJIT>,
    create_mcjit_compiler: Symbol<'static, FnCreateMCJITCompilerForModule>,
    dispose_execution_engine: Symbol<'static, FnDisposeExecutionEngine>,
    get_function_address: Symbol<'static, FnGetFunctionAddress>,
    run_function: Symbol<'static, FnRunFunction>,
    generic_value_to_int: Symbol<'static, FnGenericValueToInt>,
    dispose_generic_value: Symbol<'static, FnDisposeGenericValue>,
    add_global_mapping: Symbol<'static, FnAddGlobalMapping>,
}

impl LLVMApi {
    pub unsafe fn new() -> Result<Self, String> {
        let paths = [
            "C:\\Program Files\\LLVM\\bin\\LLVM-C.dll",
            "C:\\Program Files\\LLVM\\bin\\libllvm.dll",
        ];
        
        let mut lib = None;
        for path in &paths {
            if let Ok(l) = Library::new(path) {
                lib = Some(l);
                break;
            }
        }
        
        let lib = ManuallyDrop::new(lib.ok_or("Failed to load LLVM DLL")?);
        
        // Helper to load symbol (uses &*lib to borrow from ManuallyDrop)
        macro_rules! load {
            ($name:expr, $ty:ty) => {
                lib.get::<$ty>($name).map(|s| unsafe { std::mem::transmute(s) })
                   .map_err(|e| format!("Failed to load {}: {}", std::str::from_utf8($name).unwrap(), e))
            }
        }
        
        // Pre-extract optional target symbols
        let opt_aarch64_ti: Option<Symbol<'static, FnInit>> = lib.get::<FnInit>(b"LLVMInitializeAArch64TargetInfo").ok().map(|s| unsafe { std::mem::transmute(s) });
        let opt_aarch64_t: Option<Symbol<'static, FnInit>> = lib.get::<FnInit>(b"LLVMInitializeAArch64Target").ok().map(|s| unsafe { std::mem::transmute(s) });
        let opt_aarch64_mc: Option<Symbol<'static, FnInit>> = lib.get::<FnInit>(b"LLVMInitializeAArch64TargetMC").ok().map(|s| unsafe { std::mem::transmute(s) });
        let opt_aarch64_ap: Option<Symbol<'static, FnInit>> = lib.get::<FnInit>(b"LLVMInitializeAArch64AsmPrinter").ok().map(|s| unsafe { std::mem::transmute(s) });
        let opt_aarch64_apar: Option<Symbol<'static, FnInit>> = lib.get::<FnInit>(b"LLVMInitializeAArch64AsmParser").ok().map(|s| unsafe { std::mem::transmute(s) });
        let opt_riscv_ti: Option<Symbol<'static, FnInit>> = lib.get::<FnInit>(b"LLVMInitializeRISCVTargetInfo").ok().map(|s| unsafe { std::mem::transmute(s) });
        let opt_riscv_t: Option<Symbol<'static, FnInit>> = lib.get::<FnInit>(b"LLVMInitializeRISCVTarget").ok().map(|s| unsafe { std::mem::transmute(s) });
        let opt_riscv_mc: Option<Symbol<'static, FnInit>> = lib.get::<FnInit>(b"LLVMInitializeRISCVTargetMC").ok().map(|s| unsafe { std::mem::transmute(s) });
        let opt_riscv_ap: Option<Symbol<'static, FnInit>> = lib.get::<FnInit>(b"LLVMInitializeRISCVAsmPrinter").ok().map(|s| unsafe { std::mem::transmute(s) });
        let opt_riscv_apar: Option<Symbol<'static, FnInit>> = lib.get::<FnInit>(b"LLVMInitializeRISCVAsmParser").ok().map(|s| unsafe { std::mem::transmute(s) });
        let opt_avr_ti: Option<Symbol<'static, FnInit>> = lib.get::<FnInit>(b"LLVMInitializeAVRTargetInfo").ok().map(|s| unsafe { std::mem::transmute(s) });
        let opt_avr_t: Option<Symbol<'static, FnInit>> = lib.get::<FnInit>(b"LLVMInitializeAVRTarget").ok().map(|s| unsafe { std::mem::transmute(s) });
        let opt_avr_mc: Option<Symbol<'static, FnInit>> = lib.get::<FnInit>(b"LLVMInitializeAVRTargetMC").ok().map(|s| unsafe { std::mem::transmute(s) });
        let opt_avr_ap: Option<Symbol<'static, FnInit>> = lib.get::<FnInit>(b"LLVMInitializeAVRAsmPrinter").ok().map(|s| unsafe { std::mem::transmute(s) });
        let opt_avr_apar: Option<Symbol<'static, FnInit>> = lib.get::<FnInit>(b"LLVMInitializeAVRAsmParser").ok().map(|s| unsafe { std::mem::transmute(s) });
        
        Ok(Self {
            context_create: load!(b"LLVMContextCreate", FnContextCreate)?,
            context_dispose: load!(b"LLVMContextDispose", FnContextDispose)?,
            module_create: load!(b"LLVMModuleCreateWithNameInContext", FnModuleCreate)?,
            module_dispose: load!(b"LLVMDisposeModule", FnModuleDispose)?,
            create_builder: load!(b"LLVMCreateBuilderInContext", FnCreateBuilder)?,
            dispose_builder: load!(b"LLVMDisposeBuilder", FnDisposeBuilder)?,
            verify_module: load!(b"LLVMVerifyModule", FnVerifyModule)?,
            dispose_message: load!(b"LLVMDisposeMessage", FnDisposeMessage)?,
            print_module_to_string: load!(b"LLVMPrintModuleToString", FnPrintModuleToString)?,
            
            int64_type: load!(b"LLVMInt64TypeInContext", FnInt64Type)?,
            int32_type: load!(b"LLVMInt32TypeInContext", FnInt32Type)?,
            int16_type: load!(b"LLVMInt16TypeInContext", FnInt16Type)?,
            int8_type: load!(b"LLVMInt8TypeInContext", FnInt8Type)?,
            int1_type: load!(b"LLVMInt1TypeInContext", FnInt1Type)?,
            double_type: load!(b"LLVMDoubleTypeInContext", FnDoubleType)?,
            void_type: load!(b"LLVMVoidTypeInContext", FnVoidType)?,
            ptr_type: load!(b"LLVMPointerTypeInContext", FnPointerType)?,
            function_type: load!(b"LLVMFunctionType", FnFunctionType)?,
            struct_type: load!(b"LLVMStructTypeInContext", FnStructType)?,
            type_of: load!(b"LLVMTypeOf", FnTypeOf)?,
            get_type_kind: load!(b"LLVMGetTypeKind", FnGetTypeKind)?,
            
            const_int: load!(b"LLVMConstInt", FnConstInt)?,
            const_real: load!(b"LLVMConstReal", FnConstReal)?,
            add_function: load!(b"LLVMAddFunction", FnAddFunction)?,
            get_named_function: load!(b"LLVMGetNamedFunction", FnGetNamedFunction)?,
            get_param: load!(b"LLVMGetParam", FnGetParam)?,
            set_value_name2: load!(b"LLVMSetValueName2", FnSetValueName2)?,
            
            append_basic_block: load!(b"LLVMAppendBasicBlockInContext", FnAppendBasicBlock)?,
            position_builder: load!(b"LLVMPositionBuilderAtEnd", FnPositionBuilder)?,
            build_ret_void: load!(b"LLVMBuildRetVoid", FnBuildRetVoid)?,
            build_ret: load!(b"LLVMBuildRet", FnBuildRet)?,
            build_alloca: load!(b"LLVMBuildAlloca", FnBuildAlloca)?,
            build_store: load!(b"LLVMBuildStore", FnBuildStore)?,
            build_load2: load!(b"LLVMBuildLoad2", FnBuildLoad2)?,
            build_add: load!(b"LLVMBuildAdd", FnBuildAdd)?,
            build_sub: load!(b"LLVMBuildSub", FnBuildSub)?,
            build_mul: load!(b"LLVMBuildMul", FnBuildMul)?,
            build_sdiv: load!(b"LLVMBuildSDiv", FnBuildSDiv)?,
            build_fadd: load!(b"LLVMBuildFAdd", FnBuildFAdd)?,
            build_fsub: load!(b"LLVMBuildFSub", FnBuildFSub)?,
            build_fmul: load!(b"LLVMBuildFMul", FnBuildFMul)?,
            build_fdiv: load!(b"LLVMBuildFDiv", FnBuildFDiv)?,
            build_icmp: load!(b"LLVMBuildICmp", FnBuildICmp)?,
            build_fcmp: load!(b"LLVMBuildFCmp", FnBuildFCmp)?,
            build_call2: load!(b"LLVMBuildCall2", FnBuildCall2)?,
            build_br: load!(b"LLVMBuildBr", FnBuildBr)?,
            build_cond_br: load!(b"LLVMBuildCondBr", FnBuildCondBr)?,
            build_global_string_ptr: load!(b"LLVMBuildGlobalStringPtr", FnBuildGlobalStringPtr)?,
            build_struct_gep2: load!(b"LLVMBuildStructGEP2", FnBuildStructGEP2)?,
            build_gep2: load!(b"LLVMBuildGEP2", FnBuildGEP2)?,
            build_sext: load!(b"LLVMBuildSExt", FnBuildSExt)?,
            build_zext: load!(b"LLVMBuildZExt", FnBuildZExt)?,
            build_trunc: load!(b"LLVMBuildTrunc", FnBuildTrunc)?,
            build_int_to_ptr: load!(b"LLVMBuildIntToPtr", FnBuildIntToPtr)?,
            build_ptr_to_int: load!(b"LLVMBuildPtrToInt", FnBuildPtrToInt)?,
            build_fptosi: load!(b"LLVMBuildFPToSI", FnBuildFPToSI)?,
            build_sitofp: load!(b"LLVMBuildSIToFP", FnBuildSIToFP)?,
            set_volatile: load!(b"LLVMSetVolatile", FnSetVolatile)?,
            array_type: load!(b"LLVMArrayType", FnArrayType)?,
            get_int_type_width: load!(b"LLVMGetIntTypeWidth", FnGetIntTypeWidth)?,
            // Bitwise operations
            build_and: load!(b"LLVMBuildAnd", FnBuildAnd)?,
            build_or: load!(b"LLVMBuildOr", FnBuildOr)?,
            build_xor: load!(b"LLVMBuildXor", FnBuildXor)?,
            build_shl: load!(b"LLVMBuildShl", FnBuildShl)?,
            build_ashr: load!(b"LLVMBuildAShr", FnBuildAShr)?,
            build_not: load!(b"LLVMBuildNot", FnBuildNot)?,
            
            get_insert_block: load!(b"LLVMGetInsertBlock", FnGetInsertBlock)?,
            get_basic_block_terminator: load!(b"LLVMGetBasicBlockTerminator", FnGetBasicBlockTerminator)?,
            get_first_instruction: load!(b"LLVMGetFirstInstruction", FnGetFirstInstruction)?,
            position_builder_before: load!(b"LLVMPositionBuilderBefore", FnPositionBuilderBefore)?,
            get_first_basic_block: load!(b"LLVMGetFirstBasicBlock", FnGetFirstBasicBlock)?,
            get_next_basic_block: load!(b"LLVMGetNextBasicBlock", FnGetNextBasicBlock)?,
            get_basic_block_name: load!(b"LLVMGetBasicBlockName", FnGetBasicBlockName)?,
            get_return_type: load!(b"LLVMGetReturnType", FnGetReturnType)?,
            count_param_types: load!(b"LLVMCountParamTypes", FnCountParamTypes)?,
            get_param_types: load!(b"LLVMGetParamTypes", FnGetParamTypes)?,
            
            init_target_infos: load!(b"LLVMInitializeX86TargetInfo", FnInit)?,
            init_targets: load!(b"LLVMInitializeX86Target", FnInit)?,
            init_target_mcs: load!(b"LLVMInitializeX86TargetMC", FnInit)?,
            init_asm_printers: load!(b"LLVMInitializeX86AsmPrinter", FnInit)?,
            init_asm_parsers: load!(b"LLVMInitializeX86AsmParser", FnInit)?,
            
            init_aarch64_target_infos: opt_aarch64_ti,
            init_aarch64_targets: opt_aarch64_t,
            init_aarch64_target_mcs: opt_aarch64_mc,
            init_aarch64_asm_printers: opt_aarch64_ap,
            init_aarch64_asm_parsers: opt_aarch64_apar,
            
            init_riscv_target_infos: opt_riscv_ti,
            init_riscv_targets: opt_riscv_t,
            init_riscv_target_mcs: opt_riscv_mc,
            init_riscv_asm_printers: opt_riscv_ap,
            init_riscv_asm_parsers: opt_riscv_apar,
            
            init_avr_target_infos: opt_avr_ti,
            init_avr_targets: opt_avr_t,
            init_avr_target_mcs: opt_avr_mc,
            init_avr_asm_printers: opt_avr_ap,
            init_avr_asm_parsers: opt_avr_apar,
            get_default_triple: load!(b"LLVMGetDefaultTargetTriple", FnGetDefaultTriple)?,
            get_target_from_triple: load!(b"LLVMGetTargetFromTriple", FnGetTargetFromTriple)?,
            create_target_machine: load!(b"LLVMCreateTargetMachine", FnCreateTargetMachine)?,
            emit_to_file: load!(b"LLVMTargetMachineEmitToFile", FnTargetMachineEmitToFile)?,
            
            // PassBuilder (New Pass Manager)
            create_pass_builder_options: load!(b"LLVMCreatePassBuilderOptions", FnCreatePassBuilderOptions)?,
            dispose_pass_builder_options: load!(b"LLVMDisposePassBuilderOptions", FnDisposePassBuilderOptions)?,
            run_passes: load!(b"LLVMRunPasses", FnRunPasses)?,
            get_error_message: load!(b"LLVMGetErrorMessage", FnGetErrorMessage)?,
            consume_error: load!(b"LLVMConsumeError", FnConsumeError)?,
            
            // MCJIT for JIT execution
            link_in_mcjit: load!(b"LLVMLinkInMCJIT", FnLinkInMCJIT)?,
            create_mcjit_compiler: load!(b"LLVMCreateMCJITCompilerForModule", FnCreateMCJITCompilerForModule)?,
            dispose_execution_engine: load!(b"LLVMDisposeExecutionEngine", FnDisposeExecutionEngine)?,
            get_function_address: load!(b"LLVMGetFunctionAddress", FnGetFunctionAddress)?,
            run_function: load!(b"LLVMRunFunction", FnRunFunction)?,
            generic_value_to_int: load!(b"LLVMGenericValueToInt", FnGenericValueToInt)?,
            dispose_generic_value: load!(b"LLVMDisposeGenericValue", FnDisposeGenericValue)?,
            add_global_mapping: load!(b"LLVMAddGlobalMapping", FnAddGlobalMapping)?,
            
            lib: lib,
        })
    }
}

pub struct LlvmCodegen {
    api: Arc<LLVMApi>,
    context: LLVMContextRef,
    module: LLVMModuleRef,
    builder: LLVMBuilderRef,
    execution_engine: LLVMExecutionEngineRef,
    variables: HashMap<String, LLVMValueRef>,
    variable_types: HashMap<String, LLVMTypeRef>,
    function_types: HashMap<String, LLVMTypeRef>,
    struct_types: HashMap<String, LLVMTypeRef>,
    struct_field_indices: HashMap<String, HashMap<String, u32>>,
    struct_field_types: HashMap<String, HashMap<String, LLVMTypeRef>>,
    current_function: Option<LLVMValueRef>,
    entry_block: Option<LLVMBasicBlockRef>,
    current_return_type: Option<LLVMTypeRef>,
    // Type interning caches
    enum_type: Option<LLVMTypeRef>,   // Cached {i32 tag, i64 payload} type
    range_type: Option<LLVMTypeRef>,  // Cached {i64 start, i64 end} type
    // Enum definitions for tag lookup: enum_name -> [(variant_name, tag)]
    enum_defs: HashMap<String, Vec<(String, u32)>>,
    // Closure tracking: variable_name -> generated_function_name  
    closure_functions: HashMap<String, String>,
    // Temporary tracking for closures: temp_name -> generated_function_name
    // Used to propagate closure info through Move instructions
    temp_to_closure: HashMap<String, String>,
}

impl LlvmCodegen {
    pub fn new(module_name: &str) -> Self {
        unsafe {
            let api = Arc::new(LLVMApi::new().expect("Failed to load LLVM API"));
            let context = (api.context_create)();
            let c_name = CString::new(module_name).unwrap();
            let module = (api.module_create)(c_name.as_ptr(), context);
            let builder = (api.create_builder)(context);
            
            Self {
                api,
                context,
                module,
                builder,
                execution_engine: ptr::null_mut(),
                variables: HashMap::new(),
                variable_types: HashMap::new(),
                function_types: HashMap::new(),
                struct_types: HashMap::new(),
                struct_field_indices: HashMap::new(),
                struct_field_types: HashMap::new(),
                current_function: None,
                entry_block: None,
                current_return_type: None,
                enum_type: None,
                range_type: None,
                enum_defs: HashMap::new(),
                closure_functions: HashMap::new(),
                temp_to_closure: HashMap::new(),
            }
        }
    }

    pub unsafe fn shutdown(&mut self) {
        #[cfg(windows)]
        {
            if !self.execution_engine.is_null() {
                // Windows MCJIT teardown is unstable in this process model.
                // If a JIT engine exists, skip all explicit LLVM disposals and
                // let process teardown reclaim resources.
                self.execution_engine = ptr::null_mut();
                self.builder = ptr::null_mut();
                self.module = ptr::null_mut();
                self.context = ptr::null_mut();
                return;
            }
        }

        if !self.execution_engine.is_null() {
            #[cfg(not(windows))]
            {
                (self.api.dispose_execution_engine)(self.execution_engine);
            }
            #[cfg(windows)]
            {
                // Should be unreachable because Windows JIT engine path returns early above.
            }
            self.execution_engine = ptr::null_mut();
        }
        if !self.builder.is_null() {
            (self.api.dispose_builder)(self.builder);
            self.builder = ptr::null_mut();
        }
        if !self.module.is_null() {
            (self.api.module_dispose)(self.module);
            self.module = ptr::null_mut();
        }
        if !self.context.is_null() {
            (self.api.context_dispose)(self.context);
            self.context = ptr::null_mut();
        }
    }
    
    // ... Implement methods calling (self.api.func)(...) ...
    pub fn compile(&mut self, ir_module: &IrModule) -> Result<(), String> {
        unsafe {
            // Load enum definitions for tag lookup
            for enum_def in &ir_module.enums {
                let variants: Vec<(String, u32)> = enum_def.variants.iter()
                    .map(|v| (v.name.clone(), v.tag))
                    .collect();
                self.enum_defs.insert(enum_def.name.clone(), variants);
            }
            
            // Declare all functions first
            for func in &ir_module.functions {
                self.declare_function(func)?;
            }
            
            // Compile struct types
            for struct_def in &ir_module.structs {
                self.compile_struct_type(struct_def)?;
            }
            
            // Declare runtime functions
            self.declare_runtime_functions()?;
            
            // Compile bodies
            for func in &ir_module.functions {
                self.compile_function(func)?;
            }
            
            // Verify module in debug builds to catch LLVM IR bugs early
            #[cfg(debug_assertions)]
            {
                let mut error_msg: *mut c_char = ptr::null_mut();
                let failed = (self.api.verify_module)(
                    self.module, 
                    2, // ReturnStatusAction
                    &mut error_msg
                );
                
                if failed != 0 {
                    let msg = CStr::from_ptr(error_msg).to_string_lossy().into_owned();
                    (self.api.dispose_message)(error_msg);
                    return Err(format!("Module verification failed: {}", msg));
                }
            }
        }
        
        Ok(())
    }

    /// Run LLVM optimization passes on the module
    /// 
    /// opt_level:
    ///   0 = No optimization
    ///   1 = Basic optimizations (mem2reg, simplifycfg, instcombine)
    ///   2 = Standard O2 pipeline
    ///   3 = Aggressive O3 pipeline
    pub fn optimize(&mut self, opt_level: u8) -> Result<(), String> {
        if opt_level == 0 {
            return Ok(());
        }
        
        unsafe {
            // Initialize targets for target-specific optimizations
            (self.api.init_target_infos)();
            (self.api.init_targets)();
            (self.api.init_target_mcs)();
            (self.api.init_asm_printers)();
            (self.api.init_asm_parsers)();
            if let Some(f) = self.api.init_aarch64_target_infos.as_ref() { f() }
            if let Some(f) = self.api.init_aarch64_targets.as_ref() { f() }
            if let Some(f) = self.api.init_aarch64_target_mcs.as_ref() { f() }
            if let Some(f) = self.api.init_aarch64_asm_printers.as_ref() { f() }
            if let Some(f) = self.api.init_aarch64_asm_parsers.as_ref() { f() }
            if let Some(f) = self.api.init_riscv_target_infos.as_ref() { f() }
            if let Some(f) = self.api.init_riscv_targets.as_ref() { f() }
            if let Some(f) = self.api.init_riscv_target_mcs.as_ref() { f() }
            if let Some(f) = self.api.init_riscv_asm_printers.as_ref() { f() }
            if let Some(f) = self.api.init_riscv_asm_parsers.as_ref() { f() }
            if let Some(f) = self.api.init_avr_target_infos.as_ref() { f() }
            if let Some(f) = self.api.init_avr_targets.as_ref() { f() }
            if let Some(f) = self.api.init_avr_target_mcs.as_ref() { f() }
            if let Some(f) = self.api.init_avr_asm_printers.as_ref() { f() }
            if let Some(f) = self.api.init_avr_asm_parsers.as_ref() { f() }
            let triple_ptr = (self.api.get_default_triple)();
            let mut target: LLVMTargetRef = ptr::null_mut();
            let mut error: *mut c_char = ptr::null_mut();
            
            let failed = (self.api.get_target_from_triple)(triple_ptr, &mut target, &mut error);
            if failed != 0 {
                let msg = CStr::from_ptr(error).to_string_lossy().into_owned();
                return Err(format!("Failed to get target: {}", msg));
            }
            
            let cpu = CString::new("generic").unwrap();
            let features = CString::new("").unwrap();
            
            let target_machine = (self.api.create_target_machine)(
                target,
                triple_ptr,
                cpu.as_ptr(),
                features.as_ptr(),
                opt_level as u32, // OptLevel
                0, // Reloc default
                0  // CodeModel default
            );
            
            // Build pass pipeline based on optimization level
            let passes = match opt_level {
                1 => "mem2reg,simplifycfg,instcombine,dce",
                2 => "default<O2>",
                _ => "default<O3>",
            };
            let passes_cstr = CString::new(passes).unwrap();
            
            // Create pass builder options
            let options = (self.api.create_pass_builder_options)();
            
            // Run the optimization passes
            let error = (self.api.run_passes)(
                self.module,
                passes_cstr.as_ptr(),
                target_machine,
                options
            );
            
            // Clean up options
            (self.api.dispose_pass_builder_options)(options);
            
            // Check for errors
            if !error.is_null() {
                let msg = (self.api.get_error_message)(error);
                let error_str = CStr::from_ptr(msg).to_string_lossy().into_owned();
                (self.api.consume_error)(error);
                (self.api.dispose_message)(msg);
                return Err(format!("Optimization failed: {}", error_str));
            }
        }
        
        Ok(())
    }
    
    /// Execute the compiled module using LLVM MCJIT (no object files, no clang)
    /// Returns the exit code from main()
    pub fn execute_jit(&mut self) -> Result<i32, String> {
        unsafe {
            let module = self.module;
            if module.is_null() {
                return Err("LLVM JIT error: module not initialized".to_string());
            }

            // Initialize targets for JIT
            (self.api.init_target_infos)();
            (self.api.init_targets)();
            (self.api.init_target_mcs)();
            (self.api.init_asm_printers)();
            (self.api.init_asm_parsers)();
            if let Some(f) = self.api.init_aarch64_target_infos.as_ref() { f() }
            if let Some(f) = self.api.init_aarch64_targets.as_ref() { f() }
            if let Some(f) = self.api.init_aarch64_target_mcs.as_ref() { f() }
            if let Some(f) = self.api.init_aarch64_asm_printers.as_ref() { f() }
            if let Some(f) = self.api.init_aarch64_asm_parsers.as_ref() { f() }
            if let Some(f) = self.api.init_riscv_target_infos.as_ref() { f() }
            if let Some(f) = self.api.init_riscv_targets.as_ref() { f() }
            if let Some(f) = self.api.init_riscv_target_mcs.as_ref() { f() }
            if let Some(f) = self.api.init_riscv_asm_printers.as_ref() { f() }
            if let Some(f) = self.api.init_riscv_asm_parsers.as_ref() { f() }
            if let Some(f) = self.api.init_avr_target_infos.as_ref() { f() }
            if let Some(f) = self.api.init_avr_targets.as_ref() { f() }
            if let Some(f) = self.api.init_avr_target_mcs.as_ref() { f() }
            if let Some(f) = self.api.init_avr_asm_printers.as_ref() { f() }
            if let Some(f) = self.api.init_avr_asm_parsers.as_ref() { f() }
            (self.api.link_in_mcjit)();
            
            // Create execution engine
            let mut ee: LLVMExecutionEngineRef = ptr::null_mut();
            let mut error: *mut c_char = ptr::null_mut();
            
            // MCJITCompilerOptions - pass null for defaults
            let failed = (self.api.create_mcjit_compiler)(
                &mut ee,
                module,
                ptr::null_mut(), // options
                0,               // options size
                &mut error
            );
            
            if failed != 0 || ee.is_null() {
                let msg = if !error.is_null() {
                    let s = CStr::from_ptr(error).to_string_lossy().into_owned();
                    (self.api.dispose_message)(error);
                    s
                } else {
                    "Unknown error".to_string()
                };
                return Err(format!("Failed to create JIT: {}", msg));
            }
            self.execution_engine = ee;

            // Ownership of `module` is now inside the execution engine.
            // Ensure shutdown() never disposes it.
            self.module = ptr::null_mut();

            // Map runtime functions to JIT execution engine
            if let Err(err) = self.map_runtime_functions(ee, module) {
                (self.api.dispose_execution_engine)(ee);
                self.execution_engine = ptr::null_mut();
                return Err(err);
            }

            // Resolve main function directly in the module.
            let main_name = CString::new("main").unwrap();
            let main_func = (self.api.get_named_function)(module, main_name.as_ptr());

            if main_func.is_null() {
                (self.api.dispose_execution_engine)(ee);
                self.execution_engine = ptr::null_mut();
                return Err("main function not found".to_string());
            }

            let ret_ty = (self.api.get_return_type)((self.api.type_of)(main_func));
            let ret_kind = (self.api.get_type_kind)(ret_ty);
            let main_addr = (self.api.get_function_address)(ee, main_name.as_ptr());
            if main_addr == 0 {
                (self.api.dispose_execution_engine)(ee);
                self.execution_engine = ptr::null_mut();
                return Err("main function address not found".to_string());
            }

            // Match call ABI with LLVM main return type to avoid UB.
            let result = if ret_kind == LLVMTypeKind::LLVMVoidTypeKind {
                let main_fn: extern "C" fn() = std::mem::transmute(main_addr);
                main_fn();
                0
            } else {
                let main_fn: extern "C" fn() -> i64 = std::mem::transmute(main_addr);
                main_fn()
            };

            // NOTE: On Windows, disposing MCJIT here intermittently triggers an
            // access violation at process exit. This CLI process is short-lived,
            // so we intentionally skip explicit disposal on the success path.
            Ok(result as i32)
        }
    }
    
    /// Map runtime functions to the JIT execution engine
    unsafe fn map_runtime_functions(&self, ee: LLVMExecutionEngineRef, module: LLVMModuleRef) -> Result<(), String> {
        // falcon_print_int -> jit_print_int
        let name = CString::new("falcon_print_int").unwrap();
        let func = (self.api.get_named_function)(module, name.as_ptr());
        if !func.is_null() {
            (self.api.add_global_mapping)(ee, func, jit_print_int as *mut c_void);
        }
        
        // falcon_print_i32 -> jit_print_i32
        let name = CString::new("falcon_print_i32").unwrap();
        let func = (self.api.get_named_function)(module, name.as_ptr());
        if !func.is_null() {
            (self.api.add_global_mapping)(ee, func, jit_print_i32 as *mut c_void);
        }
        
        // falcon_println -> jit_println
        let name = CString::new("falcon_println").unwrap();
        let func = (self.api.get_named_function)(module, name.as_ptr());
        if !func.is_null() {
            (self.api.add_global_mapping)(ee, func, jit_println as *mut c_void);
        }
        
        // falcon_print -> jit_print
        let name = CString::new("falcon_print").unwrap();
        let func = (self.api.get_named_function)(module, name.as_ptr());
        if !func.is_null() {
            (self.api.add_global_mapping)(ee, func, jit_print as *mut c_void);
        }

        // falcon_bounds_check -> jit_bounds_check
        let name = CString::new("falcon_bounds_check").unwrap();
        let func = (self.api.get_named_function)(module, name.as_ptr());
        if !func.is_null() {
            (self.api.add_global_mapping)(ee, func, jit_bounds_check as *mut c_void);
        }
        
        Ok(())
    }

    unsafe fn declare_function(&mut self, func: &IrFunction) -> Result<LLVMValueRef, String> {
        let mut param_types: Vec<LLVMTypeRef> = Vec::new();
        for param in &func.params {
            param_types.push(self.ir_type_to_llvm(&param.ty)?);
        }
        
        // Define return type
        // main() always returns i64 (C convention), even if user didn't declare it
        let ret_type = if func.name == "main" {
            (self.api.int64_type)(self.context)
        } else {
            match &func.return_type {
                Some(ty) => self.ir_type_to_llvm(ty)?,
                None => (self.api.void_type)(self.context),
            }
        };
        
        let function_type = (self.api.function_type)(
            ret_type, 
            param_types.as_mut_ptr(), 
            param_types.len() as u32, 
            0 // IsVarArg = false
        );
        
        let c_name = CString::new(func.name.as_str()).map_err(|e| e.to_string())?;
        let function = (self.api.add_function)(self.module, c_name.as_ptr(), function_type);
        
        self.function_types.insert(func.name.clone(), function_type);
        
        Ok(function)
    }

    unsafe fn declare_runtime_functions(&mut self) -> Result<(), String> {
        let void_type = (self.api.void_type)(self.context);
        let i64_type = (self.api.int64_type)(self.context);
        let f64_type = (self.api.double_type)(self.context);
        let ptr_type = (self.api.ptr_type)(self.context, 0);
        
        // void falcon_print_int(int64_t)
        let mut print_int_args = [i64_type];
        let print_int_type = (self.api.function_type)(void_type, print_int_args.as_mut_ptr(), 1, 0);
        let print_int_name = CString::new("falcon_print_int").unwrap();
        (self.api.add_function)(self.module, print_int_name.as_ptr(), print_int_type);
        self.function_types.insert("falcon_print_int".to_string(), print_int_type);
        
        // void falcon_println(i8*)
        let mut println_args = [ptr_type];
        let println_type = (self.api.function_type)(void_type, println_args.as_mut_ptr(), 1, 0);
        let println_name = CString::new("falcon_println").unwrap();
        (self.api.add_function)(self.module, println_name.as_ptr(), println_type);
        self.function_types.insert("falcon_println".to_string(), println_type);

        // void falcon_print(i8*)
        let mut print_args = [ptr_type];
        let print_type = (self.api.function_type)(void_type, print_args.as_mut_ptr(), 1, 0);
        let print_name = CString::new("falcon_print").unwrap();
        (self.api.add_function)(self.module, print_name.as_ptr(), print_type);
        self.function_types.insert("falcon_print".to_string(), print_type);

        // void falcon_print_float(double)
        let mut print_float_args = [f64_type];
        let print_float_type =
            (self.api.function_type)(void_type, print_float_args.as_mut_ptr(), 1, 0);
        let print_float_name = CString::new("falcon_print_float").unwrap();
        (self.api.add_function)(self.module, print_float_name.as_ptr(), print_float_type);
        self.function_types
            .insert("falcon_print_float".to_string(), print_float_type);
        self.function_types
            .insert("print_float".to_string(), print_float_type);

        // void falcon_print_bool(i1)
        let bool_type = (self.api.int1_type)(self.context);
        let mut print_bool_args = [bool_type];
        let print_bool_type =
            (self.api.function_type)(void_type, print_bool_args.as_mut_ptr(), 1, 0);
        let print_bool_name = CString::new("falcon_print_bool").unwrap();
        (self.api.add_function)(self.module, print_bool_name.as_ptr(), print_bool_type);
        self.function_types
            .insert("falcon_print_bool".to_string(), print_bool_type);
        self.function_types
            .insert("print_bool".to_string(), print_bool_type);

        // Aliases for print_int/print_i32 -> falcon_print_int
        let i32_type = (self.api.int32_type)(self.context);
        let mut print_i32_args = [i32_type];
        let print_i32_type = (self.api.function_type)(void_type, print_i32_args.as_mut_ptr(), 1, 0);
        let print_i32_name = CString::new("falcon_print_i32").unwrap();
        (self.api.add_function)(self.module, print_i32_name.as_ptr(), print_i32_type);
        self.function_types.insert("falcon_print_i32".to_string(), print_i32_type);
        self.function_types.insert("print_i32".to_string(), print_i32_type);
        self.function_types.insert("print_int".to_string(), print_int_type);

        // void falcon_print_newline(void)
        let print_nl_type = (self.api.function_type)(void_type, ptr::null_mut(), 0, 0);
        let print_nl_name = CString::new("falcon_print_newline").unwrap();
        (self.api.add_function)(self.module, print_nl_name.as_ptr(), print_nl_type);
        self.function_types.insert("falcon_print_newline".to_string(), print_nl_type);
        self.function_types.insert("print_newline".to_string(), print_nl_type);

        // i8* falcon_input(i8*) - read line from stdin
        let mut input_args = [ptr_type];
        let input_type = (self.api.function_type)(ptr_type, input_args.as_mut_ptr(), 1, 0);
        let input_name = CString::new("falcon_input").unwrap();
        (self.api.add_function)(self.module, input_name.as_ptr(), input_type);
        self.function_types.insert("falcon_input".to_string(), input_type);
        self.function_types.insert("input".to_string(), input_type);

        // i8* falcon_str_concat(i8*, i8*)
        let mut str_concat_args = [ptr_type, ptr_type];
        let str_concat_type =
            (self.api.function_type)(ptr_type, str_concat_args.as_mut_ptr(), 2, 0);
        let str_concat_name = CString::new("falcon_str_concat").unwrap();
        (self.api.add_function)(self.module, str_concat_name.as_ptr(), str_concat_type);
        self.function_types
            .insert("falcon_str_concat".to_string(), str_concat_type);
        self.function_types
            .insert("str_concat".to_string(), str_concat_type);

        // i1 falcon_str_eq(i8*, i8*)
        let mut str_eq_args = [ptr_type, ptr_type];
        let str_eq_type = (self.api.function_type)(bool_type, str_eq_args.as_mut_ptr(), 2, 0);
        let str_eq_name = CString::new("falcon_str_eq").unwrap();
        (self.api.add_function)(self.module, str_eq_name.as_ptr(), str_eq_type);
        self.function_types.insert("falcon_str_eq".to_string(), str_eq_type);
        self.function_types.insert("str_eq".to_string(), str_eq_type);

        // i1 falcon_str_is_empty(i8*)
        let mut str_empty_args = [ptr_type];
        let str_empty_type =
            (self.api.function_type)(bool_type, str_empty_args.as_mut_ptr(), 1, 0);
        let str_empty_name = CString::new("falcon_str_is_empty").unwrap();
        (self.api.add_function)(self.module, str_empty_name.as_ptr(), str_empty_type);
        self.function_types
            .insert("falcon_str_is_empty".to_string(), str_empty_type);
        self.function_types
            .insert("str_is_empty".to_string(), str_empty_type);

        // i64 falcon_str_len(i8*)
        let mut str_len_args = [ptr_type];
        let str_len_type = (self.api.function_type)(i64_type, str_len_args.as_mut_ptr(), 1, 0);
        let str_len_name = CString::new("falcon_str_len").unwrap();
        (self.api.add_function)(self.module, str_len_name.as_ptr(), str_len_type);
        self.function_types
            .insert("falcon_str_len".to_string(), str_len_type);
        self.function_types.insert("str_len".to_string(), str_len_type);

        // i64 falcon_str_find_from(i8*, i8*, i64)
        let mut str_find_args = [ptr_type, ptr_type, i64_type];
        let str_find_type = (self.api.function_type)(i64_type, str_find_args.as_mut_ptr(), 3, 0);
        let str_find_name = CString::new("falcon_str_find_from").unwrap();
        (self.api.add_function)(self.module, str_find_name.as_ptr(), str_find_type);
        self.function_types
            .insert("falcon_str_find_from".to_string(), str_find_type);
        self.function_types
            .insert("str_find_from".to_string(), str_find_type);

        // i64 falcon_strlen(i8*) - get string length
        let mut strlen_args = [ptr_type];
        let strlen_type = (self.api.function_type)(i64_type, strlen_args.as_mut_ptr(), 1, 0);
        let strlen_name = CString::new("falcon_strlen").unwrap();
        (self.api.add_function)(self.module, strlen_name.as_ptr(), strlen_type);
        self.function_types.insert("falcon_strlen".to_string(), strlen_type);
        self.function_types.insert("strlen".to_string(), strlen_type);

        // i64 falcon_str_contains(i8*, i8*) - check if string contains substring
        let mut str_contains_args = [ptr_type, ptr_type];
        let str_contains_type = (self.api.function_type)(i64_type, str_contains_args.as_mut_ptr(), 2, 0);
        let str_contains_name = CString::new("falcon_str_contains").unwrap();
        (self.api.add_function)(self.module, str_contains_name.as_ptr(), str_contains_type);
        self.function_types.insert("falcon_str_contains".to_string(), str_contains_type);
        self.function_types.insert("str_contains".to_string(), str_contains_type);

        // i64 falcon_str_equals(i8*, i8*) - check string equality
        let mut str_equals_args = [ptr_type, ptr_type];
        let str_equals_type = (self.api.function_type)(i64_type, str_equals_args.as_mut_ptr(), 2, 0);
        let str_equals_name = CString::new("falcon_str_equals").unwrap();
        (self.api.add_function)(self.module, str_equals_name.as_ptr(), str_equals_type);
        self.function_types.insert("falcon_str_equals".to_string(), str_equals_type);
        self.function_types.insert("str_equals".to_string(), str_equals_type);

        // i64 falcon_str_char_at(i8*, i64) - get char at index
        let mut str_char_at_args = [ptr_type, i64_type];
        let str_char_at_type = (self.api.function_type)(i64_type, str_char_at_args.as_mut_ptr(), 2, 0);
        let str_char_at_name = CString::new("falcon_str_char_at").unwrap();
        (self.api.add_function)(self.module, str_char_at_name.as_ptr(), str_char_at_type);
        self.function_types.insert("falcon_str_char_at".to_string(), str_char_at_type);
        self.function_types.insert("str_char_at".to_string(), str_char_at_type);

        // i8* falcon_str_slice(i8*, i64, i64) - get substring by range
        let mut str_slice_args = [ptr_type, i64_type, i64_type];
        let str_slice_type = (self.api.function_type)(ptr_type, str_slice_args.as_mut_ptr(), 3, 0);
        let str_slice_name = CString::new("falcon_str_slice").unwrap();
        (self.api.add_function)(self.module, str_slice_name.as_ptr(), str_slice_type);
        self.function_types.insert("falcon_str_slice".to_string(), str_slice_type);
        self.function_types.insert("str_slice".to_string(), str_slice_type);

        // i8* falcon_str_substr(i8*, i64, i64)
        let mut str_substr_args = [ptr_type, i64_type, i64_type];
        let str_substr_type =
            (self.api.function_type)(ptr_type, str_substr_args.as_mut_ptr(), 3, 0);
        let str_substr_name = CString::new("falcon_str_substr").unwrap();
        (self.api.add_function)(self.module, str_substr_name.as_ptr(), str_substr_type);
        self.function_types
            .insert("falcon_str_substr".to_string(), str_substr_type);
        self.function_types
            .insert("str_substr".to_string(), str_substr_type);

        // i8* falcon_str_replace_all(i8*, i8*, i8*)
        let mut str_replace_args = [ptr_type, ptr_type, ptr_type];
        let str_replace_type =
            (self.api.function_type)(ptr_type, str_replace_args.as_mut_ptr(), 3, 0);
        let str_replace_name = CString::new("falcon_str_replace_all").unwrap();
        (self.api.add_function)(self.module, str_replace_name.as_ptr(), str_replace_type);
        self.function_types
            .insert("falcon_str_replace_all".to_string(), str_replace_type);
        self.function_types
            .insert("str_replace_all".to_string(), str_replace_type);

        // i8* falcon_str_strip_html_tags(i8*)
        let mut str_strip_args = [ptr_type];
        let str_strip_type =
            (self.api.function_type)(ptr_type, str_strip_args.as_mut_ptr(), 1, 0);
        let str_strip_name = CString::new("falcon_str_strip_html_tags").unwrap();
        (self.api.add_function)(self.module, str_strip_name.as_ptr(), str_strip_type);
        self.function_types
            .insert("falcon_str_strip_html_tags".to_string(), str_strip_type);
        self.function_types
            .insert("str_strip_html_tags".to_string(), str_strip_type);

        // i8* falcon_str_json_extract_values(i8*, i8*, i64)
        let mut str_json_extract_args = [ptr_type, ptr_type, i64_type];
        let str_json_extract_type =
            (self.api.function_type)(ptr_type, str_json_extract_args.as_mut_ptr(), 3, 0);
        let str_json_extract_name = CString::new("falcon_str_json_extract_values").unwrap();
        (self.api.add_function)(
            self.module,
            str_json_extract_name.as_ptr(),
            str_json_extract_type,
        );
        self.function_types.insert(
            "falcon_str_json_extract_values".to_string(),
            str_json_extract_type,
        );
        self.function_types
            .insert("str_json_extract_values".to_string(), str_json_extract_type);

        // i64 falcon_abs(i64)
        let mut abs_args = [i64_type];
        let abs_type = (self.api.function_type)(i64_type, abs_args.as_mut_ptr(), 1, 0);
        let abs_name = CString::new("falcon_abs").unwrap();
        (self.api.add_function)(self.module, abs_name.as_ptr(), abs_type);
        self.function_types.insert("falcon_abs".to_string(), abs_type);
        self.function_types.insert("abs".to_string(), abs_type);

        // i64 falcon_min(i64, i64)
        let mut min_args = [i64_type, i64_type];
        let min_type = (self.api.function_type)(i64_type, min_args.as_mut_ptr(), 2, 0);
        let min_name = CString::new("falcon_min").unwrap();
        (self.api.add_function)(self.module, min_name.as_ptr(), min_type);
        self.function_types.insert("falcon_min".to_string(), min_type);
        self.function_types.insert("min".to_string(), min_type);

        // i64 falcon_max(i64, i64)
        let mut max_args = [i64_type, i64_type];
        let max_type = (self.api.function_type)(i64_type, max_args.as_mut_ptr(), 2, 0);
        let max_name = CString::new("falcon_max").unwrap();
        (self.api.add_function)(self.module, max_name.as_ptr(), max_type);
        self.function_types.insert("falcon_max".to_string(), max_type);
        self.function_types.insert("max".to_string(), max_type);

        // void falcon_bounds_check(i64 index, i64 length) - userland bounds checks
        let mut bounds_check_args = [i64_type, i64_type];
        let bounds_check_type = (self.api.function_type)(void_type, bounds_check_args.as_mut_ptr(), 2, 0);
        let bounds_check_name = CString::new("falcon_bounds_check").unwrap();
        (self.api.add_function)(self.module, bounds_check_name.as_ptr(), bounds_check_type);
        self.function_types.insert("falcon_bounds_check".to_string(), bounds_check_type);

        // i64 falcon_array_len(ptr) - get array length for iterator protocol
        let mut array_len_args = [ptr_type];
        let array_len_type = (self.api.function_type)(i64_type, array_len_args.as_mut_ptr(), 1, 0);
        let array_len_name = CString::new("falcon_array_len").unwrap();
        (self.api.add_function)(self.module, array_len_name.as_ptr(), array_len_type);
        self.function_types.insert("falcon_array_len".to_string(), array_len_type);

        // void falcon_ollama_generate(i8*, i8*) - LLM text generation via Ollama
        let mut ollama_args = [ptr_type, ptr_type];
        let ollama_type = (self.api.function_type)(void_type, ollama_args.as_mut_ptr(), 2, 0);
        let ollama_name = CString::new("falcon_ollama_generate").unwrap();
        (self.api.add_function)(self.module, ollama_name.as_ptr(), ollama_type);
        self.function_types.insert("falcon_ollama_generate".to_string(), ollama_type);
        self.function_types.insert("ollama_generate".to_string(), ollama_type);

        // void falcon_ollama_chat(i8*, i8*) - interactive chat with personality
        let chat_name = CString::new("falcon_ollama_chat").unwrap();
        (self.api.add_function)(self.module, chat_name.as_ptr(), ollama_type);
        self.function_types.insert("falcon_ollama_chat".to_string(), ollama_type);
        self.function_types.insert("ollama_chat".to_string(), ollama_type);

        // i8* falcon_os_exec_capture(i8*) - thin process capture binding
        let mut exec_capture_args = [ptr_type];
        let exec_capture_type =
            (self.api.function_type)(ptr_type, exec_capture_args.as_mut_ptr(), 1, 0);
        let exec_capture_name = CString::new("falcon_os_exec_capture").unwrap();
        (self.api.add_function)(self.module, exec_capture_name.as_ptr(), exec_capture_type);
        self.function_types
            .insert("falcon_os_exec_capture".to_string(), exec_capture_type);

        // i64 falcon_os_exec_stream(i8*) - thin process stream binding
        let mut exec_stream_args = [ptr_type];
        let exec_stream_type =
            (self.api.function_type)(i64_type, exec_stream_args.as_mut_ptr(), 1, 0);
        let exec_stream_name = CString::new("falcon_os_exec_stream").unwrap();
        (self.api.add_function)(self.module, exec_stream_name.as_ptr(), exec_stream_type);
        self.function_types
            .insert("falcon_os_exec_stream".to_string(), exec_stream_type);

        // void falcon_ollama_list_models() - list local models via `ollama list`
        let ollama_list_type = (self.api.function_type)(void_type, ptr::null_mut(), 0, 0);
        let ollama_list_name = CString::new("falcon_ollama_list_models").unwrap();
        (self.api.add_function)(self.module, ollama_list_name.as_ptr(), ollama_list_type);
        self.function_types
            .insert("falcon_ollama_list_models".to_string(), ollama_list_type);
        self.function_types
            .insert("ollama_list_models".to_string(), ollama_list_type);

        // ============================================
        // Native C Math Library (libm)
        // Same functions Python uses - native speed!
        // ============================================
        let double_type = (self.api.double_type)(self.context);

        // double falcon_X(double) - single arg math functions
        let mut math1_args = [double_type];
        let math1_type = (self.api.function_type)(double_type, math1_args.as_mut_ptr(), 1, 0);
        
        for name in &["sin", "cos", "tan", "asin", "acos", "atan", 
                      "sqrt", "exp", "log", "log10", 
                      "floor", "ceil", "round"] {
            let full_name = format!("falcon_{}", name);
            let c_name = CString::new(full_name.clone()).unwrap();
            (self.api.add_function)(self.module, c_name.as_ptr(), math1_type);
            self.function_types.insert(full_name, math1_type);
            self.function_types.insert(name.to_string(), math1_type);
        }

        // double falcon_pow(double, double) - two arg
        let mut math2_args = [double_type, double_type];
        let math2_type = (self.api.function_type)(double_type, math2_args.as_mut_ptr(), 2, 0);
        let pow_name = CString::new("falcon_pow").unwrap();
        (self.api.add_function)(self.module, pow_name.as_ptr(), math2_type);
        self.function_types.insert("falcon_pow".to_string(), math2_type);
        self.function_types.insert("pow".to_string(), math2_type);

        // double falcon_pi() / falcon_e() - constants
        let const_type = (self.api.function_type)(double_type, ptr::null_mut(), 0, 0);
        let pi_name = CString::new("falcon_pi").unwrap();
        (self.api.add_function)(self.module, pi_name.as_ptr(), const_type);
        self.function_types.insert("falcon_pi".to_string(), const_type);
        self.function_types.insert("pi".to_string(), const_type);
        
        let e_name = CString::new("falcon_e").unwrap();
        (self.api.add_function)(self.module, e_name.as_ptr(), const_type);
        self.function_types.insert("falcon_e".to_string(), const_type);
        self.function_types.insert("e".to_string(), const_type);

        // ============================================
        // Python Random Module (Ported)
        // Same API as Python's random module!
        // ============================================

        // void falcon_random_seed(i64)
        let mut seed_args = [i64_type];
        let seed_type = (self.api.function_type)(void_type, seed_args.as_mut_ptr(), 1, 0);
        let seed_name = CString::new("falcon_random_seed").unwrap();
        (self.api.add_function)(self.module, seed_name.as_ptr(), seed_type);
        self.function_types.insert("falcon_random_seed".to_string(), seed_type);
        self.function_types.insert("random_seed".to_string(), seed_type);

        // double falcon_random()
        let random_type = (self.api.function_type)(double_type, ptr::null_mut(), 0, 0);
        let random_name = CString::new("falcon_random").unwrap();
        (self.api.add_function)(self.module, random_name.as_ptr(), random_type);
        self.function_types.insert("falcon_random".to_string(), random_type);
        self.function_types.insert("random".to_string(), random_type);

        // i64 falcon_randint(i64, i64)
        let mut randint_args = [i64_type, i64_type];
        let randint_type = (self.api.function_type)(i64_type, randint_args.as_mut_ptr(), 2, 0);
        let randint_name = CString::new("falcon_randint").unwrap();
        (self.api.add_function)(self.module, randint_name.as_ptr(), randint_type);
        self.function_types.insert("falcon_randint".to_string(), randint_type);
        self.function_types.insert("randint".to_string(), randint_type);

        // i64 falcon_randrange(i64)
        let mut randrange_args = [i64_type];
        let randrange_type = (self.api.function_type)(i64_type, randrange_args.as_mut_ptr(), 1, 0);
        let randrange_name = CString::new("falcon_randrange").unwrap();
        (self.api.add_function)(self.module, randrange_name.as_ptr(), randrange_type);
        self.function_types.insert("falcon_randrange".to_string(), randrange_type);
        self.function_types.insert("randrange".to_string(), randrange_type);

        // i64 falcon_time_seed()
        let time_seed_type = (self.api.function_type)(i64_type, ptr::null_mut(), 0, 0);
        let time_seed_name = CString::new("falcon_time_seed").unwrap();
        (self.api.add_function)(self.module, time_seed_name.as_ptr(), time_seed_type);
        self.function_types.insert("falcon_time_seed".to_string(), time_seed_type);
        self.function_types.insert("time_seed".to_string(), time_seed_type);

        Ok(())
    }

    unsafe fn compile_struct_type(&mut self, def: &crate::ir::IrStructDef) -> Result<(), String> {
        let mut field_types: Vec<LLVMTypeRef> = Vec::new();
        let mut field_indices: HashMap<String, u32> = HashMap::new();
        let mut field_type_map: HashMap<String, LLVMTypeRef> = HashMap::new();
        
        for (i, (name, ty)) in def.fields.iter().enumerate() {
            let llvm_ty = self.ir_type_to_llvm(ty)?;
            field_types.push(llvm_ty);
            field_indices.insert(name.clone(), i as u32);
            field_type_map.insert(name.clone(), llvm_ty);
        }
        
        let struct_ty = (self.api.struct_type)(
            self.context,
            field_types.as_mut_ptr(),
            field_types.len() as u32,
            0 // not packed
        );
        
        self.struct_types.insert(def.name.clone(), struct_ty);
        self.struct_field_indices.insert(def.name.clone(), field_indices);
        self.struct_field_types.insert(def.name.clone(), field_type_map);
        Ok(())
    }

    /// Get or create the cached enum type {i32 tag, i64 payload}
    unsafe fn get_enum_type(&mut self) -> LLVMTypeRef {
        if let Some(ty) = self.enum_type {
            ty
        } else {
            let i32_type = (self.api.int32_type)(self.context);
            let i64_type = (self.api.int64_type)(self.context);
            let mut field_types = [i32_type, i64_type];
            let ty = (self.api.struct_type)(
                self.context,
                field_types.as_mut_ptr(),
                2,
                0 // not packed
            );
            self.enum_type = Some(ty);
            ty
        }
    }

    /// Get or create the cached range type {i64 start, i64 end}
    unsafe fn get_range_type(&mut self) -> LLVMTypeRef {
        if let Some(ty) = self.range_type {
            ty
        } else {
            let i64_type = (self.api.int64_type)(self.context);
            let mut field_types = [i64_type, i64_type];
            let ty = (self.api.struct_type)(
                self.context,
                field_types.as_mut_ptr(),
                2,
                0 // not packed
            );
            self.range_type = Some(ty);
            ty
        }
    }

    /// Look up the tag value for an enum variant
    fn get_variant_tag(&self, enum_name: &str, variant_name: &str) -> Option<u32> {
        self.enum_defs.get(enum_name).and_then(|variants| {
            variants.iter()
                .find(|(name, _)| name == variant_name)
                .map(|(_, tag)| *tag)
        })
    }

    unsafe fn compile_function(&mut self, func: &IrFunction) -> Result<(), String> {
        let c_name = CString::new(func.name.as_str()).unwrap();
        let function = (self.api.get_named_function)(self.module, c_name.as_ptr());
        
        if function.is_null() {
            return Err(format!("Function {} not found", func.name));
        }
        
        self.current_function = Some(function);
        self.variables.clear();
        self.variable_types.clear();
        
        // Set return type for this function
        self.current_return_type = match &func.return_type {
            Some(ty) => Some(self.ir_type_to_llvm(ty)?),
            None => None,
        };
        
        // Create entry block
        let entry_name = CString::new("entry").unwrap();
        let entry_block = (self.api.append_basic_block)(self.context, function, entry_name.as_ptr());
        self.entry_block = Some(entry_block);
        (self.api.position_builder)(self.builder, entry_block);
        
        // Handle parameters
        for (i, param) in func.params.iter().enumerate() {
            let llvm_param = (self.api.get_param)(function, i as u32);
            let param_name_c = CString::new(param.name.as_str()).unwrap();
            (self.api.set_value_name2)(llvm_param, param_name_c.as_ptr(), param.name.len());
            
            // Allocate stack space for param
            let param_type = self.ir_type_to_llvm(&param.ty)?;
            let alloca = (self.api.build_alloca)(self.builder, param_type, param_name_c.as_ptr());
            (self.api.build_store)(self.builder, llvm_param, alloca);
            
            self.variables.insert(param.name.clone(), alloca);
            self.variable_types.insert(param.name.clone(), param_type);
        }
        
        // Compile instructions
        for inst in &func.body.instructions {
            self.compile_instruction(inst)?;
        }
        
        // Implicit return — ensure every function terminates properly
        let current_block = (self.api.get_insert_block)(self.builder);
        if (self.api.get_basic_block_terminator)(current_block).is_null() {
            if func.name == "main" {
                // main() always returns i64 0 (success exit code)
                let i64_type = (self.api.int64_type)(self.context);
                let zero = (self.api.const_int)(i64_type, 0, 0);
                (self.api.build_ret)(self.builder, zero);
            } else if let Some(ref ret_ty) = func.return_type {
                // Function has return type but no explicit return — return zero/null
                let llvm_ty = self.ir_type_to_llvm(ret_ty)?;
                let zero = (self.api.const_int)(llvm_ty, 0, 0);
                (self.api.build_ret)(self.builder, zero);
            } else {
                // Void function — ret void
                (self.api.build_ret_void)(self.builder);
            }
        }
        
        Ok(())
    }
    
    // Stub for compile_instruction and helpers
    unsafe fn compile_instruction(&mut self, inst: &IrInstruction) -> Result<(), String> {
        // Helper to convert Rust string to CString for LLVM naming
        let c_str = |s: &str| CString::new(s).unwrap();
        
        match inst {
            IrInstruction::Literal { value, dest } => {
                let val = self.compile_literal(value)?;
                self.store_value(dest, val)?;
            }
            
            IrInstruction::TypeHint { name, ty } => {
                // Pre-create alloca with the declared type so subsequent store uses it
                let llvm_ty = self.ir_type_to_llvm(ty)?;
                let c_name = CString::new(name.as_str()).unwrap();
                
                if !self.variables.contains_key(name) {
                    // Create alloca at entry block with the hinted type
                    let current_block = (self.api.get_insert_block)(self.builder);
                    if let Some(entry) = self.entry_block {
                        let first_inst = (self.api.get_first_instruction)(entry);
                        if !first_inst.is_null() {
                            (self.api.position_builder_before)(self.builder, first_inst);
                        } else {
                            (self.api.position_builder)(self.builder, entry);
                        }
                    }
                    let alloca = (self.api.build_alloca)(self.builder, llvm_ty, c_name.as_ptr());
                    self.variables.insert(name.clone(), alloca);
                    self.variable_types.insert(name.clone(), llvm_ty);
                    (self.api.position_builder)(self.builder, current_block);
                }
            }
            
            IrInstruction::Move { src, dest } => {
                // Check if src holds a closure, and propagate mapping to dest variable.
                let src_name = src.name();
                if let Some(closure_fn) = self
                    .temp_to_closure
                    .get(&src_name)
                    .cloned()
                    .or_else(|| self.closure_functions.get(&src_name).cloned())
                {
                    let dest_name = dest.name();
                    self.closure_functions.insert(dest_name.clone(), closure_fn.clone());
                    self.temp_to_closure.insert(dest_name, closure_fn);
                    // Closure values are represented via mapping, not numeric loads/stores.
                    return Ok(());
                }

                // Regular move: missing values are deterministic compile errors.
                let val = self.load_value(src)?;
                self.store_value(dest, val)?;
            }
            
            IrInstruction::Add { left, right, dest } => {
                let lhs = self.load_value(left)?;
                let rhs = self.load_value(right)?;
                let (lhs, rhs) = self.coerce_int_types(lhs, rhs);
                let kind = (self.api.get_type_kind)((self.api.type_of)(lhs));
                
                let val = if kind == LLVMTypeKind::LLVMIntegerTypeKind {
                    (self.api.build_add)(self.builder, lhs, rhs, c_str("add").as_ptr())
                } else {
                    (self.api.build_fadd)(self.builder, lhs, rhs, c_str("fadd").as_ptr())
                };
                self.store_value(dest, val)?;
            }
            
            IrInstruction::Sub { left, right, dest } => {
                let lhs = self.load_value(left)?;
                let rhs = self.load_value(right)?;
                let (lhs, rhs) = self.coerce_int_types(lhs, rhs);
                let kind = (self.api.get_type_kind)((self.api.type_of)(lhs));
                
                let val = if kind == LLVMTypeKind::LLVMIntegerTypeKind {
                    (self.api.build_sub)(self.builder, lhs, rhs, c_str("sub").as_ptr())
                } else {
                    (self.api.build_fsub)(self.builder, lhs, rhs, c_str("fsub").as_ptr())
                };
                self.store_value(dest, val)?;
            }
            
            IrInstruction::Mul { left, right, dest } => {
                let lhs = self.load_value(left)?;
                let rhs = self.load_value(right)?;
                let (lhs, rhs) = self.coerce_int_types(lhs, rhs);
                let kind = (self.api.get_type_kind)((self.api.type_of)(lhs));
                
                let val = if kind == LLVMTypeKind::LLVMIntegerTypeKind {
                    (self.api.build_mul)(self.builder, lhs, rhs, c_str("mul").as_ptr())
                } else {
                    (self.api.build_fmul)(self.builder, lhs, rhs, c_str("fmul").as_ptr())
                };
                self.store_value(dest, val)?;
            }
            
            IrInstruction::Div { left, right, dest } => {
                let lhs = self.load_value(left)?;
                let rhs = self.load_value(right)?;
                let (lhs, rhs) = self.coerce_int_types(lhs, rhs);
                let kind = (self.api.get_type_kind)((self.api.type_of)(lhs));
                
                let val = if kind == LLVMTypeKind::LLVMIntegerTypeKind {
                    (self.api.build_sdiv)(self.builder, lhs, rhs, c_str("div").as_ptr())
                } else {
                    (self.api.build_fdiv)(self.builder, lhs, rhs, c_str("fdiv").as_ptr())
                };
                self.store_value(dest, val)?;
            }
            
            IrInstruction::Eq { left, right, dest } => {
                let lhs = self.load_value(left)?;
                let rhs = self.load_value(right)?;
                let (lhs, rhs) = self.coerce_int_types(lhs, rhs);
                let kind = (self.api.get_type_kind)((self.api.type_of)(lhs));
                
                let val = if kind == LLVMTypeKind::LLVMIntegerTypeKind {
                    (self.api.build_icmp)(self.builder, LLVMIntPredicate::LLVMIntEQ, lhs, rhs, c_str("eq").as_ptr())
                } else {
                    (self.api.build_fcmp)(self.builder, LLVMRealPredicate::LLVMRealOEQ, lhs, rhs, c_str("feq").as_ptr())
                };
                self.store_value(dest, val)?;
            }
            
            IrInstruction::Lt { left, right, dest } => {
                let lhs = self.load_value(left)?;
                let rhs = self.load_value(right)?;
                let (lhs, rhs) = self.coerce_int_types(lhs, rhs);
                let kind = (self.api.get_type_kind)((self.api.type_of)(lhs));
                
                let val = if kind == LLVMTypeKind::LLVMIntegerTypeKind {
                    (self.api.build_icmp)(self.builder, LLVMIntPredicate::LLVMIntSLT, lhs, rhs, c_str("lt").as_ptr())
                } else {
                    (self.api.build_fcmp)(self.builder, LLVMRealPredicate::LLVMRealOLT, lhs, rhs, c_str("flt").as_ptr())
                };
                self.store_value(dest, val)?;
            }

            IrInstruction::Gt { left, right, dest } => {
                let lhs = self.load_value(left)?;
                let rhs = self.load_value(right)?;
                let (lhs, rhs) = self.coerce_int_types(lhs, rhs);
                let kind = (self.api.get_type_kind)((self.api.type_of)(lhs));
                
                let val = if kind == LLVMTypeKind::LLVMIntegerTypeKind {
                    (self.api.build_icmp)(self.builder, LLVMIntPredicate::LLVMIntSGT, lhs, rhs, c_str("gt").as_ptr())
                } else {
                    (self.api.build_fcmp)(self.builder, LLVMRealPredicate::LLVMRealOGT, lhs, rhs, c_str("fgt").as_ptr())
                };
                self.store_value(dest, val)?;
            }
            
            IrInstruction::Le { left, right, dest } => {
                let lhs = self.load_value(left)?;
                let rhs = self.load_value(right)?;
                let (lhs, rhs) = self.coerce_int_types(lhs, rhs);
                let kind = (self.api.get_type_kind)((self.api.type_of)(lhs));
                
                let val = if kind == LLVMTypeKind::LLVMIntegerTypeKind {
                    (self.api.build_icmp)(self.builder, LLVMIntPredicate::LLVMIntSLE, lhs, rhs, c_str("le").as_ptr())
                } else {
                    (self.api.build_fcmp)(self.builder, LLVMRealPredicate::LLVMRealOLE, lhs, rhs, c_str("fle").as_ptr())
                };
                self.store_value(dest, val)?;
            }
            
            IrInstruction::Ge { left, right, dest } => {
                let lhs = self.load_value(left)?;
                let rhs = self.load_value(right)?;
                let (lhs, rhs) = self.coerce_int_types(lhs, rhs);
                let kind = (self.api.get_type_kind)((self.api.type_of)(lhs));
                
                let val = if kind == LLVMTypeKind::LLVMIntegerTypeKind {
                    (self.api.build_icmp)(self.builder, LLVMIntPredicate::LLVMIntSGE, lhs, rhs, c_str("ge").as_ptr())
                } else {
                    (self.api.build_fcmp)(self.builder, LLVMRealPredicate::LLVMRealOGE, lhs, rhs, c_str("fge").as_ptr())
                };
                self.store_value(dest, val)?;
            }
            
            IrInstruction::Ne { left, right, dest } => {
                let lhs = self.load_value(left)?;
                let rhs = self.load_value(right)?;
                let (lhs, rhs) = self.coerce_int_types(lhs, rhs);
                let kind = (self.api.get_type_kind)((self.api.type_of)(lhs));
                
                let val = if kind == LLVMTypeKind::LLVMIntegerTypeKind {
                    (self.api.build_icmp)(self.builder, LLVMIntPredicate::LLVMIntNE, lhs, rhs, c_str("ne").as_ptr())
                } else {
                    (self.api.build_fcmp)(self.builder, LLVMRealPredicate::LLVMRealONE, lhs, rhs, c_str("fne").as_ptr())
                };
                self.store_value(dest, val)?;
            }
            
            IrInstruction::Return { value } => {
                 match value {
                     Some(v) => {
                         let mut ret_val = self.load_value(v)?;
                         
                         // Truncate or extend return value to match function return type
                         if let Some(expected_ty) = self.current_return_type {
                             let val_ty = (self.api.type_of)(ret_val);
                             let val_kind = (self.api.get_type_kind)(val_ty);
                             let expected_kind = (self.api.get_type_kind)(expected_ty);
                             
                             // Only convert if both are integers
                             if val_kind == LLVMTypeKind::LLVMIntegerTypeKind &&
                                expected_kind == LLVMTypeKind::LLVMIntegerTypeKind {
                                 let val_width = (self.api.get_int_type_width)(val_ty);
                                 let expected_width = (self.api.get_int_type_width)(expected_ty);
                                 
                                 if val_width > expected_width {
                                     // Truncate
                                     let name = CString::new("trunc").unwrap();
                                     ret_val = (self.api.build_trunc)(self.builder, ret_val, expected_ty, name.as_ptr());
                                 } else if val_width < expected_width {
                                     // Sign extend
                                     let name = CString::new("sext").unwrap();
                                     ret_val = (self.api.build_sext)(self.builder, ret_val, expected_ty, name.as_ptr());
                                 }
                             }
                         }
                         
                         (self.api.build_ret)(self.builder, ret_val);
                     }
                     None => {
                         (self.api.build_ret_void)(self.builder);
                     }
                 }
            }
            
            IrInstruction::Call { func, args, dest } => {
                // First check if this is a call to a closure variable
                let actual_func_name = if let Some(closure_fn) = self.closure_functions.get(func) {
                    closure_fn.clone()
                } else {
                    // Map known runtime functions
                    match func.as_str() {
                        "println" => "falcon_println".to_string(),
                        "print" => "falcon_print".to_string(),
                        "print_int" => "falcon_print_int".to_string(),
                        "print_i32" => "falcon_print_i32".to_string(),
                        "print_float" => "falcon_print_float".to_string(),
                        "print_bool" => "falcon_print_bool".to_string(),
                        "print_newline" => "falcon_print_newline".to_string(),
                        "abs" => "falcon_abs".to_string(),
                        "min" => "falcon_min".to_string(),
                        "max" => "falcon_max".to_string(),
                        _ => func.clone(),
                    }
                };
                self.emit_named_call(&actual_func_name, args, dest)?;
            }
            
            IrInstruction::Label { name } => {
                 let func_val = self.current_function.unwrap();
                 
                 // Check if block already exists (forward reference)
                 let mut existing_block: LLVMBasicBlockRef = ptr::null_mut();
                 let mut block = (self.api.get_first_basic_block)(func_val);
                 while !block.is_null() {
                     let bb_name = CStr::from_ptr((self.api.get_basic_block_name)(block)).to_str().unwrap();
                     if bb_name == name {
                         existing_block = block;
                         break;
                     }
                     block = (self.api.get_next_basic_block)(block);
                 }
                 
                 let target_block = if existing_block.is_null() {
                     // Create new block
                     (self.api.append_basic_block)(self.context, func_val, c_str(name).as_ptr())
                 } else {
                     // Reuse existing forward-referenced block
                     existing_block
                 };
                 
                 // Fallthrough branch if needed
                 let current_block = (self.api.get_insert_block)(self.builder);
                 if (self.api.get_basic_block_terminator)(current_block).is_null() {
                     (self.api.build_br)(self.builder, target_block);
                 }
                 
                 (self.api.position_builder)(self.builder, target_block);
            }
            
            IrInstruction::Branch { label } => {
                 // Skip if current block already has a terminator (e.g., from a return)
                 let current_block = (self.api.get_insert_block)(self.builder);
                 if !(self.api.get_basic_block_terminator)(current_block).is_null() {
                     // Block already terminated, skip this branch
                 } else {
                     let func_val = self.current_function.unwrap();
                     let mut block = (self.api.get_first_basic_block)(func_val);
                     let mut target_block = ptr::null_mut();
                     
                     while !block.is_null() {
                         let bb_name = CStr::from_ptr((self.api.get_basic_block_name)(block)).to_str().unwrap();
                         if bb_name == label {
                             target_block = block;
                             break;
                         }
                         block = (self.api.get_next_basic_block)(block);
                     }
                     
                     if target_block.is_null() {
                         // Create it if not found (forward ref)
                         target_block = (self.api.append_basic_block)(self.context, func_val, c_str(label).as_ptr());
                     }
                     
                     (self.api.build_br)(self.builder, target_block);
                 }
            }
            
            IrInstruction::BranchCond { condition, true_label, false_label } => {
                let cond_val = self.load_value(condition)?;
                let func_val = self.current_function.unwrap();
                
                // Helper to find/create block
                let mut find_or_create = |label: &str| -> LLVMBasicBlockRef {
                     let mut block = (self.api.get_first_basic_block)(func_val);
                     while !block.is_null() {
                         let bb_name = unsafe { CStr::from_ptr((self.api.get_basic_block_name)(block)).to_str().unwrap() };
                         if bb_name == label { return block; }
                         block = (self.api.get_next_basic_block)(block);
                     }
                     // Create forward
                     (self.api.append_basic_block)(self.context, func_val, c_str(label).as_ptr())
                };
                
                let true_block = find_or_create(true_label);
                let false_block = find_or_create(false_label);
                
                (self.api.build_cond_br)(self.builder, cond_val, true_block, false_block);
            }

            IrInstruction::VolatileLoad { addr, dest } => {
                let addr_val = self.load_value(addr)?;
                let addr_ty = (self.api.type_of)(addr_val);
                let ptr_type = (self.api.ptr_type)(self.context, 0);
                let addr_ptr = if (self.api.get_type_kind)(addr_ty) == LLVMTypeKind::LLVMPointerTypeKind {
                    addr_val
                } else {
                    (self.api.build_int_to_ptr)(self.builder, addr_val, ptr_type, c_str("mmio_ptr").as_ptr())
                };

                let i64_type = (self.api.int64_type)(self.context);
                let load_inst =
                    (self.api.build_load2)(self.builder, i64_type, addr_ptr, c_str("mmio_load").as_ptr());
                (self.api.set_volatile)(load_inst, 1);
                self.store_value(dest, load_inst)?;
            }

            IrInstruction::VolatileStore { addr, value } => {
                let addr_val = self.load_value(addr)?;
                let addr_ty = (self.api.type_of)(addr_val);
                let ptr_type = (self.api.ptr_type)(self.context, 0);
                let addr_ptr = if (self.api.get_type_kind)(addr_ty) == LLVMTypeKind::LLVMPointerTypeKind {
                    addr_val
                } else {
                    (self.api.build_int_to_ptr)(self.builder, addr_val, ptr_type, c_str("mmio_ptr").as_ptr())
                };

                let value_val = self.load_value(value)?;
                let store_inst = (self.api.build_store)(self.builder, value_val, addr_ptr);
                (self.api.set_volatile)(store_inst, 1);
            }
            
            IrInstruction::StructInit { struct_name, fields, dest } => {
                let struct_ty = *self.struct_types.get(struct_name)
                    .ok_or_else(|| format!("Unknown struct: {}", struct_name))?;
                
                // Alloca for struct
                let ptr = (self.api.build_alloca)(self.builder, struct_ty, c_str("struct_tmp").as_ptr());
                
                // Store each field using GEP
                let field_indices = self.struct_field_indices.get(struct_name).unwrap().clone();
                for (field_name, field_val) in fields {
                    let idx = *field_indices.get(field_name)
                        .ok_or_else(|| format!("Unknown field: {}", field_name))?;
                    
                    let field_ptr = (self.api.build_struct_gep2)(
                        self.builder, struct_ty, ptr, idx, c_str("field_ptr").as_ptr()
                    );
                    let val = self.load_value(field_val)?;
                    (self.api.build_store)(self.builder, val, field_ptr);
                }
                
                // Store pointer in dest
                self.store_value(dest, ptr)?;
            }
            
            IrInstruction::FieldAccess { base, field, dest } => {
                let base_ptr = self.load_value(base)?;
                
                // Find struct type for this base and get field info
                let mut found = false;
                for (struct_name, field_indices) in &self.struct_field_indices {
                    if let Some(&idx) = field_indices.get(field) {
                        let struct_ty = *self.struct_types.get(struct_name).unwrap();
                        let field_ty = *self.struct_field_types
                            .get(struct_name)
                            .and_then(|m| m.get(field))
                            .ok_or_else(|| format!("Field type not found for {}.{}", struct_name, field))?;
                        
                        let field_ptr = (self.api.build_struct_gep2)(
                            self.builder, struct_ty, base_ptr, idx, c_str("field").as_ptr()
                        );
                        let val = (self.api.build_load2)(
                            self.builder,
                            field_ty,
                            field_ptr,
                            c_str("field_val").as_ptr()
                        );
                        self.store_value(dest, val)?;
                        found = true;
                        break;
                    }
                }
                if !found {
                    return Err(format!("Could not find field '{}' in any registered struct type", field));
                }
            }
            
            IrInstruction::Range { start, end, dest } => {
                // Use cached range type {i64 start, i64 end} for type consistency
                let range_ty = self.get_range_type();
                
                // Allocate the range struct
                let ptr = (self.api.build_alloca)(self.builder, range_ty, c_str("range").as_ptr());
                
                // Store start value
                let start_val = self.load_value(start)?;
                let start_ptr = (self.api.build_struct_gep2)(
                    self.builder, range_ty, ptr, 0, c_str("range_start").as_ptr()
                );
                (self.api.build_store)(self.builder, start_val, start_ptr);
                
                // Store end value
                let end_val = self.load_value(end)?;
                let end_ptr = (self.api.build_struct_gep2)(
                    self.builder, range_ty, ptr, 1, c_str("range_end").as_ptr()
                );
                (self.api.build_store)(self.builder, end_val, end_ptr);
                
                // Store pointer to dest
                self.store_value(dest, ptr)?;
            }
            
            IrInstruction::ArrayInit { elements, dest } => {
                // Layout: [length | elem0 | elem1 | ...]
                // Return pointer to elem0 so that meta[-1] reads length
                let i64_type = (self.api.int64_type)(self.context);
                let count = elements.len();
                let total = count + 1; // +1 for length prefix
                let buf_type = (self.api.array_type)(i64_type, total as c_uint);
                
                // Allocate buffer on stack
                let buf = (self.api.build_alloca)(self.builder, buf_type, c_str("arr_buf").as_ptr());
                
                let zero = (self.api.const_int)(i64_type, 0, 0);
                
                // Store length at index 0
                let len_val = (self.api.const_int)(i64_type, count as c_ulonglong, 0);
                let mut len_indices = [zero, zero];
                let len_ptr = (self.api.build_gep2)(
                    self.builder, buf_type, buf,
                    len_indices.as_mut_ptr(), 2,
                    c_str("len_ptr").as_ptr()
                );
                (self.api.build_store)(self.builder, len_val, len_ptr);
                
                // Store each element at indices 1..count+1
                for (i, elem) in elements.iter().enumerate() {
                    let elem_val = match elem {
                        IrValue::Constant(lit) => self.compile_literal(lit)?,
                        _ => {
                            match self.load_value(elem) {
                                Ok(v) => v,
                                Err(_) => (self.api.const_int)(i64_type, 0, 0),
                            }
                        }
                    };
                    
                    let idx = (self.api.const_int)(i64_type, (i + 1) as c_ulonglong, 0);
                    let mut indices = [zero, idx];
                    let elem_ptr = (self.api.build_gep2)(
                        self.builder, buf_type, buf,
                        indices.as_mut_ptr(), 2,
                        c_str("elem_ptr").as_ptr()
                    );
                    (self.api.build_store)(self.builder, elem_val, elem_ptr);
                }
                
                // Return pointer to element[0] (index 1 in buffer)
                // This way meta[-1] reads the length prefix
                let one = (self.api.const_int)(i64_type, 1, 0);
                let mut data_indices = [zero, one];
                let data_ptr = (self.api.build_gep2)(
                    self.builder, buf_type, buf,
                    data_indices.as_mut_ptr(), 2,
                    c_str("data_ptr").as_ptr()
                );
                self.store_value(dest, data_ptr)?;
            }
            
            IrInstruction::BoundsCheck { value, bound, on_fail: _ } => {
                // Bounds check: call falcon_bounds_check(index, length)
                let idx_val = self.load_value(value)?;
                let bound_val = self.load_value(bound)?;
                
                let check_func_name = c_str("falcon_bounds_check");
                let check_func = (self.api.get_named_function)(self.module, check_func_name.as_ptr());

                if check_func.is_null() {
                    return Err("LLVM lowering error: runtime function 'falcon_bounds_check' not found".to_string());
                }

                let check_type = *self.function_types.get("falcon_bounds_check")
                    .ok_or_else(|| "LLVM lowering error: function type for 'falcon_bounds_check' not found".to_string())?;
                let mut args = [idx_val, bound_val];
                (self.api.build_call2)(
                    self.builder,
                    check_type,
                    check_func,
                    args.as_mut_ptr(),
                    2,
                    c_str("").as_ptr()
                );
            }
            
            IrInstruction::Index { base, index, dest } => {
                let base_ptr = self.load_value(base)?;
                let idx_val = self.load_value(index)?;
                
                // Use GEP to get element pointer
                let i64_type = (self.api.int64_type)(self.context);
                let zero = (self.api.const_int)(i64_type, 0, 0);
                let mut indices = [zero, idx_val];
                
                // Get element type (assume i64 array for now)
                let array_type = (self.api.array_type)(i64_type, 0); // Size doesn't matter for GEP
                let elem_ptr = (self.api.build_gep2)(
                    self.builder,
                    array_type,
                    base_ptr,
                    indices.as_mut_ptr(),
                    2,
                    c_str("idx_ptr").as_ptr()
                );
                
                // Load the element
                let val = (self.api.build_load2)(self.builder, i64_type, elem_ptr, c_str("elem").as_ptr());
                self.store_value(dest, val)?;
            }
            
            // ======== BITWISE INSTRUCTIONS ========
            
            IrInstruction::BitAnd { left, right, dest } => {
                let lhs = self.load_value(left)?;
                let rhs = self.load_value(right)?;
                let (lhs, rhs) = self.coerce_int_types(lhs, rhs);
                let val = (self.api.build_and)(self.builder, lhs, rhs, c_str("bitand").as_ptr());
                self.store_value(dest, val)?;
            }
            
            IrInstruction::BitOr { left, right, dest } => {
                let lhs = self.load_value(left)?;
                let rhs = self.load_value(right)?;
                let (lhs, rhs) = self.coerce_int_types(lhs, rhs);
                let val = (self.api.build_or)(self.builder, lhs, rhs, c_str("bitor").as_ptr());
                self.store_value(dest, val)?;
            }
            
            IrInstruction::BitXor { left, right, dest } => {
                eprintln!("[DEBUG] Compiling BitXor: {:?} ^ {:?} -> {:?}", left, right, dest);
                let lhs = self.load_value(left)?;
                eprintln!("[DEBUG] BitXor lhs loaded");
                let rhs = self.load_value(right)?;
                eprintln!("[DEBUG] BitXor rhs loaded");
                let (lhs, rhs) = self.coerce_int_types(lhs, rhs);
                eprintln!("[DEBUG] BitXor types coerced");
                let val = (self.api.build_xor)(self.builder, lhs, rhs, c_str("bitxor").as_ptr());
                eprintln!("[DEBUG] BitXor built");
                self.store_value(dest, val)?;
                eprintln!("[DEBUG] BitXor stored");
            }
            
            IrInstruction::Shl { left, right, dest } => {
                let lhs = self.load_value(left)?;
                let rhs = self.load_value(right)?;
                let (lhs, rhs) = self.coerce_int_types(lhs, rhs);
                let val = (self.api.build_shl)(self.builder, lhs, rhs, c_str("shl").as_ptr());
                self.store_value(dest, val)?;
            }
            
            IrInstruction::Shr { left, right, dest } => {
                let lhs = self.load_value(left)?;
                let rhs = self.load_value(right)?;
                let (lhs, rhs) = self.coerce_int_types(lhs, rhs);
                let val = (self.api.build_ashr)(self.builder, lhs, rhs, c_str("shr").as_ptr());
                self.store_value(dest, val)?;
            }
            
            IrInstruction::Neg { operand, dest } => {
                let val = self.load_value(operand)?;
                let ty = (self.api.type_of)(val);
                let zero = (self.api.const_int)(ty, 0, 0);
                let result = (self.api.build_sub)(self.builder, zero, val, c_str("neg").as_ptr());
                self.store_value(dest, result)?;
            }
            
            IrInstruction::Not { operand, dest } => {
                let val = self.load_value(operand)?;
                let ty = (self.api.type_of)(val);
                let zero = (self.api.const_int)(ty, 0, 0);
                let result = (self.api.build_icmp)(self.builder, LLVMIntPredicate::LLVMIntEQ, val, zero, c_str("not").as_ptr());
                self.store_value(dest, result)?;
            }
            
            IrInstruction::BitNot { operand, dest } => {
                let val = self.load_value(operand)?;
                let result = (self.api.build_not)(self.builder, val, c_str("bitnot").as_ptr());
                self.store_value(dest, result)?;
            }
            
            IrInstruction::AddrOf { operand, dest } => {
                // Get the pointer to the variable's alloca (address-of)
                let name = operand.name();
                if let Some(&ptr) = self.variables.get(&name) {
                    // ptr is already an alloca pointer — that IS the address
                    self.store_value(dest, ptr)?;
                } else {
                    return Err(format!("AddrOf: variable '{}' not found", name));
                }
            }
            
            IrInstruction::PtrDeref { operand, dest } => {
                // Load the pointer value, then load from it
                let ptr_val = self.load_value(operand)?;
                let i64_type = (self.api.int64_type)(self.context);
                let result = (self.api.build_load2)(self.builder, i64_type, ptr_val, c_str("deref").as_ptr());
                self.store_value(dest, result)?;
            }
            
            // ======== ENUM INSTRUCTIONS ========
            // Enums are represented as { i32 tag, i64 payload } in LLVM
            
            IrInstruction::EnumInit { enum_name, variant, payload, dest } => {
                // Use cached enum type {i32 tag, i64 payload} for type consistency
                let enum_ty = self.get_enum_type();
                let i32_type = (self.api.int32_type)(self.context);
                let i64_type = (self.api.int64_type)(self.context);
                
                // Allocate the enum on stack
                let ptr = (self.api.build_alloca)(self.builder, enum_ty, c_str("enum_val").as_ptr());
                
                // Look up actual variant tag from enum definitions
                let tag = self.get_variant_tag(enum_name, variant).unwrap_or(0);
                let tag_ptr = (self.api.build_struct_gep2)(
                    self.builder, enum_ty, ptr, 0, c_str("tag_ptr").as_ptr()
                );
                let tag_val = (self.api.const_int)(i32_type, tag as c_ulonglong, 0);
                (self.api.build_store)(self.builder, tag_val, tag_ptr);
                
                // Store payload (field 1) if present
                let payload_ptr = (self.api.build_struct_gep2)(
                    self.builder, enum_ty, ptr, 1, c_str("payload_ptr").as_ptr()
                );
                let payload_val = if let Some(p) = payload {
                    self.load_value(p)?
                } else {
                    // No payload - store 0
                    (self.api.const_int)(i64_type, 0, 0)
                };
                (self.api.build_store)(self.builder, payload_val, payload_ptr);
                
                // Store pointer to dest
                self.store_value(dest, ptr)?;
            }
            
            IrInstruction::EnumTag { value, dest } => {
                // Use cached enum type for consistency
                let enum_ty = self.get_enum_type();
                let i32_type = (self.api.int32_type)(self.context);
                
                let enum_ptr = self.load_value(value)?;
                let tag_ptr = (self.api.build_struct_gep2)(
                    self.builder, enum_ty, enum_ptr, 0, c_str("tag_ptr").as_ptr()
                );
                let tag_val = (self.api.build_load2)(
                    self.builder, i32_type, tag_ptr, c_str("tag").as_ptr()
                );
                self.store_value(dest, tag_val)?;
            }
            
            IrInstruction::EnumPayload { value, variant: _, dest } => {
                // Use cached enum type for consistency
                let enum_ty = self.get_enum_type();
                let i64_type = (self.api.int64_type)(self.context);
                
                let enum_ptr = self.load_value(value)?;
                let payload_ptr = (self.api.build_struct_gep2)(
                    self.builder, enum_ty, enum_ptr, 1, c_str("payload_ptr").as_ptr()
                );
                let payload_val = (self.api.build_load2)(
                    self.builder, i64_type, payload_ptr, c_str("payload").as_ptr()
                );
                self.store_value(dest, payload_val)?;
            }
            
            IrInstruction::ClosureCreate { closure_id, params, body, captures: _, dest } => {
                // Generate a unique function name for this closure
                let closure_fn_name = format!("__closure_{}", closure_id);
                log::debug!("Compiling closure #{} as function '{}' with {} params", 
                    closure_id, closure_fn_name, params.len());
                
                // Save current state so we can restore after compiling closure function
                let saved_function = self.current_function;
                let saved_entry = self.entry_block;
                let saved_return_type = self.current_return_type;
                let saved_variables = self.variables.clone();
                let saved_variable_types = self.variable_types.clone();
                
                // Create function type: (params...) -> i64
                let i64_type = (self.api.int64_type)(self.context);
                let mut param_types: Vec<LLVMTypeRef> = params.iter()
                    .map(|_| i64_type)  // All params are i64 for now
                    .collect();
                let fn_type = (self.api.function_type)(
                    i64_type,  // Return type: i64
                    param_types.as_mut_ptr(),
                    params.len() as c_uint,
                    0  // Not variadic
                );
                
                // Create the function
                let fn_name_cstr = c_str(&closure_fn_name);
                let closure_fn = (self.api.add_function)(
                    self.module, 
                    fn_name_cstr.as_ptr(), 
                    fn_type
                );
                
                // Register function type for later calls
                self.function_types.insert(closure_fn_name.clone(), fn_type);
                
                // Create entry block for closure function
                let entry = (self.api.append_basic_block)(
                    self.context, closure_fn, c_str("entry").as_ptr()
                );
                (self.api.position_builder)(self.builder, entry);
                
                // Set up closure compilation context
                self.current_function = Some(closure_fn);
                self.entry_block = Some(entry);
                self.current_return_type = Some(i64_type);
                self.variables.clear();
                self.variable_types.clear();
                
                // Set up parameters: get LLVM function params and store in variables
                for (i, param) in params.iter().enumerate() {
                    let llvm_param = (self.api.get_param)(closure_fn, i as c_uint);
                    
                    // Allocate and store parameter
                    let param_name = c_str(&param.name);
                    let alloca = (self.api.build_alloca)(self.builder, i64_type, param_name.as_ptr());
                    (self.api.build_store)(self.builder, llvm_param, alloca);
                    
                    self.variables.insert(param.name.clone(), alloca);
                    self.variable_types.insert(param.name.clone(), i64_type);
                }
                
                // Compile closure body
                for body_inst in body {
                    self.compile_instruction(body_inst)?;
                }
                
                // Get the result value from the last instruction and return it
                let return_value = if let Some(last_inst) = body.last() {
                    match last_inst {
                        IrInstruction::Literal { dest: body_dest, .. } |
                        IrInstruction::Add { dest: body_dest, .. } |
                        IrInstruction::Sub { dest: body_dest, .. } |
                        IrInstruction::Mul { dest: body_dest, .. } |
                        IrInstruction::Div { dest: body_dest, .. } |
                        IrInstruction::Mod { dest: body_dest, .. } |
                        IrInstruction::Move { dest: body_dest, .. } => {
                            if let Some(ptr) = self.variables.get(&body_dest.name()) {
                                (self.api.build_load2)(
                                    self.builder, i64_type, *ptr, c_str("ret_val").as_ptr()
                                )
                            } else {
                                // Fallback: return 0
                                (self.api.const_int)(i64_type, 0, 0)
                            }
                        }
                        _ => {
                            // Return 0 for unhandled cases
                            (self.api.const_int)(i64_type, 0, 0)
                        }
                    }
                } else {
                    // Empty body returns 0
                    (self.api.const_int)(i64_type, 0, 0)
                };
                
                // Build return instruction
                (self.api.build_ret)(self.builder, return_value);
                
                // Restore parent function context
                self.current_function = saved_function;
                self.entry_block = saved_entry;
                self.current_return_type = saved_return_type;
                self.variables = saved_variables;
                self.variable_types = saved_variable_types;
                
                // Position builder back in parent function
                if let Some(func) = self.current_function {
                    let last_block = (self.api.get_first_basic_block)(func);
                    if !last_block.is_null() {
                        (self.api.position_builder)(self.builder, last_block);
                    }
                }
                
                // Track the temp -> closure function mapping
                let dest_name = dest.name();
                self.temp_to_closure.insert(dest_name, closure_fn_name);
            }
            
            IrInstruction::ClosureCall { closure, args, dest } => {
                let closure_fn_name = self.resolve_closure_function_name(closure)?;
                self.emit_named_call(&closure_fn_name, args, dest)?;
            }
            
            IrInstruction::Cast { operand, target_type, dest } => {
                let src_val = self.load_value(operand)?;
                let target_llvm_ty = self.ir_type_to_llvm(target_type)?;
                let src_ty = (self.api.type_of)(src_val);
                let src_kind = (self.api.get_type_kind)(src_ty);
                let dst_kind = (self.api.get_type_kind)(target_llvm_ty);
                
                let result = match (src_kind, dst_kind) {
                    // int -> int (trunc or extend)
                    (LLVMTypeKind::LLVMIntegerTypeKind, LLVMTypeKind::LLVMIntegerTypeKind) => {
                        let src_width = (self.api.get_int_type_width)(src_ty);
                        let dst_width = (self.api.get_int_type_width)(target_llvm_ty);
                        if src_width == dst_width {
                            src_val // same size, no-op
                        } else if src_width > dst_width {
                            // Truncate (e.g., i64 -> u8)
                            (self.api.build_trunc)(self.builder, src_val, target_llvm_ty, c_str("cast_trunc").as_ptr())
                        } else {
                            // Extend — use sign-extend for signed types, zero-extend for unsigned
                            let is_unsigned = matches!(target_type, 
                                IrType::Int(IntType::U8) | IrType::Int(IntType::U16) | 
                                IrType::Int(IntType::U32) | IrType::Int(IntType::U64) | 
                                IrType::Int(IntType::U128) | IrType::Int(IntType::USize));
                            if is_unsigned {
                                (self.api.build_zext)(self.builder, src_val, target_llvm_ty, c_str("cast_zext").as_ptr())
                            } else {
                                (self.api.build_sext)(self.builder, src_val, target_llvm_ty, c_str("cast_sext").as_ptr())
                            }
                        }
                    }
                    // int -> ptr (inttoptr)
                    (LLVMTypeKind::LLVMIntegerTypeKind, LLVMTypeKind::LLVMPointerTypeKind) => {
                        (self.api.build_int_to_ptr)(self.builder, src_val, target_llvm_ty, c_str("cast_itp").as_ptr())
                    }
                    // ptr -> int (ptrtoint)
                    (LLVMTypeKind::LLVMPointerTypeKind, LLVMTypeKind::LLVMIntegerTypeKind) => {
                        (self.api.build_ptr_to_int)(self.builder, src_val, target_llvm_ty, c_str("cast_pti").as_ptr())
                    }
                    // float -> int (fptosi)
                    (LLVMTypeKind::LLVMDoubleTypeKind | LLVMTypeKind::LLVMFloatTypeKind, LLVMTypeKind::LLVMIntegerTypeKind) => {
                        (self.api.build_fptosi)(self.builder, src_val, target_llvm_ty, c_str("cast_fti").as_ptr())
                    }
                    // int -> float (sitofp)
                    (LLVMTypeKind::LLVMIntegerTypeKind, LLVMTypeKind::LLVMDoubleTypeKind | LLVMTypeKind::LLVMFloatTypeKind) => {
                        (self.api.build_sitofp)(self.builder, src_val, target_llvm_ty, c_str("cast_itf").as_ptr())
                    }
                    // Same type or unknown: just use src_val
                    _ => src_val,
                };
                
                self.store_value(dest, result)?;
            }
            
            _ => {
                return Err(format!(
                    "LLVM lowering error: unimplemented instruction variant: {:?}",
                    inst
                ));
            }
        }
        Ok(())
    }

    unsafe fn emit_named_call(
        &mut self,
        function_name: &str,
        args: &[IrValue],
        dest: &Option<IrValue>,
    ) -> Result<(), String> {
        let c_func_name = CString::new(function_name).unwrap();
        let func_ty = *self.function_types.get(function_name)
            .ok_or_else(|| format!("Function type not found for: {}", function_name))?;
        let mut func_val = (self.api.get_named_function)(self.module, c_func_name.as_ptr());
        if func_val.is_null() {
            // Some runtime/extern symbols may not be materialized yet in edge paths.
            // Materialize lazily using the already-known signature.
            func_val = (self.api.add_function)(self.module, c_func_name.as_ptr(), func_ty);
            if func_val.is_null() {
                return Err(format!("Function not found: {}", function_name));
            }
        }

        // Get expected parameter types from function signature
        let param_count = (self.api.count_param_types)(func_ty) as usize;
        let mut expected_types: Vec<LLVMTypeRef> = vec![ptr::null_mut(); param_count];
        if param_count > 0 {
            (self.api.get_param_types)(func_ty, expected_types.as_mut_ptr());
        }

        // Load arguments and coerce integer types to match expected parameter types
        let mut llvm_args = Vec::new();
        for (i, arg) in args.iter().enumerate() {
            let mut arg_val = self.load_value(arg)?;

            // Coerce argument type to match expected parameter type
            if i < expected_types.len() {
                let expected_ty = expected_types[i];
                let arg_ty = (self.api.type_of)(arg_val);
                let expected_kind = (self.api.get_type_kind)(expected_ty);
                let arg_kind = (self.api.get_type_kind)(arg_ty);

                // Integer type coercion
                if expected_kind == LLVMTypeKind::LLVMIntegerTypeKind && arg_kind == LLVMTypeKind::LLVMIntegerTypeKind {
                    let expected_width = (self.api.get_int_type_width)(expected_ty);
                    let arg_width = (self.api.get_int_type_width)(arg_ty);

                    if arg_width < expected_width {
                        // Sign-extend smaller to larger
                        let name = CString::new("sext").unwrap();
                        arg_val = (self.api.build_sext)(self.builder, arg_val, expected_ty, name.as_ptr());
                    } else if arg_width > expected_width {
                        // Truncate larger to smaller
                        let name = CString::new("trunc").unwrap();
                        arg_val = (self.api.build_trunc)(self.builder, arg_val, expected_ty, name.as_ptr());
                    }
                }
            }

            llvm_args.push(arg_val);
        }

        let ret_type_kind = (self.api.get_type_kind)((self.api.get_return_type)(func_ty));
        let name_c = CString::new("call").unwrap();
        let empty_c = CString::new("").unwrap();
        let name_ptr = if ret_type_kind == LLVMTypeKind::LLVMVoidTypeKind {
            empty_c.as_ptr()
        } else {
            name_c.as_ptr()
        };

        let call_val = (self.api.build_call2)(
            self.builder,
            func_ty,
            func_val,
            llvm_args.as_mut_ptr(),
            llvm_args.len() as c_uint,
            name_ptr
        );

        // If NOT void, store result
        if ret_type_kind != LLVMTypeKind::LLVMVoidTypeKind {
            if let Some(d) = dest {
                self.store_value(d, call_val)?;
            }
        }

        Ok(())
    }

    fn resolve_closure_function_name(&self, closure: &IrValue) -> Result<String, String> {
        let closure_name = closure.name();

        if let Some(function_name) = self.closure_functions.get(&closure_name) {
            return Ok(function_name.clone());
        }

        if let Some(function_name) = self.temp_to_closure.get(&closure_name) {
            return Ok(function_name.clone());
        }

        Err(format!(
            "Closure value '{}' has no generated function mapping",
            closure_name
        ))
    }
    
    unsafe fn compile_literal(&self, lit: &IrLiteral) -> Result<LLVMValueRef, String> {
        match lit {
            IrLiteral::Int(n) => {
                Ok((self.api.const_int)((self.api.int64_type)(self.context), *n as c_ulonglong, 1))
            }
            IrLiteral::Float(f) => {
                Ok((self.api.const_real)((self.api.double_type)(self.context), *f))
            }
            IrLiteral::Bool(b) => {
                 Ok((self.api.const_int)((self.api.int1_type)(self.context), *b as c_ulonglong, 0))
            }
            IrLiteral::String(s) => {
                let c_str = CString::new(s.as_str()).unwrap();
                let global_str = (self.api.build_global_string_ptr)(self.builder, c_str.as_ptr(), b"str\0".as_ptr() as *const _);
                Ok(global_str)
            }
            IrLiteral::Char(c) => {
                 Ok((self.api.const_int)((self.api.int8_type)(self.context), *c as c_ulonglong, 0))
            }
        }
    }
    
    unsafe fn store_value(&mut self, dest: &IrValue, val: LLVMValueRef) -> Result<(), String> {
         let name = dest.name();
         let c_name = CString::new(name.as_str()).unwrap();
         let ty = (self.api.type_of)(val);

         if !self.variables.contains_key(&name) {
             // CRITICAL: Create alloca at START of entry block, not at current position
             // This ensures variables created in loops are accessible on all iterations
             // and that allocas don't appear after terminators
             let current_block = (self.api.get_insert_block)(self.builder);
             
             // Move to entry block for alloca
             if let Some(entry) = self.entry_block {
                 let first_inst = (self.api.get_first_instruction)(entry);
                 if !first_inst.is_null() {
                     // Position BEFORE the first instruction to insert at the start
                     (self.api.position_builder_before)(self.builder, first_inst);
                 } else {
                     // Empty block, just position at the block
                     (self.api.position_builder)(self.builder, entry);
                 }
             }
             
             let alloca = (self.api.build_alloca)(self.builder, ty, c_name.as_ptr());
             self.variables.insert(name.clone(), alloca);
             self.variable_types.insert(name.clone(), ty);
             
             // Restore original position
             (self.api.position_builder)(self.builder, current_block);
         }
         
         let ptr = self.variables.get(&name).unwrap();
         
         // Truncate or extend value if types don't match (e.g., i64 value into u8 alloca)
         let stored_ty = *self.variable_types.get(&name).unwrap();
         let val_ty = (self.api.type_of)(val);
         let val_kind = (self.api.get_type_kind)(val_ty);
         let stored_kind = (self.api.get_type_kind)(stored_ty);
         
         let final_val = if val_kind == LLVMTypeKind::LLVMIntegerTypeKind 
             && stored_kind == LLVMTypeKind::LLVMIntegerTypeKind 
         {
             let val_width = (self.api.get_int_type_width)(val_ty);
             let stored_width = (self.api.get_int_type_width)(stored_ty);
             if val_width > stored_width {
                 // Truncate wider value to narrower alloca type
                 let trunc_name = CString::new("trunc").unwrap();
                 (self.api.build_trunc)(self.builder, val, stored_ty, trunc_name.as_ptr())
             } else if val_width < stored_width {
                 // Extend narrower value to wider alloca type
                 let sext_name = CString::new("sext").unwrap();
                 (self.api.build_sext)(self.builder, val, stored_ty, sext_name.as_ptr())
             } else {
                 val
             }
         } else {
             val
         };
         
         (self.api.build_store)(self.builder, final_val, *ptr);
         Ok(())
    }

    unsafe fn load_value(&mut self, val: &IrValue) -> Result<LLVMValueRef, String> {
        match val {
            IrValue::Variable(_) | IrValue::Temporary(_) => {
                let name = val.name();
                
                if let Some(ptr) = self.variables.get(&name) {
                    let ty = self.variable_types.get(&name)
                        .ok_or_else(|| format!("Type not found for: {}", name))?;
                    
                    let c_name = CString::new(name.as_str()).unwrap();
                    let loaded = (self.api.build_load2)(self.builder, *ty, *ptr, c_name.as_ptr());
                    Ok(loaded)
                } else {
                    Err(format!(
                        "LLVM lowering error: value '{}' not found in symbol table",
                        name
                    ))
                }
            }
            IrValue::Constant(lit) => self.compile_literal(lit),
        }
    }
    
    unsafe fn ir_type_to_llvm(&self, ty: &IrType) -> Result<LLVMTypeRef, String> {
        match ty {
            IrType::Int(int_ty) => {
                match int_ty {
                    IntType::I8 | IntType::U8 => Ok((self.api.int8_type)(self.context)),
                    IntType::I16 | IntType::U16 => Ok((self.api.int16_type)(self.context)),
                    IntType::I32 | IntType::U32 => Ok((self.api.int32_type)(self.context)),
                    IntType::I64 | IntType::U64 | IntType::ISize | IntType::USize => Ok((self.api.int64_type)(self.context)),
                    IntType::I128 | IntType::U128 => {
                        return Err("i128/u128 types are not yet supported in the LLVM backend. \
                            Use i64 or u64 instead.".to_string());
                    }
                }
            }
            IrType::Float(float_ty) => {
                 match float_ty {
                     FloatType::F32 => Ok((self.api.double_type)(self.context)),
                     FloatType::F64 => Ok((self.api.double_type)(self.context)),
                 }
            }
            IrType::Bool => Ok((self.api.int1_type)(self.context)),
            IrType::String | IrType::Str => Ok((self.api.ptr_type)(self.context, 0)),
            IrType::Unit => Ok((self.api.int8_type)(self.context)),
            IrType::Pointer { .. } | IrType::Reference { .. } => Ok((self.api.ptr_type)(self.context, 0)),
            IrType::Named(name) => {
                // Look up real struct types from the registry
                if let Some(&struct_ty) = self.struct_types.get(name) {
                    // Return pointer-to-struct type for function params and return values
                    // Struct values are always passed by pointer in Falcon
                    Ok((self.api.ptr_type)(self.context, 0))
                } else if self.enum_defs.contains_key(name) {
                    // Enums use tagged union pattern — always pointer
                    Ok((self.api.ptr_type)(self.context, 0))
                } else {
                    return Err(format!("Unknown named type '{}' in LLVM codegen. \
                        This should have been rejected earlier — add struct/enum definition or fix type resolution.", name));
                }
            }
            _ => return Err(format!("Unsupported IR type in LLVM codegen: {:?}. \
                This should have been rejected earlier.", ty)),
        }
    }

    /// Coerce two integer values to have the same type (the larger one)
    /// This is needed because LLVM icmp requires both operands to have the same type
    unsafe fn coerce_int_types(&self, lhs: LLVMValueRef, rhs: LLVMValueRef) -> (LLVMValueRef, LLVMValueRef) {
        let lhs_ty = (self.api.type_of)(lhs);
        let rhs_ty = (self.api.type_of)(rhs);
        
        let lhs_kind = (self.api.get_type_kind)(lhs_ty);
        let rhs_kind = (self.api.get_type_kind)(rhs_ty);
        
        // Only coerce if both are integers
        if lhs_kind != LLVMTypeKind::LLVMIntegerTypeKind || rhs_kind != LLVMTypeKind::LLVMIntegerTypeKind {
            return (lhs, rhs);
        }
        
        let lhs_width = (self.api.get_int_type_width)(lhs_ty);
        let rhs_width = (self.api.get_int_type_width)(rhs_ty);
        
        if lhs_width == rhs_width {
            (lhs, rhs)
        } else if lhs_width < rhs_width {
            // Extend lhs to rhs type
            let name = CString::new("sext").unwrap();
            let extended = (self.api.build_sext)(self.builder, lhs, rhs_ty, name.as_ptr());
            (extended, rhs)
        } else {
            // Extend rhs to lhs type
            let name = CString::new("sext").unwrap();
            let extended = (self.api.build_sext)(self.builder, rhs, lhs_ty, name.as_ptr());
            (lhs, extended)
        }
    }

    pub fn write_object_file_for_triple(
        &self,
        path: &Path,
        target_triple: Option<&str>,
    ) -> Result<(), String> {
        // Init targets
        unsafe {
            (self.api.init_target_infos)();
            (self.api.init_targets)();
            (self.api.init_target_mcs)();
            (self.api.init_asm_printers)();
            (self.api.init_asm_parsers)();
            if let Some(f) = self.api.init_aarch64_target_infos.as_ref() { f() }
            if let Some(f) = self.api.init_aarch64_targets.as_ref() { f() }
            if let Some(f) = self.api.init_aarch64_target_mcs.as_ref() { f() }
            if let Some(f) = self.api.init_aarch64_asm_printers.as_ref() { f() }
            if let Some(f) = self.api.init_aarch64_asm_parsers.as_ref() { f() }
            if let Some(f) = self.api.init_riscv_target_infos.as_ref() { f() }
            if let Some(f) = self.api.init_riscv_targets.as_ref() { f() }
            if let Some(f) = self.api.init_riscv_target_mcs.as_ref() { f() }
            if let Some(f) = self.api.init_riscv_asm_printers.as_ref() { f() }
            if let Some(f) = self.api.init_riscv_asm_parsers.as_ref() { f() }

            let triple_cstr = target_triple.map(|triple| CString::new(triple).unwrap());
            let triple_ptr = if let Some(ref triple) = triple_cstr {
                triple.as_ptr()
            } else {
                (self.api.get_default_triple)()
            };
            let mut target: LLVMTargetRef = ptr::null_mut();
            let mut error: *mut c_char = ptr::null_mut();
            
            let failed = (self.api.get_target_from_triple)(triple_ptr, &mut target, &mut error);
            if failed != 0 {
                 let msg = CStr::from_ptr(error).to_string_lossy().into_owned();
                 return Err(msg);
            }
            
            let cpu = CString::new("generic").unwrap();
            let features = CString::new("").unwrap();
            
            let target_machine = (self.api.create_target_machine)(
                target,
                triple_ptr,
                cpu.as_ptr(),
                features.as_ptr(),
                2, // OptLevel default
                0, // Reloc default
                0  // CodeModel default
            );
            
            let path_str = CString::new(path.to_str().unwrap()).unwrap();
            let emit_failed = (self.api.emit_to_file)(
                target_machine,
                self.module,
                path_str.as_ptr() as *mut _,
                1, // ObjectFile
                &mut error
            );
            
            if emit_failed != 0 {
                 let msg = CStr::from_ptr(error).to_string_lossy().into_owned();
                 return Err(msg);
            }
        }
        Ok(())
    }

    pub fn write_object_file(&self, path: &Path) -> Result<(), String> {
        self.write_object_file_for_triple(path, None)
    }

    pub fn dump_module(&self) -> String {
        unsafe {
            let c_str_ptr = (self.api.print_module_to_string)(self.module);
            if c_str_ptr.is_null() {
                return "<failed to print LLVM module>".to_string();
            }
            let s = CStr::from_ptr(c_str_ptr).to_string_lossy().into_owned();
            (self.api.dispose_message)(c_str_ptr); // LLVM documentation says use LLVMDisposeMessage for PrintModuleToString result
            s
        }
    }
}


