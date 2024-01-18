# Asky

> Ansi + ask + yes = Asky

Good looking prompts for the terminal (and maybe beyond).

## Usage

First of all, this is a library, so you need to add this to your project:

```bash
cargo add asky
```

Then, you can [see the documentation](https://docs.rs/asky/).

## Demos

### Confirm

![Confirm prompt gif demo](demos/confirm.gif)

<details>
<summary>Code:</summary>

```rust
use asky::Confirm;

fn main() -> std::io::Result<()> {
    if Confirm::new("Do you like coffe?").prompt()? {
        println!("Great, me too!");
    }

    // ...

    Ok(())
}

```

</details>

### Toggle

![Toggle prompt gif demo](demos/toggle.gif)

<details>
<summary>Code:</summary>

```rust
use asky::Toggle;

fn main() -> std::io::Result<()> {
    let tabs = Toggle::new("Which is better?", ["Tabs", "Spaces"]).prompt()?;
    println!("Great choice");

    // ...

    Ok(())
}
```

</details>

### Text

![Text prompt gif demo](demos/text.gif)

<details>
<summary>Code:</summary>

```rust
use asky::Text;

fn main() -> std::io::Result<()> {
    let color = Text::new("What's your favorite color?").prompt()?;
    println!("{color} is a beautiful color");

    // ...

    Ok(())
}
```

</details>

### Number

![Number prompt gif demo](demos/number.gif)

<details>
<summary>Code:</summary>

```rust
use asky::Number;

fn main() -> std::io::Result<()> {
    if let Ok(age) = Number::<u8>::new("How old are you?").prompt()? {
        if age <= 60 {
            println!("Pretty young");
        }
    }

    // ...

    Ok(())
}
```

</details>

### Password

![Password prompt gif demo](demos/password.gif)

<details>
<summary>Code:</summary>

```rust
use asky::Password;

fn main() -> std::io::Result<()> {
    let password = Password::new("What's your IG password?").prompt()?;

    if password.len() >= 1 {
        println!("Ultra secure!");
    }

    // ...

    Ok(())
}
```

</details>

### Select

![Select prompt gif demo](demos/select.gif)

<details>
<summary>Code:</summary>

```rust
use asky::Select;

fn main() -> std::io::Result<()> {
    let choice = Select::new("Choose number", 1..=30).prompt()?;
    println!("{choice}, Interesting choice");

    // ...

    Ok(())
}

```

</details>

### MultiSelect

![Multi select prompt gif demo](demos/multi_select.gif)

<details>
<summary>Code:</summary>

```rust
use asky::MultiSelect;

fn main() -> std::io::Result<()> {
    let opts = ["Dog", "Cat", "Fish", "Bird", "Other"];
    let choices = MultiSelect::new("What kind of pets do you have?", opts).prompt()?;

    if choices.len() > 2 {
        println!("So you love pets");
    }

    // ...

    Ok(())
}

```

</details>

## Bevy Support

There is experimental support for [bevy](https://bevyengine.org), a rust game engine.

### Bevy Examples

One can use Asky with bevy in two ways: without async or with async, i.e., the
hard way and the easy way.

### The Hard Way: Without Async

If asky is used directly without async, one mediates the interaction through
bevy's usual mechanisms: systems and components. See the
[bevy/confirm.rs](examples/bevy/confirm.rs) example.

The difficulty with this approach is like any naturally asynchronous problem, it
is tricky to handle waiting to receive input in synchronous code. In the
following example code, we setup asky `Confirm` then await a change of its state
from `AskyState::Reading` to `AskyState::Complete`. We then mark it `Handled`
with our own marker component so that we don't process completed AskyNodes more
than once.

Here is an excerpt of from the `bevy-confirm` example
[code](examples/bevy/confirm.rs).

``` rust
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // ...
    let confirm: Confirm<'static> = Confirm::new("Do you like coffee?");
    let node = NodeBundle {
        style: Style {
            flex_direction: FlexDirection::Column,
            ..default()
        },
        ..default()
    };
    commands.spawn(node.clone()).with_children(|parent| {
        parent
            .spawn(node)
            .insert(AskyNode(confirm, AskyState::Reading));
    });
}

fn response(
    mut commands: Commands,
    mut query: Query<(Entity, &AskyNode<Confirm<'static>>), Without<Handled>>,
) {
    for (entity, prompt) in query.iter_mut() {
        match prompt.1 {
            AskyState::Complete => {
                let response = match prompt.0.value() {
                    Ok(yes) => {
                        if yes {
                            "Great, me too."
                        } else {
                            "Oh, ok."
                        }
                    }
                    Err(_) => "Uh oh, had a problem.",
                };

                // Add our message.
                let child = commands
                    .spawn(NodeBundle { ..default() })
                    .insert(AskyNode(Message::new(response), AskyState::Reading))
                    .id();
                // Mark this entity as handled.
                commands
                    .entity(entity)
                    .push_children(&[child])
                    .insert(Handled);
            }
            _ => {}
        }
    }
}
```

#### The Easy Way: With Async

```sh
cargo run --features bevy --example bevy-confirm-async
```

![Movie of bevy-confirm-async example](https://github.com/shanecelis/asky/assets/54390/8b51c3ac-b69f-436b-baa8-b9361baa2bfc)

With async an `Asky` `SystemParam` is available in bevy after loading
`AskyPlugin`. This will allow you to prompt the user with whatever Asky query
you like and await its response asynchronously. As the excerpt from
[bevy/confirm-async.rs](examples/bevy/confirm-async.rs) shows below, it is much
more compact.

```rust
fn ask_name(mut asky: Asky, query: Query<Entity, Added<Page>>) -> Option<impl Future<Output = ()>> {
    query.get_single().ok().map(|id| async move {
        if let Ok(first_name) = asky.prompt(Text::new("What's your first name? "), id).await {
            if let Ok(last_name) = asky.prompt(Text::new("What's your last name? "), id).await {
                let _ = asky
                    .prompt(
                        Message::new(format!("Hello, {first_name} {last_name}!")),
                        id,
                    )
                    .await;
            }
        } else {
            eprintln!("Got err in ask name.");
        }
    })
}

```

There are limitations to what can go inside the `Future`. Because it is `async
move` you can't take references to much of anything you normally have access to
in a system. No `Commands`, no components. Why not? Because this future
may be around for a while. And if we have shared or exclusive access to a
component, we'd prevent other systems from accessing it.

#### For fun

Since it is easy to write these asky queries using async, here's an example just
for fun. Note: It does not actually do anything with emails or passwords.

```sh
cargo run --features bevy --example bevy-funny
```
![Movie of bevy-funny example](https://github.com/shanecelis/asky/assets/54390/8b51c3ac-b69f-436b-baa8-b9361baa2bfc)

## Mentions

Inspired by:

- [Prompts](https://www.npmjs.com/package/prompts) - Lightweight, beautiful and user-friendly interactive prompts
- [Astro](https://astro.build/) - All-in-one web framework with a beautiful command line tool
- [Gum](https://github.com/charmbracelet/gum) - A tool for glamorous shell scripts

Alternatives:

- [Dialoguer](https://github.com/console-rs/dialoguer) - A command line prompting library.
- [Inquire](https://github.com/mikaelmello/inquire) - A library for building interactive prompts on terminals.
- [Requestty](https://github.com/Lutetium-Vanadium/requestty) - An easy-to-use collection of interactive cli prompts.

---

License: [MIT](LICENSE)
