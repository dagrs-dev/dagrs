# dagrs

This project is a DAG execution engine written in Rust. For development documentation, please refer to: [Writing a DAG execution engine using Rust](https://openeuler.feishu.cn/docs/doccnVLprAY6vIMv6W1vgfLnfrf).

## Usage

Make sure the Rust compilation environment is available (`cargo build`), then run `cargo build --release` in this folder, fetch the executable in `target/release/` and put it in the PATH.

This project is aimed at two target groups.

- General users - define and schedule tasks by `YAML` files.
- Programmers - define and schedule tasks by  `Task Trait`.

## YAML

This part is catering for the general user who does't use rust, it use YAML files to define and schedule tasks.An example YAML file is as follows:

```yaml
dagrs:
  a:
    name: "Task1"
    after: [b]
    from: [b]
    run:
      type: sh
      script: . /test/test.sh
  b:
    name: "Task2"
    run:
      type: deno
      script: print("Hello!")
```

- The YAML file should start with `dagrs`.

- `a,b` is identifiers for the tasks. This field must be defined and unique (otherwise it will overwrite the earlier definition).
- `name` is the name of the task, it will be output to log file at subsequent scheduling. This field must be defined and it can be the same as other task's name .
- `after` is the order of task execution.For example, `after: [b]` in task `a` means that `a` should be executed after `b`.
- `from` is the input file for the task, `from: [b]` means that `a` will get a string from `b`'s result when it starts execution.
- `run` is the details of the task, including the subfields `type` and `script`. This field and its subfields must exist.
  - `type` is the execution type of the task, it supports shell (sh) and deno (deno) now.
  - `script` is the content of the task, which can be a program or a file.

Another example with input and output:

```yaml
dagrs:
  a:
    name: "Task1"
    after: [b]
    from: [b]
    run:
      type: sh
      script: echo > . /test/test_value_pass1.txt
  b:
    name: "Task2"
    run:
      type: deno
      script: let a = 1+4; a*2
```

In the above example:
- Task `b` execute with the built-in `deno`, apparently it returns `10`
- Then `a` will be executed, and the input value will be spliced to the end of the `script` as a string, i.e. the following command is executed.
  `echo > . /test/test_value_pass1.txt 10`
- At the end of execution, a file `test/test_value_pass1.txt` will be created, and it will have a '10' in it.

**Notice:** The deno is output only now(due to some interface issues about `deno_core`).

**How does it work? **

After writing the YAML file, you can run it with cli:

```bash
$ . /target/release/dagrs --help
dagrs 0.2.0
Command Line input

USAGE:
    dagrs [OPTIONS] <FILE>

ARGS:
    <FILE> YAML file path

OPTIONS:
    -h, --help Print help information
    -l, --logpath <LOGPATH> Log file path
    -V, --version Print version information
```

For example, run with the YAML `test/test_value_pass1.yaml`:

```bash
$ . /target/release/dagrs test/test_value_pass1.yaml 
08:31:43 [INFO] [Start] -> Task2 -> Task1 -> [End]
08:31:43 [INFO] Executing Task[name: Task2]
cargo:rerun-if-changed=/Users/wyffeiwhe/.cargo/registry/src/mirrors.ustc.edu.cn-61ef6e0cd06fb9b8/deno_core-0.121.0/00_primordials. js
cargo:rerun-if-changed=/Users/wyffeiwhe/.cargo/registry/src/mirrors.ustc.edu.cn-61ef6e0cd06fb9b8/deno_core-0.121.0/01_core.js
cargo:rerun-if-changed=/Users/wyffeiwhe/.cargo/registry/src/mirrors.ustc.edu.cn-61ef6e0cd06fb9b8/deno_core-0.121.0/02_error.js
08:31:43 [INFO] Finish Task[name: Task2], success: true
08:31:43 [INFO] Executing Task[name: Task1]
08:31:43 [INFO] Finish Task[name: Task1], success: true
```

You can get the details of the program after you run it, and you can find the log file at `$HOME/.dagrs/dagrs.log` (this is the default address, you can define it with the `-l` option).

The log file records the program's execution sequnence and result, for this example, it just like this:

``log
$ cat ~/.dagrs/dagrs.log
08:31:43 [INFO] [Start] -> Task2 -> Task1 -> [End]
08:31:43 [INFO] Executing Task[name: Task2]
08:31:43 [INFO] Finish Task[name: Task2], success: true
08:31:43 [INFO] Executing Task[name: Task1]
08:31:43 [INFO] Finish Task[name: Task1], success: true
```

## TaskTrait

Rust Programmer can define their own tasks more flexibly by `TaskTrait`. The definition of `TaskTrait` is as follows:

```rust
/// Task Trait.
///
//// Any struct implements this trait can be added into dagrs.
pub trait TaskTrait {
    fn run(&self, input: Inputval, env: EnvVar) -> Retval;
}
```

- `run` is the content of the task, it will be scheduled by dagrs.
- `input` is the input to the task, which is managed by `dagrs`.
- `env` is a global variable for the `dagrs`.
- `Retval` is the return value of the task.

Your task struct needs to be placed in the `TaskWrapper` for use and set dependencies via the `exec_after` and `input_from`, as seen in the example below.

**How to use? **

An [example](. /examples/hello.rs) is as follows:

```rust
extern crate dagrs;

use dagrs::{DagEngine, EnvVar, Inputval, Retval, TaskTrait, TaskWrapper, init_logger};

struct T1 {}

impl TaskTrait for T1 {
    fn run(&self, _input: Inputval, _env: EnvVar) -> Retval {
        let hello_dagrs = String::from("Hello Dagrs!");
        Retval::new(hello_dagrs)
    }
}

struct T2 {}

impl TaskTrait for T2 {
    fn run(&self, mut input: Inputval, _env: EnvVar) -> Retval {
        let val = input.get::<String>(0).unwrap();
        println!("{}", val);
        Retval::empty()
    }
}

fn main() {
    // Use dagrs provided logger
    init_logger(None);

    let t1 = TaskWrapper::new(T1{}, "Task 1");
    let mut t2 = TaskWrapper::new(T2{}, "Task 2");
    let mut dagrs = DagEngine::new();

    // Set up dependencies
    t2.exec_after(&[&t1]);
    t2.input_from(&[&t1]);

    dagrs.add_tasks(vec![t1, t2]);
    assert!(dagrs.run().unwrap())
}
```''

The output is as follows.

```bash
08:45:24 [INFO] [Start] -> Task 1 -> Task 2 -> [End]
08:45:24 [INFO] Executing Task[name: Task 1]
08:45:24 [INFO] Finishing Task[name: Task 1], success: true
08:45:24 [INFO] Executing Task[name: Task 2]
Hello Dagrs!
08:45:24 [INFO] Finish Task[name: Task 2], success: true
```

Some explanations.
- `input` provides a `get` method to get the task's input values, which takes an `index` where the input values are stored.
  - Please set `index` to 0 when only one source of input values is defined (as in the example where the input to `t2` comes only from `t1`).
  - If there are multiple sources, such as `t3.input_from(&[&t2, &t1])`, then index is the order of the input order (`&[&t2, &t1]`, for example, `get(0)` takes the return value of `t2`, `get(1)` takes the return value of `t1`).
- `env` provides `get` and `set`, [example reference](. /examples/hello_env.rs).
  - `set` sets the environment variable, whose name must be a string.
  - `get` is to get the value of an environment variable.
- `Retval` is the return value of the task, provides `new` and `empty` methods.

**Notice:** The whole custom task should be `Sync` and `Send` for one reason: the task is put into a thread to perform scheduling.

**How to run the script? **

You can run the script through the `RunScript` struct (or, of course, directly in the code itself without going through the struct), defined as follows:

```rust
pub struct RunScript {
    script: String,
    executor: RunType,
}
```

Here:
- `script` is the script, either the command itself ("echo hello!"), also can be the path to the script (". /test/test.sh").
- `executor` is the way the task execut, and `RunType` is an enum type.
  ``rust
  pub enum RunType {
      SH,
      DENO,
  }
  ```

`RunScript` provides the `exec` function.

```rust
pub fn exec(&self, input: Inputval) -> Result<String, DagError> {}
```

It will returns the result as `String` if it executes correctly, otherwise returns a `DagError` error type.

A simple [example](. /examples/hello_script.rs):

```rust
extern crate dagrs;

use dagrs::{DagEngine, EnvVar, Inputval, Retval, TaskTrait, TaskWrapper, init_logger, RunScript, RunType};

struct T {}

impl TaskTrait for T {
    fn run(&self, _input: Inputval, _env: EnvVar) -> Retval {
        let script = RunScript::new("echo 'Hello Dagrs!'", RunType::SH);

        let res = script.exec(None);
        println!("{:?}" , res);
        Retval::empty()
    }
}

fn main() {
    // Use dagrs provided logger
    init_logger(None);

    let t = TaskWrapper::new(T{}, "Task");
    let mut dagrs = DagEngine::new();

    dagrs.add_tasks(vec![t]);
    assert!(dagrs.run().unwrap())
}
```

The result is:

```bash
09:12:48 [INFO] [Start] -> Task -> [End]
09:12:48 [INFO] Executing Task[name: Task]
Ok("Hello Dagrs!\n")
09:12:48 [INFO] Finish Task[name: Task], success: true
ðŸ™' ðŸ™'

### How to contribute?

This project enforces the [DCO](https://developercertificate.org).

Contributors sign-off that they adhere to these requirements by adding a Signed-off-by line to commit messages.

```bash
This is my commit message

Signed-off-by: Random J Developer <random@developer.example.org>
```

Git even has a -s command line option to append this automatically to your commit message:

```bash
$ git commit -s -m 'This is my commit message'
```

### License

Freighter is licensed under this Licensed:

* MIT LICENSE ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)

### Acknowledgements
