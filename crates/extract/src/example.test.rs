mod Example {
    fn hello() {
        // comment 1
        let x = format_t!("hello");
        let x = format_t!("views.message.title", locale = "en", name = "Jason");
        // comment 3
        let x = format_t!("views.message.description", name = "Jason");

        // comment 4
        {
            format_t!(r##"Use YAML for mapping localized text, 
            and support mutiple YAML files merging."##);

            format_t!(r##"Use YAML for mapping localized text,
and support mutiple YAML files merging."##);
        }

        format_t!("The table below describes some of those behaviours.");
        // Will remove spaces for avoid duplication.
        format_t!("The table     below describes some     of those behaviours.");
    }
}
