# `xhacklight`

My experience with Linux on laptops is that the default backlight controls are very coarse -
they only work in 10% increments, and a 10% brightness at night is a lot.
The usual suggestion for fixing that is to use `xbacklight` - if you're trying to do that,
[this answer](https://askubuntu.com/a/1060843) worked for me.
However in my case, specifying the intel driver in `xorg.conf` broke things,
and this is the result of replacing `xbacklight`.

The task is simple - read and write the `/sys/class/backlight/intel_backlight/brightness` file.
While a script could easily do that, it wouldn't allow me to set the SUID bit and would require `sudo`. So, Rust it was!

## Usage

    xhacklight [=N|+N|-N|inc|dec]

The `[=+-]N` variants work as expected, but on a scale from 0 to 60000,
which is the native scale of the `brightness` file - on my system, at least.
`inc` and `dec` try to be smart by using a somewhat logarithmic scale:
values are more fine-grained on the darker side, more coarse on the brighter side.

Without arguments, the current brightness as a number between 0 and 60000 is printed.