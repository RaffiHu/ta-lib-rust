# CLAUDE.md - TA-Lib Rust Integration Guide

## Build & Test
- Build: `bin/gen_code` → `cargo check && cargo fmt` → `bin/ta_regtest`
- Workflow: Modify generator → Rebuild gen_code → Run gen_code → Test Rust

## Rust Architecture  
- Generated from C sources via `src/tools/gen_code/gen_rust.c`
- Functions wrapped in `impl Core { pub fn mult(...) -> RetCode }`
- Parameter types: `usize` for indices, `&[f64]`/`&mut [f64]` for arrays
- Maintains TA-Lib API compatibility (camelCase parameters)

## Code Generation
- Multi-language: C, .NET, Java, Rust from single source
- GENCODE sections in `.c` files contain language-specific implementations
- Cross-language macros in `include/ta_defs.h` handle syntax differences
- Key files: `gen_code.c` (validation), `gen_rust.c` (signatures), `ta_defs.h` (macros)

## Critical Fixes Implemented

### 1. Loop Variable Declaration (FIXED)
**Problem**: `DECLARE_INDEX_VAR(i)` creates unused `let mut i: usize;` since `FOR_EACH_OUTPUT` creates own binding
**Solution**: `DECLARE_LOOP_VAR(i)` - no-op for Rust, normal declaration for C

### 2. Index Validation (FIXED)
**Problem**: `startIdx < 0` meaningless for `usize` types  
**Solution**: Modified `gen_code.c:3464-3476` - Rust skips negative checks, only validates `endIdx < startIdx`

### 3. Type Conversion (FIXED)
**Problem**: f32 inputs → f64 outputs in single precision functions
**Solution**: `OUTPUT_F64(val)` macro casts to f64 in Rust, no-op in C

## Macro System
```c
// FOR_EACH_OUTPUT: C vs Rust syntax conversion
FOR_EACH_OUTPUT(startIdx, endIdx, i, outIdx)
   outReal[outIdx] = OUTPUT_F64(inReal0[i] * inReal1[i]);
FOR_EACH_OUTPUT_END(outIdx)
```

## Current Status
- ✅ MULT function compiles clean with `cargo check`
- ✅ Cross-language generation working for all targets
- ✅ Template system with proper lint allowances
- 🔄 Phase 2: Index validation fixes applied, pending regeneration
- 📋 Phase 3: Documentation generation (conditional based on function metadata)

## Key Technical Insights
- **Generator Pattern**: Modify generators, not generated code
- **Build Order**: gen_rust.c changes → rebuild gen_code → run gen_code → test
- **Macro Strategy**: 65% automated via macros, 25% specialized patterns, 15% manual
- **Type Safety**: Rust `usize` requires different validation than C `int`