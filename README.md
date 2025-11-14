# kubeswitch
## !! This is not a production ready repository !!

No seriously, I have never written anything in Rust until now. This might eat your `kubeconfig` or something. But it seems to work okay for me. It does not respect the `KUBECONFIG` env var or having multiple configs. I did this to learn a little about Rust and have something ever so slightly faster than `kubectx`/`kubens`.

## Installing
Run `mise run install`

## Usage
`kubeswitch <context|namespace> [context name|namespace name]`

If you omit the context or namespace name, it will use [skim](https://github.com/skim-rs/skim) to let you find the target context/namespace.

I use the following in my `fish` config to make it easier to work with:
```
abbr -a ktx 'kubeswitch context'
abbr -a kns 'kubeswitch namespace'
```

An alias in bash will accomplish the same thing as the above abbreviations.
