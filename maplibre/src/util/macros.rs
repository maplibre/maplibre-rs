#[macro_export]
macro_rules! match_downcast {
    ( $any:expr, { $( $bind:ident : $ty:ty => $arm:expr ),*, _ => $default:expr } ) => (
        $(
            if $any.is::<$ty>() {
                let $bind = $any.downcast::<$ty>().unwrap();
                $arm
            } else
        )*
        {
            $default
        }
    )
}

#[macro_export]
macro_rules! match_downcast_ref {
    ( $any:expr, { $( $bind:ident : $ty:ty => $arm:expr ),*, _ => $default:expr } ) => (
        $(
            if $any.is::<$ty>() {
                let $bind = $any.downcast_ref::<$ty>().unwrap();
                $arm
            } else
        )*
        {
            $default
        }
    )
}

#[macro_export]
macro_rules! match_downcast_mut {
    ( $any:expr, { $( $bind:ident : $ty:ty => $arm:expr ),*, _ => $default:expr } ) => (
        $(
            if $any.is::<$ty>() {
                let $bind = $any.downcast_mut::<$ty>().unwrap();
                $arm
            } else
        )*
        {
            $default
        }
    )
}
