# Open Design Questions

This file tracks unresolved design issues that should stay explicit without polluting the stable architecture and contract documents.

1. **Scope and path canonicalization.** What is the canonical path syntax and escaping policy for VCD escaped identifiers and other unusual names across formats?
2. **Warnings as codes versus free text.** Should warnings remain free-form strings, or should wavepeek eventually introduce stable warning codes for promote/suppress flows?
3. **Value radix options.** Should a future release add `--radix` (for example `hex`, `bin`, `dec`, `auto`), and if so what default policy should replace or complement Verilog-literal output?
4. **Schema evolution policy.** Should the project keep one canonical schema forever, or eventually split machine contracts into per-command schemas?
5. **Signal metadata schema.** Which JSON fields beyond `kind` and `width` should be part of the stable `signal` machine contract across dump formats?
6. **GHW support scope.** If GHW support is added after MVP, what acceptance criteria and priority should gate that work?
