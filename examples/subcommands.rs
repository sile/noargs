fn main() -> noargs::Result<()> {
    let mut args = noargs::raw_args();
    args.metadata_mut().app_name = env!("CARGO_PKG_NAME");
    args.metadata_mut().app_description = env!("CARGO_PKG_DESCRIPTION");

    if noargs::VERSION_FLAG.take(&mut args).is_present() {
        println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    noargs::HELP_FLAG.take_help(&mut args);

    // In real applications, split each `run_*` command into separate modules/files.
    //
    // `args` is moved into the handler that matches, so each arm leaves with `return`.
    // The handler calls `finish` itself: see below for why that order matters.
    if noargs::cmd("hello")
        .doc("Print a greeting")
        .take(&mut args)
        .is_present()
    {
        return run_hello(args);
    }
    if noargs::cmd("sum")
        .doc("Add two integers")
        .take(&mut args)
        .is_present()
    {
        return run_sum(args);
    }
    if noargs::cmd("echo")
        .doc("Print a message")
        .take(&mut args)
        .is_present()
    {
        return run_echo(args);
    }

    // No subcommand: top-level help, and the top-level argument checks.
    if let Some(help) = args.finish()? {
        print!("{help}");
    }

    Ok(())
}

/// Prints a greeting.
///
/// The handler owns `args`, so it can call `finish` before doing any work. `finish`
/// is the one place that both builds the help text and checks the arguments, so the
/// `Some(help)` branch below is the handler's only decision, and nothing is printed
/// before it.
fn run_hello(mut args: noargs::RawArgs) -> noargs::Result<()> {
    let loud = noargs::flag("loud")
        .short('l')
        .doc("Print greeting in upper case")
        .take(&mut args)
        .is_present();
    let name: String = noargs::arg("<NAME>")
        .doc("Name to greet")
        .example("Alice")
        .take(&mut args)
        .then(|a| a.value().parse())?;

    // Help text for `hello`, and the `hello` argument checks. Both happen here,
    // before anything is printed.
    if let Some(help) = args.finish()? {
        print!("{help}");
        return Ok(());
    }

    let message = format!("Hello, {name}!");
    if loud {
        println!("{}", message.to_uppercase());
    } else {
        println!("{message}");
    }
    Ok(())
}

/// Adds two integers.
fn run_sum(mut args: noargs::RawArgs) -> noargs::Result<()> {
    let repeat: usize = noargs::opt("repeat")
        .short('r')
        .ty("N")
        .doc("Print result multiple times")
        .default("1")
        .take(&mut args)
        .then(|o| o.value().parse())?;
    let left: i64 = noargs::arg("<LEFT>")
        .doc("Left operand")
        .example("3")
        .take(&mut args)
        .then(|a| a.value().parse())?;
    let right: i64 = noargs::arg("<RIGHT>")
        .doc("Right operand")
        .example("4")
        .take(&mut args)
        .then(|a| a.value().parse())?;

    if let Some(help) = args.finish()? {
        print!("{help}");
        return Ok(());
    }

    let total = left + right;
    for _ in 0..repeat {
        println!("{total}");
    }
    Ok(())
}

/// Prints a message.
fn run_echo(mut args: noargs::RawArgs) -> noargs::Result<()> {
    let upper = noargs::flag("upper")
        .short('u')
        .doc("Uppercase the message")
        .take(&mut args)
        .is_present();
    let text: String = noargs::arg("<TEXT>")
        .doc("Message text")
        .example("hello world")
        .take(&mut args)
        .then(|a| a.value().parse())?;

    if let Some(help) = args.finish()? {
        print!("{help}");
        return Ok(());
    }

    if upper {
        println!("{}", text.to_uppercase());
    } else {
        println!("{text}");
    }
    Ok(())
}
