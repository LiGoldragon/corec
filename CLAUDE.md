# corec — Core Compiler

corec reads `.core` domain definitions and generates Rust source
with rkyv derives. Bootstrap seed — the generic tool that
synth-core, aski-core, and sema-core use to produce their rkyv
contract types.

## Usage

```
corec <input-dir> <output-file>
```

Reads all `.core` files from input-dir, generates Rust to output-file.

## Role in the Pipeline

```
corec       — .core → Rust with rkyv derives (this tool)
synth-core  — grammar .core + corec → Rust rkyv types (askicc↔askic contract)
aski-core   — parse tree .core + corec → Rust rkyv types (askic↔veric↔semac contract)
sema-core   — veric-output .core + corec → Rust rkyv types (veric↔semac contract)
askicc      — uses synth-core types → dsls.rkyv (dsl tree, all 4 DSLs)
askic       — uses synth-core (input) + aski-core (output)
veric       — uses aski-core (input) + sema-core (output)
semac       — uses sema-core types
```

corec is one of only two tools that generate Rust. The other
is semac (via its domainc proc-macro + rsc codegen). Everything
between them is rkyv.

## Rust Style

**No free functions — methods on types always.** All Rust will
eventually be rewritten in aski. `main` is the only exception.

NOTE: current code has free functions (parse_file, parse_paren,
parse_brace, emit_module, etc). These need refactoring to methods.

## VCS

Jujutsu (`jj`) mandatory. Git is storage backend only.
Tests in separate files.
