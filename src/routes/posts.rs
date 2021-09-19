use crate::{appstate::AppState, templates};
use std::fs;
use trillium::{conn_try, conn_unwrap, Conn, KnownHeaderName, Status};
use trillium_router::RouterConnExt;
use trillium_ructe::RucteConnExt;

pub async fn get_posts(conn: Conn) -> Conn {
    let state = conn.state::<AppState>().unwrap().to_owned();

    let blog = state.get_blog();

    let posts = blog
        .get_all_posts()
        .map(|key| blog.get_post(key).unwrap())
        .collect();

    conn.render_html(|o| templates::posts_html(o, blog, posts))
}

pub async fn get_post(conn: Conn) -> Conn {
    let slug = conn.param("slug").unwrap_or_default();
    let normalized_slug = slug.to_lowercase();
    if slug != normalized_slug {
        return conn
            .with_status(Status::PermanentRedirect)
            .with_header(KnownHeaderName::Location, normalized_slug)
            .halt();
    }

    let state = conn.state::<AppState>().unwrap().to_owned();
    let blog = state.get_blog();

    let post = conn_unwrap!(blog.get_post(&slug), conn);
    conn.render_html(|o| templates::post(o, blog, post))
}

pub async fn get_attachment(conn: Conn) -> Conn {
    let slug = conn.param("slug").unwrap_or_default();
    let normalized_slug = slug.to_lowercase();
    if slug != normalized_slug {
        return conn
            .with_status(Status::PermanentRedirect)
            .with_header(KnownHeaderName::Location, normalized_slug)
            .halt();
    }

    let state = conn.state::<AppState>().unwrap().to_owned();
    let blog = state.get_blog();

    let post = conn_unwrap!(blog.get_post(&slug), conn);
    let attachment_name = conn_unwrap!(conn.param("attachment"), conn);
    let attachment = conn_unwrap!(post.get_attachment(attachment_name), conn);

    let contents = conn_try!(fs::read(attachment.get_path()), conn);
    conn.with_status(Status::Ok)
        .with_header(KnownHeaderName::CacheControl, "max-age=31536000") // 1 year as a second
        .with_header(
            KnownHeaderName::ContentType,
            mime_guess::from_path(attachment.get_path())
                .first_or_octet_stream()
                .to_string(),
        )
        .with_body(contents)
        .halt()
}
