# Line-oriented handling

The `joyn` handles input and output in a line-oriented manner.

The `char-by-char.sh` script is intended to simulate the case where a line does not fit into the pipe buffer and is not output at once, but is output as small substrings.

When executed, one number is output every second and a line takes 10 seconds.

```sh
$ ./char-by-char.sh a
a-1 0123456789
a-2 0123456789
a-3 0123456789
```

When the two scripts are run at once, the output of both will be mixed character by character.

```sh
$  ./char-by-char.sh a & ./char-by-char.sh b
[1] 262199
a-1 b-1 00112233445566778899
b-2 0
a-2 0112233445566778899

b-3 0a-3 0112233445566778899
```

You can retrieve them line by line by running the bash script with command substitution and the `cat` command. However, the output of the second script will not be handled until the output of the first script is complete.

```sh
$ cat <(./char-by-char.sh a) <(./char-by-char.sh b)
a-1 0123456789
a-2 0123456789
a-3 0123456789
b-1 0123456789
b-2 0123456789
b-3 0123456789
```

On the other hand, the `joyn` command can be used to handle the output of the two scripts line-by-line, in the order in which each line is generated.

```sh
$  joyn <(./char-by-char.sh a) <(./char-by-char.sh b)
a-1 0123456789
b-1 0123456789
b-2 0123456789
a-2 0123456789
b-3 0123456789
a-3 0123456789
```
