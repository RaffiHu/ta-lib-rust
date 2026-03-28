# TA-Lib Rust Conversion Plan

This document outlines the step-by-step plan for converting TA-Lib's C functions to Rust, starting with the MULT function as a prototype.

## Phase 1: Function Signature Generation

### Function Signature Fixes in gen_rust.c
- [x] Fix `printRustLookbackFunctionSignature`:
  - [x] Generate proper Rust signature with parameters
  - [x] Add proper return type (`-> i32`)
  - [x] Remove braces from function generation

- [x] Fix `printRustDoublePrecisionFunctionSignature`:
  - [x] Generate proper Rust parameter types (`&[f64]` for arrays)
  - [x] Use proper borrowing for output parameters (`&mut i32`)
  - [x] Add proper return type (`-> RetCode`)
  - [x] Remove braces from function generation

- [x] Fix `printRustSinglePrecisionFunctionSignature`:
  - [x] Generate proper Rust parameter types for single precision (`&[f32]`)
  - [x] Use consistent patterns for output parameters
  - [x] Add proper return type
  - [x] Remove braces from function generation

### Issues Discovered
- [x] Added missing `TA_IN_PRICE_MASK` constant definition in gen_rust.c
- [ ] Current output still has C code mixed with Rust code (`#else` statements in the output)
- [ ] Need to better understand how the preprocessor and template system work together

## Phase 2: Fix Function Body Generation

- [ ] Identify where the mixed C/Rust code is coming from
- [ ] Modify preprocessor logic to properly separate Rust and C code
- [ ] Ensure Rust-specific syntax is correctly used:
  - [ ] For loops should use `for i in start..=end` syntax
  - [ ] Array access should use proper usize casting
  - [ ] Output assignments should use Rust dereferencing
  - [ ] Return statements should use Rust syntax

## Phase 3: Improve Code Generation Templates

- [ ] Update templates in `src/ta_abstract/templates/`:
  - [ ] Add proper Rust-specific structures to `ta_func_mod.rs.template`
  - [ ] Update `ta_x.rs.template` to correctly handle Rust code
  - [ ] Add support for Rust-specific syntax in the templates

- [ ] Modify gen_code.c and gen_rust.c:
  - [ ] Update `genRustCodePhase2` to handle code conversion properly
  - [ ] Improve handling of C to Rust syntax differences
  - [ ] Add proper filtering of C-specific code for Rust output

## Phase 4: Create Cargo.toml and Support Files

- [ ] Create a basic `Cargo.toml` file:
  - [ ] Define project name, version, and authors
  - [ ] Add any necessary dependencies
  - [ ] Define library structure

- [ ] Create Rust module structure:
  - [ ] Ensure `core` struct is properly defined
  - [ ] Set up proper module exports
  - [ ] Implement RetCode and other supporting types

## Phase 5: Test and Iterate

- [ ] Test with MULT function:
  - [ ] Generate code with `bin/gen_code`
  - [ ] Verify the generated Rust code compiles
  - [ ] Create simple test cases
  - [ ] Compare results with C implementation

- [ ] Apply improvements based on testing:
  - [ ] Refine template and code generation
  - [ ] Fix any issues discovered
  - [ ] Document lessons learned

## Phase 6: Generalize and Apply to SMA

- [ ] Extend implementation to handle SMA:
  - [ ] Update optional parameter handling for SMA's time period
  - [ ] Test SMA generation with `bin/gen_code`
  - [ ] Verify SMA implementation matches C version

## Phase 7: Documentation and Finalization

- [ ] Document the Rust interface:
  - [ ] Update README and documentation
  - [ ] Provide usage examples
  - [ ] Document any Rust-specific considerations

- [ ] Final integration testing:
  - [ ] Verify all functions work as expected
  - [ ] Check performance against C implementation