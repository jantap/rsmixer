#[macro_export]
macro_rules! draw_rect {
    ($stdout:expr, $char:expr, $rect:expr, $style:expr) => {
        let s = (0..$rect.width).map(|_| $char).collect::<String>();
        for y in $rect.y..$rect.y + $rect.height {
            execute!($stdout, crossterm::cursor::MoveTo($rect.x, y)).unwrap();
            write!($stdout, "{}", $style.apply(s.clone())).unwrap();
        }
    };
}

#[macro_export]
macro_rules! draw_range {
    ($stdout:expr, $char:expr, $xrange:expr, $yrange:expr, $style:expr) => {
        let s = ($xrange).map(|_| $char).collect::<String>();
        let x = ($xrange).next().unwrap();
        for y in $yrange {
            execute!($stdout, crossterm::cursor::MoveTo(x, y)).unwrap();
            write!($stdout, "{}", $style.apply(s.clone())).unwrap();
        }
    };
}

#[macro_export]
macro_rules! draw_at {
    ($stdout:expr, $char:expr, $x:expr, $y:expr, $style:expr) => {
        execute!($stdout, crossterm::cursor::MoveTo($x, $y)).unwrap();
        write!($stdout, "{}", $style.apply($char)).unwrap();
    };
}

#[macro_export]
macro_rules! repeat_string {
    ($str:expr, $times:expr) => {
        (0..$times).map(|_| $str).collect::<String>()
    };
}
