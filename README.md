# unit-scrub

Log lines, config files, and API responses write byte sizes however they
feel like it: `3.5gb`, `1024 KiB`, `200bytes`, `4 MB`. Every tool that has
to read these back in ends up with its own half-working regex. This is a
small parser and formatter that turns that mess into one consistent form,
and when it can't, tells you exactly where the input broke.

The point of this project is the error messages. A parser that just
returns `Err("bad input")` is barely more useful than a crash. This one
tracks line and column as it scans, so a broken entry in line 40 of a
2000-line file points straight at line 40.

Byte sizes are the CLI's job today. Durations (`90s`, `1h30m`, `2 days`)
share the same scanner and error type and are parseable at the library
level now (see below); wiring them into the command-line document format
is next.

## Usage

Feed it a file, one size per line:

```
$ cat sizes.txt
128 MB
1024 KiB
12 gigs
4kib

$ cargo run --quiet -- sizes.txt
128.00 MB
1.00 MiB
line 3, column 4: 'gigs' is not a recognized unit
4.00 KiB
```

Or pipe input on stdin:

```
$ echo "3.5gb" | cargo run --quiet
3.50 GB
```

A bad line doesn't stop the rest of the file from being processed — you
get every error in one pass, each with its own position, instead of
stopping at the first one.

## What counts as a unit

Decimal: `B`/`byte`/`bytes`, `K`/`KB`/`kilobyte(s)`, `M`/`MB`/`megabyte(s)`,
`G`/`GB`/`gigabyte(s)`, `T`/`TB`/`terabyte(s)`, `P`/`PB`/`petabyte(s)`.

Binary: `Ki`/`KiB`/`kibibyte(s)`, `Mi`/`MiB`/`mebibyte(s)`,
`Gi`/`GiB`/`gibibyte(s)`, `Ti`/`TiB`/`tebibyte(s)`, `Pi`/`PiB`/`pebibyte(s)`.

Matching is case-insensitive (`GB`, `gb`, `Gb` all work). Decimal and
binary units are kept distinct on output — a gigabyte and a gibibyte are
different numbers of bytes, and normalizing shouldn't quietly change which
one you meant.

## As a library

```rust
use unit_scrub::parse_document;

let results = parse_document("128 MB\n12 gigs\n");
for result in results {
    match result {
        Ok(size) => println!("{}", size),
        Err(err) => eprintln!("{}", err), // "line 2, column 4: 'gigs' is not a recognized unit"
    }
}
```

Durations parse the same way, as a library type, via `FromStr`:

```rust
use unit_scrub::Duration;

let d: Duration = "1h30m".parse().unwrap();
assert_eq!(d.seconds(), 5_400.0);
println!("{}", d); // "1h 30m"
```

A duration is one or more `<number><unit>` components, run together or
separated by spaces: `90s`, `1h30m`, `2 days`, `1.5h` all parse. Recognized
units are `ms`/`millisecond(s)`, `s`/`sec(s)`/`second(s)`,
`m`/`min(s)`/`minute(s)`, `h`/`hr(s)`/`hour(s)`, `d`/`day(s)`, and
`w`/`wk(s)`/`week(s)`, matched case-insensitively. Formatting breaks the
total back down into the largest units that divide it evenly, so `90s`
prints back as `1m 30s`.

## Building

Standard library only, no dependencies to fetch:

```
cargo build
cargo test
```
