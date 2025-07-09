# Shell Core Module

This module forms the heart of the shell's functionality, responsible for parsing user input and executing commands.

## Key Responsibilities:

1.  **Parsing:** The `command_executor` contains a sophisticated parser that tokenizes the raw command string. It correctly handles arguments, quotes, and, most importantly, special shell operators like pipes (`|`) and output redirection (`>`).

2.  **Execution:** It manages the execution of both built-in commands (like `cd`, `ls`, `echo`, `grep`) and external system commands.

3.  **Pipeline Handling:** The executor can manage complex command pipelines, chaining multiple commands together by piping the standard output of one command to the standard input of the next.

4.  **Redirection:** It supports redirecting the final output of a command or pipeline to a file.

This setup allows for a powerful and flexible shell experience, mimicking the behavior of standard command-line interfaces.
