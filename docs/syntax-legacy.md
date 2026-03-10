# Desain Syntax & Keyword Wibu-Lang (Legacy-Style)

Dokumen ini mendefinisikan desain syntax/keyword Wibu-Lang agar konsisten dengan versi legacy (WibuCPP).

## 1. Keyword Inti
- `bikin` : deklarasi variabel
- `baka`  : output/print
- `kalo`  : kondisi `if`
- `moshi` : deklarasi function
- `bentar`: loop `while`
- `balikin`: return dari function

## 2. Literal & Tipe Dasar
- Number: `123`, `3.14`
- String: `"teks"`
- Boolean: `true`, `false`
- Nil: `nil`

## 3. Operator
- Aritmatika: `+ - * / %`
- Perbandingan: `== != < <= > >=`
- Logika: `&& || !`
- Assignment: `=`

## 4. Struktur Statement

### Deklarasi Variabel
```wibu
bikin nama = "Wibu"
bikin umur = 17
```

### Output
```wibu
baka "Nama: " + nama
```

### If
```wibu
kalo online {
    baka "online!"
}
```

### While
```wibu
bentar (n < 5) {
    baka n
    n = n + 1
}
```

### Function
```wibu
moshi hai(nama) {
    baka "hai " + nama
}

hai("ramen")
```

### Return
```wibu
moshi tambah(a, b) {
    balikin a + b
}
```

## 5. Komentar
- Single-line: `// komentar`
- Multi-line: `/* komentar */`

## 6. Aturan Umum
- Blok memakai `{ }`.
- Statement dipisah newline atau `;` (opsional).
- Identifer: huruf/underscore di awal, lalu alnum/underscore.

