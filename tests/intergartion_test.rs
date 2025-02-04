rust_i18n::i18n!("./tests/locales");

#[cfg(test)]
mod tests {
    use rust_i18n::format_t;
    use rust_i18n::locale;

    #[test]
    fn it_foo_title() {
        assert_eq!(format_t!("hello"), "Foo - Hello, World!");
    }

    #[test]
    fn it_t() {
        assert_eq!(format_t!("hello"), "Bar - Hello, World!");

        // Vars
        assert_eq!(
            format_t!("a.very.nested.message"),
            "Hello, %{name}. Your message is: %{msg}"
        );
        assert_eq!(
            format_t!("a.very.nested.message", name = "Jason"),
            "Hello, Jason. Your message is: %{msg}"
        );
        assert_eq!(
            format_t!("a.very.nested.message", name = "Jason", msg = "Bla bla"),
            "Hello, Jason. Your message is: Bla bla"
        );

        rust_i18n::set_locale("de");
        assert_eq!(format_t!("messages.hello", name = "world"), "Hallo, world!");

        rust_i18n::set_locale("en");
        assert_eq!(format_t!("messages.hello", name = "world"), "Hello, world!");
    }

    #[test]
    fn it_t_with_locale_and_args() {
        assert_eq!(format_t!("hello", locale = "de"), "Bar - Hallo Welt!");
        assert_eq!(format_t!("hello", locale = "en"), "Bar - Hello, World!");

        rust_i18n::set_locale("en");
        assert_eq!(format_t!("messages.hello", name = "Jason"), "Hello, Jason!");
        assert_eq!(
            format_t!("messages.hello", locale = "en", name = "Jason"),
            "Hello, Jason!"
        );
        assert_eq!(
            format_t!("messages.hello", name = "Jason", locale = "en"),
            "Hello, Jason!"
        );
        assert_eq!(
            format_t!("messages.hello", locale = "de", name = "Jason"),
            "Hallo, Jason!"
        );
    }

    #[test]
    fn it_with_merge_file() {
        rust_i18n::set_locale("en");
        assert_eq!(format_t!("user.title"), "User Title");
        assert_eq!(format_t!("messages.user.title"), "Message User Title");
    }

    #[test]
    fn it_support_expr() {
        rust_i18n::set_locale("en");
        let name = "Jason Lee";
        let locale = "en";

        let key = "messages.hello";

        assert_eq!(
            format_t!("messages.hello", name = name),
            "Hello, Jason Lee!"
        );
        assert_eq!(format_t!(key, name = name), "Hello, Jason Lee!");

        assert_eq!(
            format_t!("messages.hello", name = &name.to_string()),
            "Hello, Jason Lee!"
        );
        assert_eq!(
            format_t!("messages.hello", name = &format!("this is {}", name)),
            "Hello, this is Jason Lee!"
        );

        assert_eq!(
            format_t!("messages.hello", locale = locale),
            "Hello, %{name}!"
        );

        assert_eq!(
            format_t!("messages.hello", name = name, locale = locale),
            "Hello, Jason Lee!"
        );
        assert_eq!(
            format_t!("messages.hello", name = name, locale = &locale.to_string()),
            "Hello, Jason Lee!"
        );
    }
}
