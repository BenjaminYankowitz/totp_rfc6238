* This is a one time code generator (the same thing as an app like google authenticator) written in rust.
* Follows the rfc6238 spec.
* It has no dependencies outside of std.
* `cargo run` will prompt for a base32 encoded key, and it will then print current 6 digit code. 
* Assumes inital time is unix epoch and code changes every 30 seconds.
* Works for signing into google acounts (I have not tested other websites).