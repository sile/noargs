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
    // Otherwise values like "-v" — or any unknown "--bogus" — can be silently
    // consumed as positional arguments.
    //
    // Required option / positional: set example("...") so help mode renders
    // a meaningful Usage / Example line. Optional fields covered by default()
    // or present_and_then() do not need example().
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

    let input: String = noargs::arg("<INPUT>")
        .doc("Input file path")
        .example("input.txt")
        .take(&mut args)
        .then(|a| a.value().parse())?;
    // Optional positional with default(): the value is always a String,
    // collapsing absent / present-as-out.txt into one branch.
    // For "absent vs present" distinction, use present_and_then() and bind
    // to Option<String> — see the README's [BAZ] example.
    let output: String = noargs::arg("[OUTPUT]")
        .doc("Output file path")
        .default("out.txt")
        .take(&mut args)
        .then(|a| a.value().parse())?;

    if let Some(help) = args.finish()? {
        // When help is requested, finish() returns the built help text.
        // Print it here and exit without running application logic.
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
