#[cfg(test)]
mod test {
    #[test]
    fn should_convert_duration_correctly() {
        assert_eq!("1:00", harvest::f32_to_duration_str(1.0));
        assert_eq!("0:01", harvest::f32_to_duration_str(1.0 / 60.0));
        assert_eq!("0:05", harvest::f32_to_duration_str(5.0 / 60.0));
        assert_eq!("0:10", harvest::f32_to_duration_str(10.0 / 60.0));
    }

    #[test]
    fn should_not_crash_duration_str_to_f32() {
        assert_eq!(0.0, harvest::duration_str_to_f32("0:00"));
        assert_eq!(1.5, harvest::duration_str_to_f32("1:30"));
        assert_eq!(1.0, harvest::duration_str_to_f32("1"));
    }
}
