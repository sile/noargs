fn main() -> noargs::Result<()> {
    let mut args = noargs::raw_args();
    args.metadata_mut().app_name = env!("CARGO_PKG_NAME");
    args.metadata_mut().app_description = env!("CARGO_PKG_DESCRIPTION");

    if noargs::VERSION_FLAG.take(&mut args).is_present() {
        println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    noargs::HELP_FLAG.take_help(&mut args);

    // In real applications, split each `try_run_*` command into separate modules/files.
    let _ = try_run_hello(&mut args)? || try_run_sum(&mut args)? || try_run_echo(&mut args)?;

    if let Some(help) = args.finish()? {
        print!("{help}");
    }

    Ok(())
}

fn try_run_hello(args: &mut noargs::RawArgs) -> noargs::Result<bool> {
    if !noargs::cmd("hello")
        .doc("Print a greeting")
        .take(args)
        .is_present()
    {
        return Ok(false);
    }

    let loud = noargs::flag("loud")
        .short('l')
        .doc("Print greeting in upper case")
        .take(args)
        .is_present();
    let name: String = noargs::arg("<NAME>")
        .doc("Name to greet")
        .example("Alice")
        .take(args)
        .then(|a| a.value().parse())?;

    if args.metadata().help_mode {
        return Ok(true);
    }

    let message = format!("Hello, {name}!");
    if loud {
        println!("{}", message.to_uppercase());
    } else {
        println!("{message}");
    }
    Ok(true)
}

fn try_run_sum(args: &mut noargs::RawArgs) -> noargs::Result<bool> {
    if !noargs::cmd("sum")
        .doc("Add two integers")
        .take(args)
        .is_present()
    {
        return Ok(false);
    }

    let repeat: usize = noargs::opt("repeat")
        .short('r')
        .ty("N")
        .doc("Print result multiple times")
        .default("1")
        .take(args)
        .then(|o| o.value().parse())?;
    let left: i64 = noargs::arg("<LEFT>")
        .doc("Left operand")
        .example("3")
        .take(args)
        .then(|a| a.value().parse())?;
    let right: i64 = noargs::arg("<RIGHT>")
        .doc("Right operand")
        .example("4")
        .take(args)
        .then(|a| a.value().parse())?;

    if args.metadata().help_mode {
        return Ok(true);
    }

    let total = left + right;
    for _ in 0..repeat {
        println!("{total}");
    }
    Ok(true)
}

fn try_run_echo(args: &mut noargs::RawArgs) -> noargs::Result<bool> {
    if !noargs::cmd("echo")
        .doc("Print a message")
        .take(args)
        .is_present()
    {
        return Ok(false);
    }

    let upper = noargs::flag("upper")
        .short('u')
        .doc("Uppercase the message")
        .take(args)
        .is_present();
    let text: String = noargs::arg("<TEXT>")
        .doc("Message text")
        .example("hello world")
        .take(args)
        .then(|a| a.value().parse())?;

    if args.metadata().help_mode {
        return Ok(true);
    }

    if upper {
        println!("{}", text.to_uppercase());
    } else {
        println!("{text}");
    }
    Ok(true)
}
