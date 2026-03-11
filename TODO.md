# TODO WB-Rust

Dokumen ini berisi daftar pekerjaan untuk rewrite interpreter Wibu ke Rust.

## 0. Landasan Proyek
- [ ] Tetapkan tujuan fitur v1 (syntax, tipe data, scope, control flow).
- [ ] Definisikan non-goals (fitur yang ditunda).
- [ ] Buat spesifikasi grammar awal (BNF/EBNF ringan).
- [ ] Tentukan format error & diagnostic.

## 1. Arsitektur Workspace
- [ ] Validasi struktur crate dan dependensi antar-crate.
- [ ] Tetapkan aturan layering (lexer -> parser -> ast -> runtime).
- [ ] Definisikan public API minimal untuk tiap crate.

## 2. Lexer (`wb-lexer`)
- [x] Definisikan token set final (keyword, operator, literal, delimiter).
- [x] Implementasi lexer streaming dengan posisi (line/col/offset).
- [x] Support string literal dan escape.
- [x] Support komentar single-line dan block.
- [ ] Unit tests untuk tokenisasi dasar.

## 3. AST (`wb-ast`)
- [x] Lengkapi node AST (expr, stmt, decl, block, function, call).
- [ ] Tambahkan span/source range untuk setiap node.
- [x] Struktur tipe data literal & operator.

## 4. Parser (`wb-parser`)
- [x] Implement parser Pratt / recursive descent.
- [x] Implement parsing statement: var/let, if/else, loop, function.
- [ ] Error recovery sederhana (sync tokens).
- [ ] Unit tests untuk parsing ekspresi dan statement.

## 5. Runtime (`wb-runtime`)
- [x] Definisikan value model (number, string, bool, nil, function).
- [x] Implement environment/scope (stacked env).
- [x] Implement built-in function minimal (print/baka).
- [ ] Unit tests untuk environment dan evaluation.

## 6. Core Interpreter (`wb-core`)
- [x] Wire pipeline lex -> parse -> eval.
- [x] Expose API interpret/execute file.
- [x] Centralize error handling & diagnostics.

## 7. Diagnostics (`wb-diagnostics`)
- [ ] Standarisasi error types (lexer, parser, runtime).
- [ ] Implement pretty error message (line highlight).
- [ ] Pastikan error dapat dipropagasi antar-crate.

## 8. CLI (`wibu`)
- [x] Tambahkan arg parsing (run file, repl, version).
- [x] Implement REPL sederhana.
- [x] Integrasi output error/diagnostic.

## 9. Testing
- [ ] Port test fixture dari `WibuCPP-Legacy/test`.
- [ ] Golden tests untuk output interpreter.
- [ ] Tambahkan unit tests per crate.

## 10. Tooling & Quality
- [ ] Tambahkan `cargo fmt` dan `cargo clippy`.
- [ ] Setup CI minimal (lint + test).
- [ ] Tambahkan `deny.toml` (opsional, jika mau strict).

## 11. Dokumentasi
- [ ] Update `readme.md` untuk versi Rust.
- [ ] Tulis spec bahasa (docs/grammar.md).
- [ ] Contoh program dan output di `examples/`.

## 12. Roadmap Versi
- [x] v0.1: lexer + parser + evaluasi ekspresi dasar.
- [x] v0.2: control flow (if/loop), function sederhana.
- [ ] v1.0: runtime stabil + error report lengkap + CLI.
