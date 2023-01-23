# joyn

Join input files. Create a thread for each input file that reads a line, and write a line each time any thread reads it.

**Keywords**: command line utility, pipe, parallel execution

## Usage

```sh
joyn file1 file2 ...
```

Read input files and their content lines and output them.
The order of lines in the output is "interleaved". That is, unlike the `cat` command, lines in the file specified earlier in the command line may be output later.

However, the expected use of `joyn` is to invoke the processes that run command lines and merge their output lines.
In the case of `bash` shell, this usage can be accomplished with a command line such as:

```sh
joyn <(command line 1) <(command line 2) ...
```

## Samples

&rarr; [Difference between command substitution joined with cat and joined with joyn](samples/command-subsutitution-joyn.md)

## Release history

#### 0.2.0

* feat: new option --summary to print LOC of each input file on exit

#### 0.1.0

* First release