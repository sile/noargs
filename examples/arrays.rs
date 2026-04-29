fn main() -> noargs::Result<()> {
    let mut args = noargs::raw_args();
    args.metadata_mut().app_name = env!("CARGO_PKG_NAME");
    args.metadata_mut().app_description = env!("CARGO_PKG_DESCRIPTION");

    if noargs::VERSION_FLAG.take(&mut args).is_present() {
        println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    noargs::HELP_FLAG.take_help(&mut args);

    // Same-name options can be collected by calling take() in a loop.
    let include_opt = noargs::opt("include")
        .short('I')
        .ty("PATH")
        .doc("Include path (can be specified multiple times)");
    let mut includes = Vec::<String>::new();
    while let Some(path) = include_opt
        .take(&mut args)
        .present_and_then(|o| o.value().parse())?
    {
        includes.push(path);
    }

    let label_opt = noargs::opt("label")
        .short('l')
        .ty("LABEL")
        .doc("Label value (can be specified multiple times)");
    let mut labels = Vec::<String>::new();
    while let Some(label) = label_opt
        .take(&mut args)
        .present_and_then(|o| o.value().parse())?
    {
        labels.push(label);
    }

    let output: String = noargs::opt("output")
        .short('o')
        .ty("PATH")
        .doc("Output path")
        .default("summary.txt")
        .take(&mut args)
        .then(|o| o.value().parse())?;

    // Positional arrays are handled with one required argument + optional rest.
    // Naming convention: use `<NAME>...` for required-many and `[NAME]...` for optional-many.
    // The `<>` / `[]` / `...` markers are cosmetic (used only in help output);
    // required-ness is enforced below: `.then()` makes the first input required,
    // the loop with `.present_and_then()` consumes zero or more rest inputs.
    let first_input: String = noargs::arg("<INPUT>")
        .doc("First input (required)")
        .example("a.txt")
        .take(&mut args)
        .then(|a| a.value().parse())?;
    let rest_input_arg = noargs::arg("[INPUT]...").doc("Additional inputs");
    let mut inputs = vec![first_input];
    while let Some(input) = rest_input_arg
        .take(&mut args)
        .present_and_then(|a| a.value().parse())?
    {
        inputs.push(input);
    }

    if let Some(help) = args.finish()? {
        print!("{help}");
        return Ok(());
    }

    println!("includes={includes:?}");
    println!("labels={labels:?}");
    println!("inputs={inputs:?}");
    println!("output={output}");
    Ok(())
}
