use std::time::Duration;

use chromiumoxide::browser::Browser;
use chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotFormat;
use chromiumoxide::handler::HandlerConfig;
use chromiumoxide::page::ScreenshotParams;
use futures::StreamExt;
use spider_fingerprint::spoof_viewport::get_random_viewport;
use spider_fingerprint::Fingerprint;
use tokio::fs::create_dir_all;

#[tokio::test]
async fn test_basic() -> Result<(), Box<dyn std::error::Error>> {
    create_dir_all("./download/").await?;

    let ua = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/137.0.0.0 Safari/537.36";

    let mut emulation_config = spider_fingerprint::EmulationConfiguration::setup_defaults(&ua);

    emulation_config.fingerprint = Fingerprint::None;
    emulation_config.tier = spider_fingerprint::configs::Tier::Basic;
    emulation_config.user_agent_data = Some(false);

    let vp = get_random_viewport();

    let viewport = vp.into();

    let emulation_script =
        spider_fingerprint::emulate(&ua, &emulation_config, &Some(&viewport), &None);

    let headers = spider_fingerprint::spoof_headers::emulate_headers(
        ua,
        &None,
        &None,
        true,
        &Some(viewport),
        &None,
        &Some(spider_fingerprint::spoof_headers::HeaderDetailLevel::Extensive),
    );

    let extra_headers = spider_fingerprint::spoof_headers::headers_to_hashmap(headers);

    let config = HandlerConfig {
        request_intercept: true,
        viewport: Some(chromiumoxide::handler::viewport::Viewport {
            width: viewport.width,
            height: viewport.height,
            device_scale_factor: viewport.device_scale_factor,
            emulating_mobile: viewport.emulating_mobile,
            is_landscape: viewport.is_landscape,
            has_touch: viewport.has_touch,
        }),
        extra_headers: Some(extra_headers),
        ..HandlerConfig::default()
    };

    let (mut browser, mut handler) =
        Browser::connect_with_config("http://localhost:9222", config.clone()).await?;

    let handle = tokio::task::spawn(async move {
        loop {
            let _ = handler.next().await.unwrap();
        }
    });

    let browser = browser.start_incognito_context().await?;

    let page = browser.new_page("about:blank").await?;

    let _ = tokio::join!(
        page.add_script_to_evaluate_on_new_document(emulation_script),
        page.set_user_agent(ua)
    );

    let _ = page.goto("https://abrahamjuliot.github.io/creepjs/").await;
    let _ = page.wait_for_navigation().await;

    tokio::time::sleep(Duration::from_millis(15_000)).await;

    page.save_screenshot(
        ScreenshotParams::builder()
            .format(CaptureScreenshotFormat::Png)
            .full_page(true)
            .omit_background(true)
            .build(),
        "./download/creep-page.png",
    )
    .await?;

    tokio::select! {
        _ = tokio::time::sleep(Duration::from_millis(500)) => {}
        _ = handle => {}
    };

    browser.quit_incognito_context().await?;

    Ok(())
}
