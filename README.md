# sherlock_dict_rs
Here's hoping this works!
Much like the https://github.com/Skxxtz/sherlock-wiki, this is meant to tap into dict.org

```
git clone https://github.com/MoonBurst/sherlock_dict_rs.git
cd sherlock_dict_rs
cargo build --release
Then place it into your .config/sherlock/scripts/
```

Apply the following to your sherlock's fallback.json and run it with "define $words"

	{
    "name": "Dictionary Lookup",
    "alias": "define",
    "type": "bulk_text",
    "on_return": "next",
    "async": true,
    "args": {
        "icon": "dictionary",
        "exec": "/home/$USER/.config/sherlock/scripts/sherlock-dictionary",
        "exec-args": "{keyword}"
    },
    "priority": 0,
    "shortcut": false
	}

Frankly, I'm not much of a programmer, this is my attempt to make something work though! There's PROBABLY a better way to do all of this.
