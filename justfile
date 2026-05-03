build:
  cargo mommy build
test:
  cargo mommy test

webbuild:
  trunk build

serve: webbuild
  trunk serve

release:
  trunk build --release --public-url './'
  tar -czf han_foo.tgz -C dist .
deploy: release
  scp han_foo.tgz asticassia:~/
