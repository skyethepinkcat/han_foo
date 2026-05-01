build:
  cargo mommy build
test:
  cargo mommy test
pack:
  wasm-pack build --target web
serve: pack
  miniserve .
