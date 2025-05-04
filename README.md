# Makatime

![GitHub last commit](https://img.shields.io/github/last-commit/kallydev/makatime?style=flat-square)
![GitHub commit activity](https://img.shields.io/github/commit-activity/m/kallydev/makatime?style=flat-square)
![GitHub license](https://img.shields.io/github/license/kallydev/makatime?style=flat-square)

A real-time service that tracks what GitHub users are currently doing.

## Installation

> [!IMPORTANT]  
> Currently only supports macOS.

```shell
# Install the client from GitHub
cargo install --git https://github.com/kallydev/makatime makatime-client

# Start the client
makatime-client --token "github_pat_{YOUR_GITHUB_PERSONAL_TOKEN}"
```

Then you can add the badge url to your profile in the following format:

### Markdown

`![](https://makatime.kallydev.workers.dev/{USERNAME}.svg)`

### HTML

`<img src="https://makatime.kallydev.workers.dev/{USERNAME}.svg">`

## Preview

![](https://makatime.kallydev.workers.dev/kallydev.svg)

## License

Licensed under the [MIT](LICENSE) license.
