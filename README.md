# kube_switch
## !! This is not a production ready repository !!

No seriously, I have never written anything in Rust until now. This might eat your `kubeconfig` or something. But it seems to work okay for me.

## Installing
Run `mise run install`

## Usage
`kube_switch <context|namespace> [context name|namespace name]`

If you omit the context or namespace name, it will use [skim](https://github.com/skim-rs/skim) to let you find the target context/namespace.

I use the following in my `fish` config to make it easier to work with:
```
abbr -a ktx 'kube_switch context'
abbr -a kns 'kube_switch namespace'
```
