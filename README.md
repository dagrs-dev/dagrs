# dagrs

本项目是用 Rust 写的 DAG 执行引擎，开发文档请参考：[使用 Rust 编写 DAG 执行引擎](https://openeuler.feishu.cn/docs/doccnVLprAY6vIMv6W1vgfLnfrf)。

## 用法

确保 Rust 编译环境可用（`cargo build`），然后在此文件夹中运行`cargo build --release`，在`target/release/`中获取可执行文件，并将其放入PATH。

本项目面向两个目标群体：

- 普通用户 - 通过 `YAML` 文件对任务进行定义和调度运行。
- 程序员 - 通过实现 `Task Trait` 进行任务的定义和调度运行。


## YAML

此部分是面向普通用户的，即用户并不通过 rust 编程，而是利用 YAML 文件对任务进行描述并调度运行。YAML 文件的一个示例如下：

```yaml
dagrs:
  a:
    name: 任务1
    after: [b]
    from: [b]
    run:
      type: sh
      script: ./test/test.sh
  b:
    name: "任务2"
    run:
      type: deno
      script: print("Hello!")
```

- YAML 文件应该以 `dagrs` 开头。

- `a,b` 是任务的标识符（也可理解为 ID），主要作为标识使用，无具体含义。该字段必须存在且不能重复（否则会覆盖早先定义）。
- `name` 是任务的名称，在后续调度时会输出到 log 中以便用户查看。该字段必须存在，可以重复。
- `after` 是任务的执行顺序，如 `after: [b]` 就表明 `a` 要在 `b` 之后执行。
- `from` 是任务输入值的来源，`from: [b]` 表示 `a` 在开始执行时，会得到 `b` 的执行结果，以字符串的形式输入。
- `run` 是任务的内容定义，包括 `type` 和 `script` 两个子字段。该字段及其子字段必须存在。
  - `type` 是任务的执行方式，当前支持 shell 执行（sh），和 deno 执行（deno）。
  - `script` 是任务的执行内容，可以是具体的命令，也可以是一个文件。



另一个实际涉及输入输出的例子：

```yaml
dagrs:
  a:
    name: "任务1"
    after: [b]
    from: [b]
    run:
      type: sh
      script: echo > ./test/test_value_pass1.txt
  b:
    name: "任务2"
    run:
      type: deno
      script: let a = 1+4; a*2
```
在上面的描述中：
- 任务 `b` 是一个用内置 `deno` 来执行的任务，其返回值显然是 `10`
- 随后 `a` 会被执行，输入值将以字符串的形式拼接到 `script` 的最后面，即以下指令被执行：
  `echo > ./test/test_value_pass1.txt 10`
- 执行结束后，会得到一个文件 `test/test_value_pass1.txt`，其中的内容就会是 `10` 。

**Notice:** 当前 deno 执行只支持有返回值，但输入值并未实现（`deno_core` 的一些接口问题导致）。

**如何运行？**

在编写好 YAML 文件后，可以通过 cli 进行运行：

```bash
$ ./target/release/dagrs --help
dagrs 0.2.0
Command Line input

USAGE:
    dagrs [OPTIONS] <FILE>

ARGS:
    <FILE>    YAML file path

OPTIONS:
    -h, --help                 Print help information
    -l, --logpath <LOGPATH>    Log file path
    -V, --version              Print version information
```

例如运行上述带输入输出的 YAML 的情况：

```bash
$ ./target/release/dagrs test/test_value_pass1.yaml 
08:31:43 [INFO] [Start] -> 任务2 -> 任务1 -> [End]
08:31:43 [INFO] Executing Task[name: 任务2]
cargo:rerun-if-changed=/Users/wyffeiwhe/.cargo/registry/src/mirrors.ustc.edu.cn-61ef6e0cd06fb9b8/deno_core-0.121.0/00_primordials.js
cargo:rerun-if-changed=/Users/wyffeiwhe/.cargo/registry/src/mirrors.ustc.edu.cn-61ef6e0cd06fb9b8/deno_core-0.121.0/01_core.js
cargo:rerun-if-changed=/Users/wyffeiwhe/.cargo/registry/src/mirrors.ustc.edu.cn-61ef6e0cd06fb9b8/deno_core-0.121.0/02_error.js
08:31:43 [INFO] Finish Task[name: 任务2], success: true
08:31:43 [INFO] Executing Task[name: 任务1]
08:31:43 [INFO] Finish Task[name: 任务1], success: true
```

可以看到详细的运行情况输出，同时 log 文件可在 `$HOME/.dagrs/dagrs.log` 下找到（这是默认地址，可以通过 `-l` 选项来自定义。

log 文件记录任务的执行顺序以及执行结果，其内容如下：

```log
$ cat ~/.dagrs/dagrs.log
08:31:43 [INFO] [Start] -> 任务2 -> 任务1 -> [End]
08:31:43 [INFO] Executing Task[name: 任务2]
08:31:43 [INFO] Finish Task[name: 任务2], success: true
08:31:43 [INFO] Executing Task[name: 任务1]
08:31:43 [INFO] Finish Task[name: 任务1], success: true
```



## TaskTrait

Rust Programmer 可以通过实现 `TaskTrait` 来更灵活的定义自己的任务。 `TaskTrait` 的定义如下：

```rust
/// Task Trait.
///
/// Any struct implements this trait can be added into dagrs.
pub trait TaskTrait {
    fn run(&self, input: Inputval, env: EnvVar) -> Retval;
}
```

- `run` 是任务的执行内容，在被调度执行时由 dagrs 调用。
- `input` 是任务的输入，由 `dagrs` 来管理。
- `env` 是整个 `dagrs` 的全局变量，所有任务可见。
- `Retval` 是任务的返回值。


程序员实现的 task struct 需要放到 `TaskWrapper` 中进行使用，并通过其提供的 `exec_after` 和 `input_from` 函数进行依赖设置，具体可见下方的例子。


**如何使用？**

一个[例子](./examples/hello.rs)如下：

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

```

运行的输出如下：

```bash
08:45:24 [INFO] [Start] -> Task 1 -> Task 2 -> [End]
08:45:24 [INFO] Executing Task[name: Task 1]
08:45:24 [INFO] Finish Task[name: Task 1], success: true
08:45:24 [INFO] Executing Task[name: Task 2]
Hello Dagrs!
08:45:24 [INFO] Finish Task[name: Task 2], success: true
```

一些解释：
- `input` 提供一个 `get` 方法，用来获取任务的输入值，其接受一个输入值存放的 `index`。
  - 当定义只有一个输入值来源时（如例子中 `t2` 的输入只来自 `t1`），那么将 `index` 设置为 0 即可。
  - 如果有多个来源，假设 `t3.input_from(&[&t2, &t1])`，那么 index 就是定义任务输入时，任务的排列顺序（`&[&t2, &t1]`，如 `get(0)` 就是拿 `t2` 的返回值，`get(1)` 就是拿 `t1` 的返回值。
- `env` 提供 `get` 和 `set`，[例子参考](./examples/hello_env.rs)。
  - `set` 即设置环境变量，其名称必须是一个字符串。
  - `get` 即获取一个环境变量的值。
- `Retval` 是任务的返回值，提供 `new` 和 `empty` 两个方法。

**Notice:** 整个自定义的任务都应该是 `Sync` 和 `Send` 的，原因是：任务是被放到一个线程中执行调度的。


**如何运行脚本？**

程序员可以通过 `RunScript` 结构来实现脚本的运行（当然也可以直接在代码里自行运行而不通过该结构体），定义如下：

```rust
pub struct RunScript {
    script: String,
    executor: RunType,
}

```

这里：
- `script` 即脚本，可以是命令本身（"echo hello!"），也可以是脚本的路径（"./test/test.sh"）。
- `executor` 是任务的执行方式，`RunType` 是一个 enum 类型：
  ```rust
  pub enum RunType {
      SH,
      DENO,
  }
  ```

`RunScript` 提供了 `exec` 函数：
```rust
pub fn exec(&self, input: Inputval) -> Result<String, DagError> {}
```
如果执行正常，则将结果以 `String` 的形式返回，否则返回一个 `DagError` 的错误类型。

一个简单的[例子](./examples/hello_script.rs)：
```rust
extern crate dagrs;

use dagrs::{DagEngine, EnvVar, Inputval, Retval, TaskTrait, TaskWrapper, init_logger, RunScript, RunType};

struct T {}

impl TaskTrait for T {
    fn run(&self, _input: Inputval, _env: EnvVar) -> Retval {
        let script = RunScript::new("echo 'Hello Dagrs!'", RunType::SH);

        let res = script.exec(None);
        println!("{:?}", res);
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

其执行结果为：
```bash
09:12:48 [INFO] [Start] -> Task -> [End]
09:12:48 [INFO] Executing Task[name: Task]
Ok("Hello Dagrs!\n")
09:12:48 [INFO] Finish Task[name: Task], success: true
```

