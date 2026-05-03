# Han.Foo

The new 🦀 [BLAZINGLY FAST](https://rust-lang.org) 🦀 and ❄️ [REPRODUCIBLE](https://nixos.org/) ❄️ flashcard webapp for memorizing han/foo to point conversions, 100% organic home grown and with options!

## Options

### Kiriage

Enable [kiriage mangan](https://riichi.wiki/Japanese_mahjong_scoring_rules#Kiriage_mangan), in which anything above 3 han/60 fu and 4 han/30 fu becomes mangan.

### Modes

Modes effect how common different scores are by editing the raw states pulled from [tenhou houou logs](https://github.com/Apricot-S/houou-logs).

#### Chaos

Every score is equally common.

#### Normal

A balance between realism and practice, making common scores a bit less likely.

#### Realistic

Directly uses raw tenhou stats.

#### Unlucky

For emulating your actual wins.

## Running

Thanks to be the power of ❄️ REPRODUCIBILITY ❄️, all you need is to be a masochist and thus have [nix](https://nixos.org) installed!

### Development Server

```
git clone git@github.com:skyethepinkcat/han_foo
cd han_foo
nix run
```

### Release package

```
git clone git@github.com:skyethepinkcat/han_foo
cd han_foo
nix build
```

### Without Nix

I could tell you how but I don't feel like it. Just look up trunk.

### With docker

This is for masochists only get that container shit outta here we use nix in this house.

## TODO

- [ ] Consider all limit wins to be the same regardless of fu
- [ ] Factor tsumo and dealer into odds
- [ ] Allow user to make individual scores less likely to appear

## FAQ

### Why did you write this in Rust

I've decided I dislike javascript so I choose an infinitely worse solution.

### Why Do You Have "Cargo Mommy" installed in the nix dev shell and justfile...

Reading the card explains the card.
