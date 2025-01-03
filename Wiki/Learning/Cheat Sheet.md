Below is a comprehensive Rust cheat sheet that covers the essentials of the language. This is by no means exhaustive, but it should serve as a handy reference for commonly used syntax, data types, and fundamental concepts.

---

## 1. Setup & Project Structure

### Installing Rust

- **Recommended**: Use [rustup](https://rustup.rs/) (manages multiple Rust versions).

  ```bash
  curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh
  ```

### Updating / Checking Version

```bash
rustup update
rustc --version
cargo --version
```

### Creating & Building a Project

```bash
# Create a new cargo project
cargo new my_project

# Move into the project directory
cd my_project

# Build (debug mode)
cargo build

# Run
cargo run

# Build + run tests
cargo test

# Build documentation
cargo doc --open
```

Project structure:

```bash
my_project
├── Cargo.toml       # Project manifest (dependencies, metadata, etc.)
├── Cargo.lock       # Automatic lock file for dependencies
└── src
    └── main.rs      # Entry point for a binary crate
```

---

## 2. Hello, World!

```rust
fn main() {
    println!("Hello, world!");
}
```

---

## 3. Variables & Mutability

### Declaring Variables

```rust
let x = 5;                // Immutable
let mut y = 10;           // Mutable (can be changed later)

// Type annotations are optional, but can be explicit
let z: i32 = 20;
```

### Shadowing

```rust
let x = 5;
let x = x + 1;  // shadows previous x
println!("{}", x);  // prints 6
```

---

## 4. Scalar & Compound Types

### Scalar Types

- **Integer**: `i8`, `i16`, `i32`, `i64`, `i128`, `isize` (signed)
- **Integer**: `u8`, `u16`, `u32`, `u64`, `u128`, `usize` (unsigned)
- **Floating-point**: `f32`, `f64`
- **Boolean**: `bool`
- **Character**: `char` (4 bytes, Unicode scalar value)

### Compound Types

- **Tuples**: Fixed size, can contain different types.

  ```rust
  let tup: (i32, f64, u8) = (500, 6.4, 1);
  let (a, b, c) = tup;  // destructuring
  println!("{}", tup.1); // 6.4
  ```

- **Arrays**: Fixed length, all elements must be the same type.

  ```rust
  let arr = [1, 2, 3, 4, 5];
  println!("{}", arr[0]);
  ```

---

## 5. Functions

### Basic Function Declaration

```rust
fn main() {
    let result = add(5, 10);
    println!("Result = {}", result);
}

fn add(x: i32, y: i32) -> i32 {
    x + y  // Implicit return (no semicolon)
}
```

### Statements vs. Expressions

- **Statements**: Perform an action, do not return a value.
- **Expressions**: Evaluate to a value (e.g. `x + 1`, blocks with no `;`).

---

## 6. Control Flow

### `if` / `else if` / `else`

```rust
let num = 5;

if num > 10 {
    println!("Greater than 10");
} else if num == 10 {
    println!("Equal to 10");
} else {
    println!("Less than 10");
}
```

### Using `if` in a `let` Statement

```rust
let condition = true;
let number = if condition { 5 } else { 6 };
```

### Loops

1. **`loop`**

   ```rust
   let mut counter = 0;
   loop {
       counter += 1;
       if counter == 5 {
           break;
       }
   }
   ```

2. **`while`**

   ```rust
   let mut count = 0;
   while count < 5 {
       println!("count = {}", count);
       count += 1;
   }
   ```

3. **`for`**

   ```rust
   let arr = [10, 20, 30];
   for val in arr.iter() {
       println!("{}", val);
   }

   // or
   for i in 0..5 {
       println!("{}", i); // prints 0 through 4
   }
   ```

---

## 7. Ownership & Borrowing

### Ownership Rules

1. Each value in Rust has a single owner.
2. When the owner goes out of scope, the value is dropped.
3. Only one owner at a time (unless the ownership is transferred or a reference is borrowed).

### Move Semantics

```rust
let s1 = String::from("hello");
let s2 = s1;    // s1 is moved to s2, s1 is no longer valid
// println!("{}", s1); // Error! s1 is invalidated
```

### Clone

```rust
let s1 = String::from("hello");
let s2 = s1.clone(); // Performs a deep copy
println!("{}", s1);  // Now valid
```

### Borrowing

```rust
fn main() {
    let s1 = String::from("hello");
    let len = calculate_length(&s1); // &s1 is an immutable reference
    println!("Length of '{}': {}", s1, len);
}

fn calculate_length(s: &String) -> usize {
    s.len()
}
```

- **Mutable Borrow**:

  ```rust
  fn main() {
      let mut s = String::from("hello");
      change(&mut s);
      println!("{}", s); // "hello world"
  }

  fn change(some_string: &mut String) {
      some_string.push_str(" world");
  }
  ```

- **Rules**:
  - At most one mutable reference to a particular piece of data in a scope, *or* any number of immutable references (but not both at the same time).
  - References must always be valid.

---

## 8. Slices

### String Slice

```rust
let s = String::from("hello");
let hello = &s[0..5];  // or &s[..5]
```

### Array Slice

```rust
let arr = [1, 2, 3, 4, 5];
let slice = &arr[1..3]; // [2, 3]
```

---

## 9. Structs

### Defining & Instantiating

```rust
struct User {
    username: String,
    email: String,
    sign_in_count: u64,
    active: bool,
}

fn main() {
    let user1 = User {
        email: String::from("user@example.com"),
        username: String::from("username123"),
        sign_in_count: 1,
        active: true,
    };
}
```

### Update Syntax

```rust
let user2 = User {
    email: String::from("another@example.com"),
    ..user1  // fill remaining fields from user1
};
```

### Tuple Structs

```rust
struct Color(i32, i32, i32);

let black = Color(0, 0, 0);
```

---

## 10. Enums & Pattern Matching

### Defining Enums

```rust
enum IpAddr {
    V4(String),
    V6(String),
}

let home = IpAddr::V4(String::from("127.0.0.1"));
let loopback = IpAddr::V6(String::from("::1"));
```

### `match` Expressions

```rust
enum Coin {
    Penny,
    Nickel,
    Dime,
    Quarter,
}

fn value_in_cents(coin: Coin) -> u8 {
    match coin {
        Coin::Penny   => 1,
        Coin::Nickel  => 5,
        Coin::Dime    => 10,
        Coin::Quarter => 25,
    }
}
```

### `if let`

```rust
let config_max = Some(3u8);
if let Some(max) = config_max {
    println!("Max is {}", max);
}
```

---

## 11. Packages, Crates & Modules

### Packages & Crates

- **Package**: A set of crates.
- **Crate**: A compilation unit in Rust (binary or library).

### Modules

- Organize code within a crate and control scope/visibility.

```rust
// main.rs
mod front_of_house;

fn main() {
    front_of_house::hosting::add_to_waitlist();
}
```

```rust
// front_of_house.rs
pub mod hosting {
    pub fn add_to_waitlist() {
        println!("Added to waitlist.");
    }
}
```

---

## 12. Generics

### Generic Functions

```rust
fn largest<T: PartialOrd>(list: &[T]) -> &T {
    let mut max = &list[0];
    for item in list {
        if item > max {
            max = item;
        }
    }
    max
}
```

### Generic Structs

```rust
struct Point<T> {
    x: T,
    y: T,
}

let integer_point = Point { x: 5, y: 10 };
let float_point = Point { x: 1.0, y: 4.0 };
```

---

## 13. Traits & Trait Bounds

### Defining a Trait

```rust
pub trait Summary {
    fn summarize(&self) -> String;
}
```

### Implementing a Trait

```rust
pub struct NewsArticle {
    pub headline: String,
    pub author: String,
    pub content: String,
}

impl Summary for NewsArticle {
    fn summarize(&self) -> String {
        format!("{}, by {}", self.headline, self.author)
    }
}
```

### Trait Bounds on Functions

```rust
fn notify(item: &impl Summary) {
    println!("Breaking news! {}", item.summarize());
}

// Equivalent form with trait bounds:
fn notify<T: Summary>(item: &T) {
    println!("Breaking news! {}", item.summarize());
}
```

---

## 14. Error Handling

### `Result<T, E>`

```rust
use std::fs::File;
use std::io::ErrorKind;

fn main() {
    let f = File::open("hello.txt");
    let f = match f {
        Ok(file) => file,
        Err(ref error) if error.kind() == ErrorKind::NotFound => {
            match File::create("hello.txt") {
                Ok(fc) => fc,
                Err(e) => panic!("Error creating file: {:?}", e),
            }
        },
        Err(e) => panic!("Error opening file: {:?}", e),
    };
}
```

### `?` Operator

```rust
fn read_username_from_file() -> Result<String, std::io::Error> {
    let mut f = File::open("hello.txt")?;
    let mut s = String::new();
    f.read_to_string(&mut s)?;
    Ok(s)
}
```

---

## 15. Smart Pointers & Common Types

### Box

- Used for storing data on the heap.

```rust
let b = Box::new(5);
println!("b = {}", b);
```

### `Rc<T>`

- Reference counting pointer enabling multiple ownership.

```rust
use std::rc::Rc;

let a = Rc::new(5);
let b = Rc::clone(&a);
println!("{}", Rc::strong_count(&a)); // 2 owners
```

### `RefCell<T>`

- Enforces borrowing rules at runtime rather than compile time.

---

## 16. Concurrency

### Threads

```rust
use std::thread;
use std::time::Duration;

fn main() {
    let handle = thread::spawn(|| {
        for i in 1..5 {
            println!("Hi number {} from the spawned thread!", i);
            thread::sleep(Duration::from_millis(50));
        }
    });

    for i in 1..5 {
        println!("Hi number {} from the main thread!", i);
        thread::sleep(Duration::from_millis(50));
    }

    handle.join().unwrap();
}
```

### Channels

```rust
use std::sync::mpsc;
use std::thread;

fn main() {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let val = String::from("hello");
        tx.send(val).unwrap();
    });

    let received = rx.recv().unwrap();
    println!("Got: {}", received);
}
```

### `Mutex`

```rust
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];

    for _ in 0..10 {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            let mut num = counter.lock().unwrap();
            *num += 1;
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Result = {}", *counter.lock().unwrap());
}
```

---

## 17. Common Collections

- **Vectors**: `Vec<T>`

  ```rust
  let mut v = Vec::new();
  v.push(1);
  v.push(2);
  let third = &v[2];
  ```

- **Strings**: `String` & `str`

  ```rust
  let mut s = String::new();
  s.push_str("Hello");
  s.push(' ');
  s.push_str("World");
  ```

- **Hash Maps**: `HashMap<K, V>`

  ```rust
  use std::collections::HashMap;

  let mut scores = HashMap::new();
  scores.insert(String::from("Blue"), 10);
  scores.insert(String::from("Red"), 50);
  ```

---

## 18. Macros

### Declarative Macros (`macro_rules!`)

```rust
macro_rules! say_hello {
    () => {
        println!("Hello!");
    };
}

fn main() {
    say_hello!();
}
```

### Procedural Macros

- Custom derive, attribute-like, function-like macros (more advanced usage).

---

## 19. Useful Cargo Commands

```bash
cargo build       # Compile
cargo build --release
cargo run         # Compile and run
cargo test        # Run tests
cargo fmt         # Format code
cargo clippy      # Lint code
cargo doc --open  # Generate and open documentation
```

---

## 20. Useful Links & References

- **Official Book**: [The Rust Programming Language](https://doc.rust-lang.org/book/)
- **Rust Reference**: [Rust Reference](https://doc.rust-lang.org/reference/)
- **Rust by Example**: [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- **Crate Registry**: [crates.io](https://crates.io/)

---

### Closing Tips

1. **Understand Ownership & Borrowing**: This is central to Rust’s safety guarantees.
2. **Leverage Pattern Matching**: `match` is a powerful alternative to if-else chains.
3. **Check Out the Ecosystem**: Cargo, crates.io, and Rust’s standard library are full of helpful utilities.
4. **Practice**: Writing Rust code regularly will help you internalize the concepts quickly.

Enjoy Rust! You now have a handy reference for the core language and ecosystem. Keep exploring and happy coding!
