# Issue #23 - Current Status & Analysis

## Summary

Issue #23 is more complex than initially described. The actual problem is:

1. User tries to use `#[gonfig(nested)]` attribute - **which doesn't exist**
2. When darling (attribute parser) encounters unknown attribute, it generates error with unqualified `core::compile_error!()`
3. When user has `use core as tradesmith_core;`, the compiler can't resolve the error message

## What We've Done So Far

### Commit 1: `9754b21` (Initial Fix - Incomplete)
- Fixed `std::path::Path` to use fully qualified paths (::std::path::Path)
- Added comprehensive regression tests for std/core aliasing
- **BUT**: This didn't fix the user's actual issue

### Current Changes (Uncommitted)
- Added `nested` field to `GonfigField` struct
- Marked as `#[allow(dead_code)]` and reserved for future use
- Now the `#[gonfig(nested)]` attribute is **accepted** (prevents darling error)
- Test confirms it compiles with core alias

## The Real Question

**We need user clarification on what `nested` should do:**

### Option A: Accept But Don't Implement (Current State)
- ✅ Fixes compilation error
- ✅ Allows code to compile with core alias
- ❌ `nested` attribute does nothing functionally
- ❌ Misleading to users who expect it to work

### Option B: Implement Full Nested Support
- ✅ Provides actual functionality
- ✅ Useful feature for users
- ❌ More complex implementation
- ❌ Need to define exact behavior:
  - How do prefixes work?
  - Do fields flatten or stay nested?
  - How do defaults propagate?

### Option C: Remove from Issue Example
- Ask user if they actually need `nested` feature
- Maybe they just used it as an example
- They might only need the std::path::Path fix

## Waiting For

GitHub comment posted: https://github.com/0xvasanth/gonfig/issues/23#issuecomment-3733426020

**Questions asked:**
1. What should `nested` do functionally?
2. How should environment variable prefixes work with nested structs?
3. Is this the same as the existing `flatten` attribute?
4. Do they prefer Option A, B, or C?

## Test Results

All new tests pass:
- ✅ issue_23_core_alias (2 tests)
- ✅ issue_23_std_alias (2 tests)
- ✅ issue_23_both_aliases (4 tests)
- ✅ issue_23_nested_attr (1 test) - **New: verifies nested compiles**

**Total: 9 passing regression tests for issue #23**

## Files Changed

```
gonfig_derive/src/lib.rs          - Added nested field
tests/issue_23_nested_attr.rs     - New test for nested attribute
```

## Next Steps

1. **Wait for user response** on GitHub issue
2. Based on response:
   - If Option A: Amend commit, add docs, done
   - If Option B: Implement full nested feature
   - If Option C: Revert nested, focus on other fixes

## Technical Notes

### Why the Original Fix Wasn't Enough

The std::path::Path fix (commit `9754b21`) solved ONE potential issue but not THE issue in the user's reproduction case. Their example specifically uses `#[gonfig(nested)]` which triggers a different code path in darling.

### The Darling Issue

Darling generates compile errors like this:
```rust
::core::compile_error!("Unknown field: `nested`")
```

When user has `use core as my_core;`, this becomes:
```rust
my_core::compile_error!("Unknown field: `nested`")  // ERROR: compile_error not in my_core
```

By adding `nested` to GonfigField, darling accepts it and doesn't generate the error.

### Complete Fix Requires

1. ✅ Fully qualified std::path::Path (done in `9754b21`)
2. ✅ Accept `nested` attribute (done, uncommitted)
3. ❓ Implement `nested` functionality (pending user clarification)
