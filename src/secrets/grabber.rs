use headless_chrome::protocol::cdp::Page;
use headless_chrome::Browser;
use serde_json::Value;
use std::time::Duration;
use tracing::{debug, error, info, instrument, warn};

const STEALTH_JS: &str = r"(function () {
	
	Object.defineProperty(navigator, 'webdriver', { get: () => false });

	Object.defineProperty(navigator, 'languages', { get: () => ['en-US', 'en'] });

	Object.defineProperty(navigator, 'plugins', { get: () => [1, 2, 3, 4, 5] });

	window.chrome = { runtime: {} };

	const originalQuery = window.navigator.permissions.query;
	window.navigator.permissions.query = (parameters) => (
		parameters.name === 'notifications' ?
			Promise.resolve({ state: Notification.permission }) :
			originalQuery(parameters)
	);

	const getParameter = WebGLRenderingContext.prototype.getParameter;
	WebGLRenderingContext.prototype.getParameter = function (param) {
		if (param === 37445) return 'Intel Inc.';
		if (param === 37446) return 'Intel Iris OpenGL Engine';
		return getParameter.call(this, param);
	};
})();";

const HOOK_JS: &str = r"(()=>{if(globalThis.__secretHookInstalled)return;globalThis.__secretHookInstalled=true;globalThis.__captures=[];Object.defineProperty(Object.prototype,'secret',{configurable:true,set:function(v){try{__captures.push({secret:v,version:this.version,obj:this});}catch(e){}Object.defineProperty(this,'secret',{value:v,writable:true,configurable:true,enumerable:true});}});})();";

#[instrument(skip_all)]
pub async fn grab_live() -> Result<Vec<Value>, Box<dyn std::error::Error>> {
    info!("Launching headless browser...");

    let browser = Browser::default()?;
    let tab = browser.new_tab()?;

    info!("Installing stealth script to evaluate on new document...");
    tab.call_method(Page::AddScriptToEvaluateOnNewDocument {
        source: STEALTH_JS.to_string(),
        world_name: None,
        include_command_line_api: None,
        run_immediately: None,
    })?;

    info!("Installing secret hook to evaluate on new document...");
    tab.call_method(Page::AddScriptToEvaluateOnNewDocument {
        source: HOOK_JS.to_string(),
        world_name: None,
        include_command_line_api: None,
        run_immediately: None,
    })?;

    info!("Opening https://open.spotify.com");
    match tab.navigate_to("https://open.spotify.com") {
        Ok(_) => info!("Navigation successful"),
        Err(e) => {
            error!("Navigation error: {:?}", e);
            return Err(e.into());
        }
    }

    info!("Waiting for page to load (3 seconds)...");
    std::thread::sleep(Duration::from_secs(3));

    info!("Evaluating captured secrets...");

    let captures = match tab.evaluate(
        "(function() { try { return JSON.stringify(globalThis.__captures || []); } catch(e) { return '[]'; } })()",
        false
    ) {
        Ok(remote_obj) => {
            debug!("Received remote object");
            remote_obj.value.map_or_else(
                || {
                    error!("No value in remote object");
                    Vec::new()
                },
                |value| {
                    value.as_str().map_or_else(
                        || {
                            error!("Value is not a string: {:?}", value);
                            Vec::new()
                        },
                        |json_str| {
                            debug!("Got JSON string: {}", json_str);
                            match serde_json::from_str::<Vec<Value>>(json_str) {
                                Ok(arr) => {
                                    info!("Successfully parsed array with {} items", arr.len());
                                    arr
                                }
                                Err(e) => {
                                    error!("Failed to parse JSON: {}", e);
                                    Vec::new()
                                }
                            }
                        },
                    )
                },
            )
        }
        Err(e) => {
            error!("Failed to evaluate: {:?}", e);
            Vec::new()
        }
    };

    if captures.is_empty() {
        warn!("No secrets captured - __captures is empty");
    } else {
        info!("Captured {} items successfully", captures.len());
        for cap in &captures {
            if let Some(secret) = cap.get("secret").and_then(Value::as_str) {
                if let Some(version) = cap.get("version").and_then(Value::as_i64) {
                    info!("Secret({}): {}", version, secret);
                }
            }
        }
    }

    Ok(captures)
}
