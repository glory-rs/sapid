//! Browser history services for the Rust documentation application.
use crate::showcase_navigation::Page;
use glory::Cage;
pub fn request(request: sapid::AnchorRequest) {
    if let Err(error) = navigate(&request.href) {
        #[cfg(target_arch = "wasm32")]
        browser::report(error);
        #[cfg(not(target_arch = "wasm32"))]
        let _ = error;
    }
}
pub fn palette_request(request: sapid::AnchorRequest) {
    #[cfg(target_arch = "wasm32")]
    let result = browser::palette_navigate(&request.href);
    #[cfg(not(target_arch = "wasm32"))]
    let result = {
        let _ = request;
        Ok::<(), String>(())
    };
    if let Err(error) = result {
        #[cfg(target_arch = "wasm32")]
        browser::report(error);
        #[cfg(not(target_arch = "wasm32"))]
        let _ = error;
    }
}
pub fn navigate(url: &str) -> Result<(), String> {
    #[cfg(target_arch = "wasm32")]
    {
        browser::navigate(url)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = url;
        Ok(())
    }
}
pub fn present(page: &Page, source: Cage<Page>) -> Result<(), String> {
    #[cfg(target_arch = "wasm32")]
    {
        browser::present(page, source)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (page, source);
        Ok(())
    }
}
#[cfg(target_arch = "wasm32")]
mod browser;
