// silcrow/src/response/macros.rs
// JSON builder macro
// These macros are designed to provide a convenient and flexible way to create responses in Silcrow handlers. They support both JSON and HTML responses, as well as error and redirect responses, with optional parameters for status codes, headers, caching, and toast notifications. The `ok!` macro is the most powerful and flexible, allowing you to specify all aspects of the response in a single invocation, while the other macros provide simpler shortcuts for common cases.
#[macro_export]
macro_rules! json_ok {
    () => {
        $crate::response::Json::new()
    };

    ( $($key:ident : $value:expr),+ $(,)? ) => {{
        let mut res = $crate::response::Json::new()
        $(
            res = res.set(stringify!($key), $value);
        )+
        res
    }};
}

// HTML macro
#[macro_export]
macro_rules! html_ok {
    ($($tt:tt)*) => {
        $crate::response::Html::new(
            $crate::maud::html! { $($tt)* }
        )
    };

    ($status:expr, $($tt:tt)*) => {
        $crate::response::Html::new(
            $crate::maud::html! { $($tt)* }
        ).status($status)
    };
}

// Error macro
#[macro_export]
macro_rules! error {
    ($msg:expr) => {
        $crate::response::Error::new($msg)
    };

    ($msg:expr, $status:expr) => {
        $crate::response::Error::new($msg).status($status)
    };
}

// Redirect macro
#[macro_export]
macro_rules! redirect {
    ($url:expr) => {
        $crate::response::Redirect::new().to($url)
    };

    ($url:expr, $status:expr) => {
        $crate::response::Redirect::new().to($url).status($status)
    };
}
#[macro_export]
macro_rules! ok {
    // ---------------------------
    // JSON variant
    // ---------------------------
    (
        json { $($key:tt : $value:expr),* $(,)? }
        $(, status: $status:expr)?
        $(, header: ($hkey:expr, $hval:expr))*
        $(, no_cache: $no_cache:expr)?
        $(, toast: $toast:expr)?
    ) => {{
        let mut res = $crate::response::Json::new();

        $(
            res = res.set($key, $value);
        )*

        $(
            res = res.status($status);
        )?

        $(
            res = res.header($hkey, $hval);
        )*

        $(
            if $no_cache {
                res = res.no_cache();
            }
        )?

        $(
            res = res.toast($toast);
        )?

        res
    }};

    // ---------------------------
    // HTML variant
    // ---------------------------
    (
        html $content:expr
        $(, status: $status:expr)?
        $(, header: ($hkey:expr, $hval:expr))*
        $(, no_cache: $no_cache:expr)?
    ) => {{
        let mut res = $crate::response::Html::new($content);

        $(
            res = res.status($status);
        )?

        $(
            res = res.header($hkey, $hval);
        )*

        $(
            if $no_cache {
                res = res.no_cache();
            }
        )?

        res
    }};
}
/* example usage:

Using the macros
1. JSON (full power)
ok! {
    json {
        user: user,
        status: "ok"
    },
    status: StatusCode::CREATED,
    header: ("x-request-id", request_id),
    no_cache: true,
    toast: "Saved successfully"
}
2. JSON (minimal)
ok! {
    json { user: user }
}

3. HTML (full power)
ok! {
    html html! {
        h1 { "Hello, world!" }
    },
    status: StatusCode::OK,
    header: ("x-greeting", "welcome"),
    no_cache: true
}
4. HTML (minimal)
ok! {
    html html! {
        h1 { "Hello, world!" }
    }
}


more examples:
ok! {
    json { user: user }
}
.toast("Extra")
.header("x-debug", "1")



others are like this..
json_ok! {
    user: user,
    status: "ok"
}
.toast("Saved")

json_ok! { user: user }
    .toast("Saved")
    .status(StatusCode::CREATED)


html_ok!(html! {
    h1 { "Hello" }
})

error!("Invalid input")
error!("Not found", StatusCode::NOT_FOUND)

redirect!("/dashboard")
redirect!("/login", StatusCode::SEE_OTHER)

*/
