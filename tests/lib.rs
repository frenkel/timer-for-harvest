#[cfg(test)]
mod test {
    #[test]
    fn should_convert_duration_correctly() {
        assert_eq!("1:00", timer_for_harvest::f32_to_duration_str(1.0));
        assert_eq!("0:01", timer_for_harvest::f32_to_duration_str(1.0 / 60.0));
        assert_eq!("0:05", timer_for_harvest::f32_to_duration_str(5.0 / 60.0));
        assert_eq!("0:10", timer_for_harvest::f32_to_duration_str(10.0 / 60.0));
    }

    #[test]
    fn should_not_crash_duration_str_to_f32() {
        assert_eq!(0.0, timer_for_harvest::duration_str_to_f32("0:00"));
        assert_eq!(1.5, timer_for_harvest::duration_str_to_f32("1:30"));
        assert_eq!(1.0, timer_for_harvest::duration_str_to_f32("1"));
    }

    #[test]
    fn should_parse_account_id() {
        assert_eq!("123", timer_for_harvest::parse_account_details("GET /?access_token=abc&scope=harvest%3A123").1);
        assert_eq!("123", timer_for_harvest::parse_account_details("GET /?scope=harvest%3A123&access_token=abc").1);
    }

    #[test]
    fn should_parse_access_token() {
        assert_eq!("abc", timer_for_harvest::parse_account_details("GET /?access_token=abc&scope=harvest%3A123").0);
        assert_eq!("abc", timer_for_harvest::parse_account_details("GET /?scope=harvest%3A123&access_token=abc").0);
    }

    #[test]
    fn should_parse_expires_in() {
        assert_eq!("123", timer_for_harvest::parse_account_details("GET /?expires_in=123&scope=harvest%3A456").2);
        assert_eq!("123", timer_for_harvest::parse_account_details("GET /?scope=harvest%3A456&expires_in=123").2);
    }
}
