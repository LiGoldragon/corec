# corec — Core Compiler

corec reads `.core` domain definitions and generates Rust source
with rkyv derives. Bootstrap seed — the generic tool that
synth-core, aski-core, and veri-core use to produce their rkyv
contract types.

## ⚠️ STATUS: PARTIALLY STALE (v0.20 grammar not yet supported)

corec builds and works against the LEGACY .core file format where
the first line is a fake-enum TOC:

```aski
(Trait TraitDecl TraitImpl Method Signature NamedMethod AssociatedTypeBinding)

{TraitDecl ...}
```

**What's stale:**

v0.20 added an optional proper module header to `core/Root.synth`
in askicc:

```aski
?#Module#(@ModuleName <:aski:Module>)
// *#Enum#( @EnumName <Enum> )
...
```

This allows `.core` files to declare a module name AND imports
from other `.core` files. corec's internal parser (`src/parse.rs`)
does NOT understand this new form yet — it only parses the legacy
fake-enum-TOC.

**How to fix (when time comes):**

1. Extend corec's `parse.rs` to accept the v0.20 module header form:
   - Optional `(Name [Import1 Imports...])` before the type
     definitions
   - Existing fake-enum form should still work (backward compat)
2. Add `imports: Vec<Import>` field to corec's `Module` struct.
3. Emit corresponding Rust `use` statements (or validate references)
   when imports are present.
4. Update aski-core/core/*.core, synth-core/core/dialect.core,
   veri-core/source/program.core to use the new module header form
   (retiring the fake-enum TOC).

This is a paired change — corec update and .core file migration
land together to avoid breakage.

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
veri-core   — veric-output .core + corec → Rust rkyv types (veric↔semac contract; renamed from sema-core 2026-04-18)
askicc      — uses synth-core types → dsls.rkyv (domain-data-tree, all 5 DSLs)
askic       — uses synth-core (input) + aski-core (output)
veric       — uses aski-core (input) + veri-core (output)
semac       — uses veri-core types
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
