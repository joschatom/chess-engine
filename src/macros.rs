#[macro_export]
macro_rules! sliders_gen {
    (@$name:ident$($r:literal:$l:literal,)*) => {{
        #[allow(non_upper_case_globals)]
        static $name : &'static [Slider] = &[$(Slider::new($l, $r),)*];
        $name
    }};
}
