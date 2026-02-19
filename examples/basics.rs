fn main() -> noargs::Result<()> {
    let mut args = noargs::raw_args();
    args.metadata_mut().app_name = env!("CARGO_PKG_NAME");
    args.metadata_mut().app_description = env!("CARGO_PKG_DESCRIPTION");

    if noargs::VERSION_FLAG.take(&mut args).is_present() {
        println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    noargs::HELP_FLAG.take_help(&mut args);

    // Important: call flag()/opt() before arg().
    // Otherwise values like "-v" can be consumed as positional arguments.
    let verbose = noargs::flag("verbose")
        .short('v')
        .doc("Enable verbose output")
        .take(&mut args)
        .is_present();
    let dry_run = noargs::flag("dry-run")
        .doc("Print parsed values and exit without running processing")
        .take(&mut args)
        .is_present();
    let retries: usize = noargs::opt("retries")
        .short('r')
        .ty("N")
        .doc("How many times to retry")
        .default("1")
        .take(&mut args)
        .then(|o| o.value().parse())?;
    let endpoint: String = noargs::opt("endpoint")
        .ty("URL")
        .doc("Server endpoint")
        .env("NOARGS_ENDPOINT")
        .default("http://localhost:8080")
        .take(&mut args)
        .then(|o| o.value().parse())?;
    // Important: a required option should set example()
    // so help mode can still produce meaningful usage/example text.
    // Optional options are fine without example() when you use
    // default() or present_and_then().
    let format: String = noargs::opt("format")
        .ty("FORMAT")
        .doc("Output format")
        .example("json")
        .take(&mut args)
        .then(|o| o.value().parse())?;
    let timeout_secs: Option<u64> = noargs::opt("timeout")
        .ty("SECONDS")
        .doc("Optional timeout in seconds")
        .take(&mut args)
        .present_and_then(|o| o.value().parse())?;

    // Important: a required positional argument should set example()
    // so help mode can still produce meaningful usage/example text.
    // Optional positional arguments are fine without example() when you use
    // default() or present_and_then().
    let input: String = noargs::arg("<INPUT>")
        .doc("Input file path")
        .example("input.txt")
        .take(&mut args)
        .then(|a| a.value().parse())?;
    let output: String = noargs::arg("[OUTPUT]")
        .doc("Output file path")
        .default("out.txt")
        .take(&mut args)
        .then(|a| a.value().parse())?;

    if let Some(help) = args.finish()? {
        print!("{help}");
        return Ok(());
    }

    if dry_run {
        println!(
            "dry-run: verbose={verbose}, retries={retries}, endpoint={endpoint}, format={format}, timeout_secs={timeout_secs:?}, input={input}, output={output}"
        );
        return Ok(());
    }

    println!("processing: {input} -> {output}");
    println!("using endpoint={endpoint}, format={format}, retries={retries}");
    if verbose {
        println!("verbose: timeout_secs={timeout_secs:?}");
    }

    Ok(())
}
