## Bringing Paths into Scope with the `use` Keyword

Having to write out the paths to call functions can feel inconvenient and
repetitive. In Listing 7-7, whether we chose the absolute or relative path to
the `add_to_waitlist` function, every time we wanted to call `add_to_waitlist`
we had to specify `front_of_house` and `hosting` too. Fortunately, there’s a
way to simplify this process: We can create a shortcut to a path with the `use`
keyword once and then use the shorter name everywhere else in the scope.

In Listing 7-11, we bring the `crate::front_of_house::hosting` module into the
scope of the `eat_at_restaurant` function so that we only have to specify
`hosting::add_to_waitlist` to call the `add_to_waitlist` function in
`eat_at_restaurant`.

<Listing number="7-11" file-name="src/lib.rs" caption="Bringing a module into scope with `use`">

```rust,noplayground,test_harness
mod front_of_house {
    pub mod hosting {
        pub fn add_to_waitlist() {}
    }
}

use crate::front_of_house::hosting;

pub fn eat_at_restaurant() {
    hosting::add_to_waitlist();
}
```

</Listing>

Adding `use` and a path in a scope is similar to creating a symbolic link in
the filesystem. By adding `use crate::front_of_house::hosting` in the crate
root, `hosting` is now a valid name in that scope, just as though the `hosting`
module had been defined in the crate root. Paths brought into scope with `use`
also check privacy, like any other paths.

Note that `use` only creates the shortcut for the particular scope in which the
`use` occurs. Listing 7-12 moves the `eat_at_restaurant` function into a new
child module named `customer`, which is then a different scope than the `use`
statement, so the function body won’t compile.

<Listing number="7-12" file-name="src/lib.rs" caption="A `use` statement only applies in the scope it’s in.">

```rust,noplayground,test_harness,does_not_compile,ignore
mod front_of_house {
    pub mod hosting {
        pub fn add_to_waitlist() {}
    }
}

use crate::front_of_house::hosting;

mod customer {
    pub fn eat_at_restaurant() {
        hosting::add_to_waitlist();
    }
}
```

</Listing>

The compiler error shows that the shortcut no longer applies within the
`customer` module:

```console
$ cargo build
   Compiling restaurant v0.1.0 (file:///projects/restaurant)
error[E0433]: failed to resolve: use of unresolved module or unlinked crate `hosting`
  --> src/lib.rs:11:9
|
11 |         hosting::add_to_waitlist();
| ^^^^^^^ use of unresolved module or unlinked crate `hosting`
|
   = help: if you wanted to use a crate named `hosting`, use `cargo add hosting` to add it to your `Cargo.toml`
help: consider importing this module through its public re-export
|
10 +     use crate::hosting;
|

warning: unused import: `crate::front_of_house::hosting`
 --> src/lib.rs:7:5
|
7 | use crate::front_of_house::hosting;
| ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
|
  = note: `#[warn(unused_imports)]` on by default

For more information about this error, try `rustc --explain E0433`.
warning: `restaurant` (lib) generated 1 warning
error: could not compile `restaurant` (lib) due to 1 previous error; 1 warning emitted
```

Notice there’s also a warning that the `use` is no longer used in its scope! To
fix this problem, move the `use` within the `customer` module too, or reference
the shortcut in the parent module with `super::hosting` within the child
`customer` module.

### Creating Idiomatic `use` Paths

In Listing 7-11, you might have wondered why we specified `use
crate::front_of_house::hosting` and then called `hosting::add_to_waitlist` in
`eat_at_restaurant`, rather than specifying the `use` path all the way out to
the `add_to_waitlist` function to achieve the same result, as in Listing 7-13.

<Listing number="7-13" file-name="src/lib.rs" caption="Bringing the `add_to_waitlist` function into scope with `use`, which is unidiomatic">

```rust,noplayground,test_harness
mod front_of_house {
    pub mod hosting {
        pub fn add_to_waitlist() {}
    }
}

use crate::front_of_house::hosting::add_to_waitlist;

pub fn eat_at_restaurant() {
    add_to_waitlist();
}
```

</Listing>

Although both Listing 7-11 and Listing 7-13 accomplish the same task, Listing
7-11 is the idiomatic way to bring a function into scope with `use`. Bringing
the function’s parent module into scope with `use` means we have to specify the
parent module when calling the function. Specifying the parent module when
calling the function makes it clear that the function isn’t locally defined
while still minimizing repetition of the full path. The code in Listing 7-13 is
unclear as to where `add_to_waitlist` is defined.

On the other hand, when bringing in structs, enums, and other items with `use`,
it’s idiomatic to specify the full path. Listing 7-14 shows the idiomatic way
to bring the standard library’s `HashMap` struct into the scope of a binary
crate.

<Listing number="7-14" file-name="src/main.rs" caption="Bringing `HashMap` into scope in an idiomatic way">

```rust
use std::collections::HashMap;

fn main() {
    let mut map = HashMap::new();
    map.insert(1, 2);
}
```

</Listing>

There’s no strong reason behind this idiom: It’s just the convention that has
emerged, and folks have gotten used to reading and writing Rust code this way.

The exception to this idiom is if we’re bringing two items with the same name
into scope with `use` statements, because Rust doesn’t allow that. Listing 7-15
shows how to bring two `Result` types into scope that have the same name but
different parent modules, and how to refer to them.

<Listing number="7-15" file-name="src/lib.rs" caption="Bringing two types with the same name into the same scope requires using their parent modules.">

```rust,noplayground
use std::fmt;
use std::io;

fn function1() -> fmt::Result {
    // --snip--
}

fn function2() -> io::Result<()> {
    // --snip--
}
```

</Listing>

As you can see, using the parent modules distinguishes the two `Result` types.
If instead we specified `use std::fmt::Result` and `use std::io::Result`, we’d
have two `Result` types in the same scope, and Rust wouldn’t know which one we
meant when we used `Result`.

### Providing New Names with the `as` Keyword

There’s another solution to the problem of bringing two types of the same name
into the same scope with `use`: After the path, we can specify `as` and a new
local name, or _alias_, for the type. Listing 7-16 shows another way to write
the code in Listing 7-15 by renaming one of the two `Result` types using `as`.

<Listing number="7-16" file-name="src/lib.rs" caption="Renaming a type when it’s brought into scope with the `as` keyword">

```rust,noplayground
use std::fmt::Result;
use std::io::Result as IoResult;

fn function1() -> Result {
    // --snip--
}

fn function2() -> IoResult<()> {
    // --snip--
}
```

</Listing>

In the second `use` statement, we chose the new name `IoResult` for the
`std::io::Result` type, which won’t conflict with the `Result` from `std::fmt`
that we’ve also brought into scope. Listing 7-15 and Listing 7-16 are
considered idiomatic, so the choice is up to you!

### Re-exporting Names with `pub use`

When we bring a name into scope with the `use` keyword, the name is private to
the scope into which we imported it. To enable code outside that scope to refer
to that name as if it had been defined in that scope, we can combine `pub` and
`use`. This technique is called _re-exporting_ because we’re bringing an item
into scope but also making that item available for others to bring into their
scope.

Listing 7-17 shows the code in Listing 7-11 with `use` in the root module
changed to `pub use`.

<Listing number="7-17" file-name="src/lib.rs" caption="Making a name available for any code to use from a new scope with `pub use`">

```rust,noplayground,test_harness
mod front_of_house {
    pub mod hosting {
        pub fn add_to_waitlist() {}
    }
}

pub use crate::front_of_house::hosting;

pub fn eat_at_restaurant() {
    hosting::add_to_waitlist();
}
```

</Listing>

Before this change, external code would have to call the `add_to_waitlist`
function by using the path
`restaurant::front_of_house::hosting::add_to_waitlist()`, which also would have
required the `front_of_house` module to be marked as `pub`. Now that this `pub
use` has re-exported the `hosting` module from the root module, external code
can use the path `restaurant::hosting::add_to_waitlist()` instead.

Re-exporting is useful when the internal structure of your code is different
from how programmers calling your code would think about the domain. For
example, in this restaurant metaphor, the people running the restaurant think
about “front of house” and “back of house.” But customers visiting a restaurant
probably won’t think about the parts of the restaurant in those terms. With `pub
use`, we can write our code with one structure but expose a different structure.
Doing so makes our library well organized for programmers working on the library
and programmers calling the library. We’ll look at another example of `pub use`
and how it affects your crate’s documentation in [“Exporting a Convenient Public
API”][ch14-pub-use]<!-- ignore --> in Chapter 14.

### Using External Packages

In Chapter 2, we programmed a guessing game project that used an external
package called `rand` to get random numbers. To use `rand` in our project, we
added this line to _Cargo.toml_:

<!-- When updating the version of `rand` used, also update the version of
`rand` used in these files so they all match:
* ch02-00-guessing-game-tutorial.md
* ch14-03-cargo-workspaces.md
-->

<Listing file-name="Cargo.toml">

```toml
// File not found: /home/shiro/Desktop/Projects/LearnTUI/listings/ch02-guessing-game-tutorial/listing-02-02/Cargo.toml:9
```

</Listing>

Adding `rand` as a dependency in _Cargo.toml_ tells Cargo to download the
`rand` package and any dependencies from [crates.io](https://crates.io/) and
make `rand` available to our project.

Then, to bring `rand` definitions into the scope of our package, we added a
`use` line starting with the name of the crate, `rand`, and listed the items we
wanted to bring into scope. Recall that in [“Generating a Random
Number”][rand]<!-- ignore --> in Chapter 2, we brought the `Rng` trait into
scope and called the `rand::thread_rng` function:

```rust,ignore
use rand::Rng;

fn main() {
    let secret_number = rand::thread_rng().gen_range(1..=100);
}
```

Members of the Rust community have made many packages available at
[crates.io](https://crates.io/), and pulling any of them into your package
involves these same steps: listing them in your package’s _Cargo.toml_ file and
using `use` to bring items from their crates into scope.

Note that the standard `std` library is also a crate that’s external to our
package. Because the standard library is shipped with the Rust language, we
don’t need to change _Cargo.toml_ to include `std`. But we do need to refer to
it with `use` to bring items from there into our package’s scope. For example,
with `HashMap` we would use this line:

```rust
use std::collections::HashMap;
```

This is an absolute path starting with `std`, the name of the standard library
crate.

<!-- Old headings. Do not remove or links may break. -->

<a id="using-nested-paths-to-clean-up-large-use-lists"></a>

### Using Nested Paths to Clean Up `use` Lists

If we’re using multiple items defined in the same crate or same module, listing
each item on its own line can take up a lot of vertical space in our files. For
example, these two `use` statements we had in the guessing game in Listing 2-4
bring items from `std` into scope:

<Listing file-name="src/main.rs">

```rust,ignore
// --snip--
use std::cmp::Ordering;
use std::io;
// --snip--
```

</Listing>

Instead, we can use nested paths to bring the same items into scope in one
line. We do this by specifying the common part of the path, followed by two
colons, and then curly brackets around a list of the parts of the paths that
differ, as shown in Listing 7-18.

<Listing number="7-18" file-name="src/main.rs" caption="Specifying a nested path to bring multiple items with the same prefix into scope">

```rust,ignore
// --snip--
use std::{cmp::Ordering, io};
// --snip--
```

</Listing>

In bigger programs, bringing many items into scope from the same crate or
module using nested paths can reduce the number of separate `use` statements
needed by a lot!

We can use a nested path at any level in a path, which is useful when combining
two `use` statements that share a subpath. For example, Listing 7-19 shows two
`use` statements: one that brings `std::io` into scope and one that brings
`std::io::Write` into scope.

<Listing number="7-19" file-name="src/lib.rs" caption="Two `use` statements where one is a subpath of the other">

```rust,noplayground
use std::io;
use std::io::Write;
```

</Listing>

The common part of these two paths is `std::io`, and that’s the complete first
path. To merge these two paths into one `use` statement, we can use `self` in
the nested path, as shown in Listing 7-20.

<Listing number="7-20" file-name="src/lib.rs" caption="Combining the paths in Listing 7-19 into one `use` statement">

```rust,noplayground
use std::io::{self, Write};
```

</Listing>

This line brings `std::io` and `std::io::Write` into scope.

<!-- Old headings. Do not remove or links may break. -->

<a id="the-glob-operator"></a>

### Importing Items with the Glob Operator

If we want to bring _all_ public items defined in a path into scope, we can
specify that path followed by the `*` glob operator:

```rust
use std::collections::*;
```

This `use` statement brings all public items defined in `std::collections` into
the current scope. Be careful when using the glob operator! Glob can make it
harder to tell what names are in scope and where a name used in your program
was defined. Additionally, if the dependency changes its definitions, what
you’ve imported changes as well, which may lead to compiler errors when you
upgrade the dependency if the dependency adds a definition with the same name
as a definition of yours in the same scope, for example.

The glob operator is often used when testing to bring everything under test into
the `tests` module; we’ll talk about that in [“How to Write
Tests”][writing-tests]<!-- ignore --> in Chapter 11. The glob operator is also
sometimes used as part of the prelude pattern: See [the standard library
documentation](../std/prelude/index.html#other-preludes)<!-- ignore --> for more
information on that pattern.

[ch14-pub-use]: ch14-02-publishing-to-crates-io.html#exporting-a-convenient-public-api
[rand]: ch02-00-guessing-game-tutorial.html#generating-a-random-number
[writing-tests]: ch11-01-writing-tests.html#how-to-write-tests
