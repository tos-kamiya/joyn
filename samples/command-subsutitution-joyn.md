# Difference between command substitution joined with cat and joined with joyn

The script `stepped-countup.sh` is a counter that counts up in step increments.

The arguments are the label of the counter, the time to be measured by the counter and the interval for display.

```sh
$ ./stepped-countup.sh a 10 2
a 0
a 2
a 4
a 6
a 8
a done.
```

Running two counters simultaneously and displaying the counts will result in an interleaved display, as shown below.

```sh
$ ./stepped-countup.sh a 10 2 & ./stepped-countup.sh b 9 3
[1] 192222
b 0
a 0
a 2
b 3
a 4
b 6
a 6
a 8
b done.
a done.
```

When run two counters using bash process substitution and cat command, the output is the same as when the counters are run sequentially in order as the arguments of cat command.

```sh
$ cat <(./stepped-countup.sh a 10 2) <(./stepped-countup.sh b 9 3)
a 0
a 2
a 4
a 6
a 8
a done.
b 0
b 3
b 6
b done.
```

On the other hand, when the joyn command is used, the output is similar to when two counters are run in parallel.

```sh
$ joyn <(./stepped-countup.sh a 10 2) <(./stepped-countup.sh b 9 3)
a 0
b 0
a 2
b 3
a 4
b 6
a 6
a 8
b done.
a done.
```

The command lines with `cat` and `joyn` shown above also differ in the timing of printing each line of the output. Please check by actually running the command lines.