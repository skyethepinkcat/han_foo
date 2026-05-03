build:
  cargo mommy build
test:
  cargo mommy test

webbuild:
  trunk build

serve: webbuild
  trunk serve
