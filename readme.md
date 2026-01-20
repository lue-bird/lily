small programming language that compiles to rust, heavily inspired by [elm](https://elm-lang.org/).
Just experimentation.

## hello world

```still
\:uninitialized_or {}:_ -> Standard_out_write "hello, world\n"
```

## echo in loop

```still
\:uninitialized_or str:state_or_uninitialized ->
  let state
        case state_or_uninitialized of
        Uninitialized -> ""
        Initialized :str:initialized -> initialized
  Io_batch
    [ Standard_out_write
        (str_flatten [ ansi_clear_screen, state, "\nType a sentence to echo: " ])
    , Standard_in_read_line (\:str:line -> line)
    ]

ansi_clear_screen "\u{001B}c"
```

## cons-list map

```still
type stack A = Empty | Cons { head A, tail stack A }

stack_map \{ change :\A -> B:element_change, stack :stack A:stack } ->
  case stack of
  :stack A:Empty -> :stack B:Empty
  :stack A:Cons { head :A:head, tail :stack A:tail } ->
    :stack B:Cons
      { head element_change head
      , tail stack_map { change element_change, stack tail }
      }
```

## TODO
- remove record extension type
- allow any record in record update
- remove parse_still_documentation_comment_block_str
- change comment system to `Expression::WithComment` and `Pattern::WithComment` and `Type::WithComment` (each meaning it is prefixed by `--`) and _always_ preserve line-spread of original range! Then, remove all &comments parameters
- remove `Pattern::Parenthesized`
- split call and variant expression syntax tree
- require types for lambda variable and ignored
- `:type:expression`/`:type:pattern` for (extra) type-checking, like `:option int:None` or `add \:int:n -> :int:some expression`
- type checking
- `still build`
- small standard library (`str`, `vec`, `int` (i32), `dec` (f32), ?`order`, ?`char`(unicode_scalar/rune), `int_to_str`, `dec_to_str`, `int/dec_add`, `int/dec_multiply`, `dec_power`, `str_compare`, `int_compare`, `dec_compare`, ...)
- simple io (`standard_in_read_line`, `standard_out_write`, ?`type uninitialized_or Initialized = Uninitialized | Initialized Initialized`)

## considering
- anonymous choice types
- introduce `nat` type and require regular ints to be prefixed with `+`/`-`
- find better function call syntax that makes it easy to unwrap the last argument
- somehow find better string literal syntax
- add or pattern `| first | second | third` (potentially allow `:overall:| A | B | C` (where the inner variant patterns don't need a type) specifically for variant)

To use, [install rust](https://rust-lang.org/tools/install/) and
```bash
cargo +nightly install --git https://github.com/lue-bird/still
```
Then point your editor to `still lsp`, see also [specific setups](#editor-setups).

## editor setups
feel free to contribute, as I only use vscodium

### vscode-like
#### pre-built
1. download https://github.com/lue-bird/still/blob/main/vscode/still-0.0.1.vsix
2. open the command bar at the top and select: `>Extensions: Install from VSIX`
#### build from source
1. clone this repo
2. open `vscode/`
3. run `npm run package` to create the `.vsix`
4. open the command bar at the top and select: `>Extensions: Install from VSIX`
#### server only
There is no built-in language server bridge as far as I know but you can install an extension like [vscode-generic-lsp-proxy](https://github.com/mjmorales/vscode-generic-lsp-proxy) that will work for any language server.
Then add a `.vscode/lsp-proxy.json` like
```json
[
  {
    "languageId": "still",
    "command": "still",
    "fileExtensions": [
      ".still"
    ]
  }
]
```

### helix
write to `~/.config/helix/languages.toml`:
```toml
[language-server.still]
command = "still lsp"
[[language]]
name = "still"
scope = "source.still"
injection-regex = "still"
file-types = ["still"]
indent = { tab-width = 2, unit = "  " }
language-servers = [ "still" ]
auto-format = true
```

## setup for developing
Rebuild the project with
```bash
cargo build
```
Then point your editor to the created `???/target/debug/still lsp`.

### log of failed optimizations
- switching to mimalloc, ~>25% faster (really nice) at the cost of 25% more memory consumption.
  Might be worth for some people but I'm already worried about our memory footprint!
- `declarations.shrink_to_fit();` saves around 0.6% of memory at the cost of a bit of speed
- upgrading `lto` to `"thin"` to `"fat"` both improve runtime speed by ~13% compared to the default (and reduce binary size) but increase build time by about 30% (default to thin) and 15% (thin to fat).
  As this prolongs installation and prevents people from quickly trying it, the default is kept.
  If this language server get distributed as a binary or people end up using this language server a lot, this `"thin"` might become a reasonable trade-off.

### optimizations to try
- reparse incrementally (somewhat easy to implement but somehow it's for me at least pretty much fast enough already without? More data points welcome)
- switch to `position_encoding: Some(lsp_types::PositionEncodingKind::UTF8)`. This makes source edits and parsing easier and faster at the cost of compatibility with lsp clients below version 3.17.0. Is that acceptable? (leaning towards yes).
- if memory consumptions turns out to be a problem, stop storing the source in memory
  and request full file content on each change (potentially only for dependencies).
  This adds complexity and is slower so only if necessary.
- in syntax tree, use separate range type for single-line tokens like keywords, symbols, names etc to save on memory consumption
- in syntax tree, use `Box<[]>` instead of `Vec` for common nodes like call arguments
