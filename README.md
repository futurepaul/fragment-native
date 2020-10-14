# fragment

This is an app for searching and creating notes. It uses [ripgrep](https://github.com/BurntSushi/ripgrep) for search and [druid](https://github.com/linebender/druid) for ui.

![fragment-native-screenshot](https://user-images.githubusercontent.com/543668/96023815-fbbad580-0e20-11eb-956f-47860abc7fef.png)

If you'd like to check it out:

1. clone this repo
2. install GTK if you're on Linux
3. `cargo run -- --path ~/path/to/folder/of/plaintext/files`

Now you can search notes and open them in your default editor. Hit enter to create a new note with your search string as the title.

Inspired by [notational velocity](http://notational.net/). I've also made [a version of fragment using electron](https://github.com/futurepaul/fragment).
