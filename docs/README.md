# WibuScript (WB-Rust)

Dokumentasi ringkas bahasa WibuScript versi WB-Rust.

**Ringkas**
WibuScript adalah bahasa scripting ringan dengan sintaks sederhana. Blok bisa memakai `:` + indentasi atau `{ }`.

**Menjalankan Program**
Jalankan file `.wb` dengan CLI:

```bash
cargo run -p wibu -- path/ke/file.wb
```

REPL:

```bash
cargo run -p wibu -- --repl
```

**Sintaks Dasar**
Komentar:

```wibu
// komentar satu baris
/* komentar
   multi baris */
```

Pernyataan diakhiri newline atau `;`.

Blok dengan `:` dan indentasi:

```wibu
kalo true:
    baka("jalan")
```

Blok dengan `{ }`:

```wibu
kalo true {
    baka("jalan")
}
```

**Kata Kunci**
| Keyword | Arti |
| --- | --- |
| `bikin` | deklarasi variabel |
| `fun` | deklarasi fungsi |
| `kalo` | if |
| `ato` | else / else if |
| `bentar` | while |
| `ulang` | for-each array |
| `di` | pasangan untuk `ulang` |
| `balikin` | return |
| `baka` | print (statement khusus) |
| `lanjut` | continue |
| `berhenti` | break |
| `butuh` | impor modul |
| `ekspor` | ekspor (placeholder, evaluasi ekspresi) |
| `true` | boolean benar |
| `false` | boolean salah |
| `nil`, `kosong` | nilai kosong |
| `nani`, `yamete`, `sugoi` | keyword cadangan (belum aktif) |

**Tipe Data**
- Angka: `123`, `3.14`.
- String: `"teks"` atau `'teks'` dengan escape `\n`, `\t`, `\"`, `\'`.
- Boolean: `true` dan `false`.
- Nil: `nil`.
- Array: `[1, 2, 3]`.

Nil dan boolean `false` dianggap tidak benar, sisanya `true`.

**Operator**
- Aritmatika: `+ - * / %`.
- Perbandingan: `== != < <= > >=`.
- Logika: `&& || !`.
- Assignment: `=`.

**Variabel**
Deklarasi dan assign ulang:

```wibu
bikin nama = "Wibu"
nama = "WB-Rust"
```

**Kontrol Alur**
If / else if / else:

```wibu
kalo nilai >= 80:
    baka("A")
ato kalo nilai >= 70:
    baka("B")
ato:
    baka("C")
```

While:

```wibu
bikin i = 0
bentar i < 3:
    bakaf("loop {}", i)
    i = i + 1
```

While dengan inisialisasi di header:

```wibu
bentar bikin i = 0; i < 3:
    bakaf("loop {}", i)
    i++
```

Increment/decrement:

```wibu
i++
++i
i--
--i
```

For-each array:

```wibu
bikin angka = [1, 2, 3]
ulang item di angka:
    baka(item)
```

`lanjut` dan `berhenti` berlaku di `bentar` dan `ulang`.

**Fungsi**
Deklarasi dan return:

```wibu
fun tambah(a, b):
    balikin a + b

bikin hasil = tambah(2, 3)
```

**Array dan Index**
Index array memakai angka integer berbasis 0.

```wibu
bikin items = ["a", "b", "c"]
baka(items[0])
```

**Modul**
`butuh` menerima string nama modul:

```wibu
butuh "./modul"
```

Resolusi modul:
- Jika path adalah folder, akan mencoba `__init__.wb` di dalamnya.
- Jika tanpa ekstensi, akan mencoba menambah `.wb`.
- Prefix `wb:` diarahkan ke folder standar `~/.wb/lib/wb`.

`ekspor` saat ini belum mengembalikan nilai modul, tetapi ekspresinya tetap dievaluasi.

**Built-in**
| Fungsi | Arity | Catatan |
| --- | --- | --- |
| `baka(...)` | variadik | print dengan newline |
| `bakaf(format, ...)` | variadik | format dengan `{}` |
| `format(format, ...)` | variadik | menghasilkan string |
| `input(prompt?)` | variadik | baca input; argumen pertama jadi prompt |
| `panjang(value)` | 1 | panjang string atau array |
| `tipe(value)` | 1 | `angka`, `teks`, `boolean`, `nil`, `array`, `fungsi` |
| `angka(value)` | 1 | konversi ke angka |
| `teks(value)` | 1 | konversi ke string |
| `stdout(...)` | variadik | print tanpa newline ke stdout |
| `stderr(...)` | variadik | print tanpa newline ke stderr |
| `baca_file(path)` | 1 | baca file -> string |
| `tulis_file(path, data)` | 2 | tulis file (overwrite) |
| `append_file(path, data)` | 2 | append ke file |
| `cwd()` | 0 | current working directory |
| `env_get(key)` | 1 | baca environment variable |
| `env_set(key, value)` | 2 | set environment variable |
| `sqrt(x)` | 1 | akar kuadrat |
| `sin(x)` | 1 | sinus |
| `cos(x)` | 1 | cosinus |
| `tan(x)` | 1 | tangen |
| `pow(a, b)` | 2 | pangkat |
| `abs(x)` | 1 | absolut |
| `floor(x)` | 1 | pembulatan ke bawah |
| `ceil(x)` | 1 | pembulatan ke atas |
| `round(x)` | 1 | pembulatan terdekat |
| `regex_cocok(pattern, text)` | 2 | cocokkan regex -> boolean |
| `regex_cari(pattern, text)` | 2 | hasil match pertama atau `nil` |
| `regex_ganti(pattern, text, repl)` | 3 | ganti regex -> string |
| `tcp_connect(host, port)` | 2 | konek TCP -> handle socket |
| `tcp_listen(host, port)` | 2 | listen TCP -> handle listener |
| `tcp_accept(listener)` | 1 | terima koneksi -> handle socket |
| `tcp_send(socket, text)` | 2 | kirim data TCP -> jumlah byte |
| `tcp_recv(socket, max_bytes)` | 2 | terima data TCP -> string atau `nil` saat EOF |
| `tcp_local_addr(socket)` | 1 | alamat lokal TCP (`ip:port`) |
| `tcp_close(socket)` | 1 | tutup socket/listener TCP |
| `udp_bind(host, port)` | 2 | bind UDP -> handle socket |
| `udp_send(socket, host, port, text)` | 4 | kirim data UDP -> jumlah byte |
| `udp_recv(socket, max_bytes)` | 2 | terima data UDP -> `[text, host, port]` |
| `udp_local_addr(socket)` | 1 | alamat lokal UDP (`ip:port`) |
| `udp_close(socket)` | 1 | tutup socket UDP |

**Contoh**
Lihat contoh lengkap di folder `examples/`.

Contoh singkat:

```wibu
bikin nama = "Ming Lee"
bakaf("Halo {}, selamat datang!", nama)
```
