# corec — Core Compiler

corec reads .aski domain definitions and generates Rust source
with rkyv derives. It is the bootstrap seed — the generic tool
that both aski-core and sema-core use to produce their rkyv
contract types.

## Usage

```
corec <input-dir> <output-file>
```

Reads all .aski files from input-dir, generates Rust to output-file.

## Role in the Pipeline

```
corec       — .aski → Rust with rkyv derives (this tool)
aski-core   — grammar .aski + corec → Rust rkyv types (askicc↔askic contract)
sema-core   — parse tree .aski + corec → Rust rkyv types (askic↔semac contract)
askicc      — uses aski-core types → rkyv dialect-data-tree
askic       — uses aski-core (input) + sema-core (output)
semac       — uses sema-core types only
```

corec is one of only two tools that generate Rust. The other
is semac. Everything between them is rkyv.

## Rust Style

**No free functions — methods on types always.** All Rust
will eventually be rewritten in aski, which uses methods
(traits + impls). `main` is the only exception.

NOTE: the current code has free functions (parse_file,
parse_paren, parse_brace, emit_module, etc). These need
to be refactored to methods on types.

## VCS

Jujutsu (`jj`) mandatory. Git is storage backend only.
Tests in separate files.
